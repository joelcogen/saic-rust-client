use super::*;

#[derive(Default)]
struct FakeSession {
    logins: usize,
    commands: usize,
    fail_commands: bool,
    payloads: Vec<serde_json::Value>,
    reads: Vec<(ReadCommand, String)>,
    fail_reads: bool,
    logged_in: bool,
    fail_on: Option<usize>,
}

impl Session for FakeSession {
    async fn read(&mut self, operation: ReadCommand, vin: &str) -> Result<String, Error> {
        self.reads.push((operation, vin.to_owned()));
        if self.fail_reads {
            Err(Error::Timeout)
        } else {
            Ok("Read completed (data redacted)".into())
        }
    }
    async fn command(&mut self, _vin: &str, _command: &Command) -> Result<CommandOutcome, Error> {
        self.commands += 1;
        self.payloads.push(serde_json::to_value(_command).unwrap());
        if self.fail_commands {
            return Err(Error::Timeout);
        }
        Ok(CommandOutcome::default())
    }
    async fn login(&mut self) -> Result<(), Error> {
        self.logins += 1;
        self.logged_in = false;
        if self.fail_on == Some(self.logins) {
            return Err(Error::Timeout);
        }
        self.logged_in = true;
        Ok(())
    }

    fn remaining(&self) -> Option<Duration> {
        self.logged_in.then_some(Duration::from_secs(3600))
    }

    fn logout(&mut self) {
        self.logged_in = false;
    }
}

#[test]
fn reads_quoted_credentials_and_optional_phone_code() {
    let settings = read_settings(
        b"# test account\nSAIC_USERNAME='test@example.invalid'\nSAIC_PASSWORD='a # $ + password'\n"
            .as_slice(),
    )
    .unwrap();
    assert_eq!(settings.username, "test@example.invalid");
    assert_eq!(settings.password, "a # $ + password");
    assert_eq!(settings.country_code, None);

    let settings = read_settings(
        b"SAIC_USERNAME=123\nSAIC_PASSWORD=synthetic\nSAIC_PHONE_COUNTRY_CODE=32\n".as_slice(),
    )
    .unwrap();
    assert_eq!(settings.country_code.as_deref(), Some("32"));
}

#[test]
fn rejects_missing_empty_or_malformed_credentials_without_printing_values() {
    for input in [
        "",
        "SAIC_USERNAME=test",
        "SAIC_USERNAME=\nSAIC_PASSWORD=secret",
        "SAIC_USERNAME=test\nSAIC_PASSWORD=",
        "SAIC_USERNAME=test\nSAIC_PASSWORD=secret\nSAIC_PHONE_COUNTRY_CODE=",
        "SAIC_PASSWORD='secret",
    ] {
        let error = read_settings(input.as_bytes())
            .err()
            .expect("must reject config");
        assert!(!error.contains("secret"));
    }
}

#[tokio::test]
async fn logs_in_once_before_commands_and_exits_without_extra_requests() {
    let mut client = FakeSession::default();
    let mut output = Vec::new();
    run_session(
        &mut client,
        b"\nhelp\nstatus\nunknown\nexit\nlogin\n".as_slice(),
        &mut output,
    )
    .await
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(output.starts_with("Logging in...\nLogged in."));
    assert!(output.contains("expires in 3600 seconds"));
    assert!(output.contains("Commands:"));
    assert!(output.contains("Unknown command"));
    assert!(output.ends_with("Bye.\n"));
    assert_eq!(client.logins, 1);
    assert!(!client.logged_in);
}

#[tokio::test]
async fn failed_startup_stops_before_reading_commands() {
    let mut client = FakeSession {
        fail_on: Some(1),
        ..Default::default()
    };
    let mut output = Vec::new();
    assert!(
        run_session(&mut client, b"login\nexit\n".as_slice(), &mut output)
            .await
            .is_err()
    );
    assert_eq!(client.logins, 1);
    assert_eq!(output, b"Logging in...\n");
}

#[tokio::test]
async fn failed_command_stays_interactive_and_retries_only_when_asked() {
    let mut client = FakeSession {
        fail_on: Some(2),
        ..Default::default()
    };
    let mut output = Vec::new();
    run_session(
        &mut client,
        b"logout\nstatus\nlogin\nstatus\nlogin\nstatus\nquit\n".as_slice(),
        &mut output,
    )
    .await
    .unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("Local session cleared"));
    assert!(output.contains("Logged out or expired"));
    assert!(output.contains("Login failed: request timed out"));
    assert!(output.contains("expires in 3600 seconds"));
    assert_eq!(client.logins, 3);
    assert!(!client.logged_in);
}

#[tokio::test]
async fn eof_exits_cleanly() {
    let mut client = FakeSession::default();
    let mut output = Vec::new();
    run_session(&mut client, b"".as_slice(), &mut output)
        .await
        .unwrap();
    assert_eq!(client.logins, 1);
    assert!(!client.logged_in);
    assert!(output.ends_with(b"Bye.\n"));
}

#[tokio::test]
async fn handles_both_command_envelopes_without_login_or_refresh() {
    let mut client = FakeSession::default();
    let mut output = Vec::new();
    let input = br#"vehicle-command SYNTHETICVIN {"rvcReqType":"0","rvcParams":[]}
charging-command SYNTHETICVIN {"chrgCtrlReq":1,"tboxEleccLckCtrlReq":0,"tboxV2XReq":0}
charging-command SYNTHETICVIN {"chrgCtrlReq":1}
vehicle-command SYNTHETICVIN malformed
exit
"#;
    run_session(&mut client, input.as_slice(), &mut output)
        .await
        .unwrap();
    assert_eq!(client.commands, 2);
    assert_eq!(client.logins, 1);
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("Unknown(Cancelled)"));
    assert!(text.contains("Invalid command fields"));
    assert!(text.contains("Invalid command JSON"));
    assert!(!text.contains("SYNTHETICVIN"));
    assert!(HELP.contains("vehicle-command"));
    assert!(HELP.contains("charging-command"));
}

#[tokio::test]
async fn all_reads_are_explicit_and_failures_do_not_trigger_refreshes() {
    for fail_reads in [false, true] {
        let mut client = FakeSession {
            fail_reads,
            ..Default::default()
        };
        let mut output = Vec::new();
        run_session(&mut client, b"vehicles\nvehicle-cache vin\nvehicle-live vin\ncharging-status vin\ncharging-management vin\ncharging-settings vin\nbattery-heating-schedule vin\nexit\n".as_slice(), &mut output).await.unwrap();
        assert_eq!(
            client.reads.iter().map(|(r, _)| *r).collect::<Vec<_>>(),
            vec![
                ReadCommand::Vehicles,
                ReadCommand::Cached,
                ReadCommand::Live,
                ReadCommand::Charging,
                ReadCommand::Management,
                ReadCommand::Settings,
                ReadCommand::BatteryHeating
            ]
        );
        assert_eq!(client.reads[0].1, "");
        assert!(client.reads[1..].iter().all(|(_, vin)| vin == "vin"));
        assert_eq!(client.commands, 0);
        assert_eq!(client.logins, 1);
        if fail_reads {
            assert_eq!(
                String::from_utf8(output)
                    .unwrap()
                    .matches("Read failed")
                    .count(),
                7
            );
        }
    }
}

#[tokio::test]
async fn invalid_read_arguments_never_send() {
    let mut client = FakeSession::default();
    let mut output = Vec::new();
    for command in [
        "vehicle-cache",
        "vehicle-live",
        "charging-status",
        "charging-management",
        "charging-settings",
        "battery-heating-schedule",
    ] {
        assert!(HELP.contains(command));
        assert!(parse_read(command).is_err());
        assert!(parse_read(&format!("{command} vin extra")).is_err());
    }
    run_session(
        &mut client,
        b"vehicles extra\nvehicle-cache\nvehicle-live vin extra\nexit\n".as_slice(),
        &mut output,
    )
    .await
    .unwrap();
    assert!(client.reads.is_empty());
}

#[tokio::test]
async fn controls_can_be_followed_by_one_explicit_refresh() {
    let mut client = FakeSession::default();
    let mut output = Vec::new();
    run_session(&mut client, b"charging-command vin {\"chrgCtrlReq\":1,\"tboxEleccLckCtrlReq\":0,\"tboxV2XReq\":0}\ncharging-command vin {\"chrgCtrlReq\":0,\"tboxEleccLckCtrlReq\":1,\"tboxV2XReq\":0}\nvehicle-live vin\nexit\n".as_slice(), &mut output).await.unwrap();
    assert_eq!(client.commands, 2);
    assert_eq!(client.reads, vec![(ReadCommand::Live, "vin".into())]);
}

#[tokio::test]
async fn climate_commands_send_once_even_when_repeated_or_failed_without_reads() {
    for fail_commands in [false, true] {
        let mut client = FakeSession {
            fail_commands,
            ..Default::default()
        };
        let mut output = Vec::new();
        let input = "ac-start vin eh32 17\nac-stop vin\nac-blowing vin\ndefrost-start vin\ndefrost-stop vin\nac-temperature vin mzs3e 31\nac-temperature vin mzs3e 31\nclimate-control vin 2 omit 8\nexit\n";
        run_session(&mut client, input.as_bytes(), &mut output)
            .await
            .unwrap();
        assert_eq!(client.commands, 8);
        assert_eq!(client.logins, 1);
        assert!(client.reads.is_empty());
        assert_eq!(client.payloads[1], client.payloads[4]);
        assert_eq!(client.payloads[5], client.payloads[6]);
        assert_eq!(client.payloads[0]["rvcParams"][1]["paramValue"], "Aw==");
        assert_eq!(client.payloads[5]["rvcParams"][1]["paramValue"], "EQ==");
        for line in input.lines().take(8) {
            assert!(HELP.contains(line.split_whitespace().next().unwrap()));
        }
        if fail_commands {
            assert_eq!(
                String::from_utf8(output)
                    .unwrap()
                    .matches("Command failed")
                    .count(),
                8
            );
        }
    }
}

#[tokio::test]
async fn invalid_climate_inputs_never_reach_session_commands() {
    let invalid = [
        "ac-start",
        "ac-start vin",
        "ac-start vin unknown 22",
        "ac-start vin eh32 16",
        "ac-start vin eh32 32",
        "ac-start vin standard 15",
        "ac-start vin standard 29",
        "ac-temperature vin mzs3e 15",
        "ac-temperature vin mzs3e 32",
        "ac-temperature vin standard 22.5",
        "ac-temperature vin standard NaN",
        "ac-stop",
        "ac-stop vin extra",
        "ac-blowing",
        "defrost-start",
        "defrost-stop",
        "climate-control vin -1 on 8",
        "climate-control vin 256 off 8",
        "climate-control vin 0 omit -1",
        "climate-control vin 2 on 256",
        "climate-control vin 2 unknown 8",
        "climate-control vin 2 on",
        "climate-control vin 2 on 8 extra",
    ];
    for line in invalid {
        assert!(parse_climate(line).is_err(), "{line}");
    }
    let mut client = FakeSession::default();
    run_session(&mut client, invalid.join("\n").as_bytes(), &mut Vec::new())
        .await
        .unwrap();
    assert_eq!(client.commands, 0);
    assert!(client.reads.is_empty());
}

#[tokio::test]
async fn independent_heaters_dispatch_exact_payloads_without_reads_even_on_failure() {
    let fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../tests/fixtures/heaters.json")).unwrap();
    for fail_commands in [false, true] {
        let mut client = FakeSession {
            fail_commands,
            ..Default::default()
        };
        let mut input = String::new();
        for case in &fixtures {
            let name = format!("{}-heat", case["heater"].as_str().unwrap());
            assert!(HELP.contains(&name));
            let state = if case["enabled"].as_bool().unwrap() {
                "on"
            } else {
                "off"
            };
            input.push_str(&format!("{name} SYNTHETICVIN {state}\n"));
        }
        run_session(&mut client, input.as_bytes(), &mut Vec::new())
            .await
            .unwrap();
        assert_eq!(client.commands, 4);
        assert_eq!(client.logins, 1);
        assert!(client.reads.is_empty());
        for (mut actual, case) in client.payloads.into_iter().zip(&fixtures) {
            assert_eq!(
                actual.as_object_mut().unwrap().remove("kind"),
                Some(serde_json::json!("Vehicle"))
            );
            assert_eq!(actual, case["payload"]);
        }
    }
}

#[tokio::test]
async fn invalid_heater_arguments_never_send() {
    let mut input = String::new();
    for name in ["rear-window-heat", "steering-wheel-heat"] {
        for args in ["", "vin", "vin omit", "vin true", "vin 1", "vin on extra"] {
            let line = format!("{name} {args}");
            assert!(parse_climate(&line).is_err());
            input.push_str(&line);
            input.push('\n');
        }
    }
    let mut client = FakeSession::default();
    run_session(&mut client, input.as_bytes(), &mut Vec::new())
        .await
        .unwrap();
    assert_eq!(client.commands, 0);
    assert!(client.reads.is_empty());
}

#[tokio::test]
async fn lock_variants_dispatch_without_reads_even_on_failure() {
    let fixtures: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../tests/fixtures/locks.json")).unwrap();
    for fail_commands in [false, true] {
        let mut client = FakeSession {
            fail_commands,
            ..Default::default()
        };
        let mut input = String::new();
        for case in &fixtures {
            let name = case["cli"].as_str().unwrap();
            assert!(HELP.contains(name));
            input.push_str(&format!(
                "{name} SYNTHETICVIN {}\n",
                case["action"].as_str().unwrap_or("")
            ));
        }
        input.push_str("locks-control SYNTHETICVIN lock\nlocks-control SYNTHETICVIN unlock doors\nlocks-control SYNTHETICVIN unlock tailgate\n");
        let mut output = Vec::new();
        run_session(&mut client, input.as_bytes(), &mut output)
            .await
            .unwrap();
        assert_eq!(client.commands, 8);
        assert_eq!(client.logins, 1);
        assert!(client.reads.is_empty());
        assert_eq!(client.payloads[0], client.payloads[5]);
        assert_eq!(client.payloads[1], client.payloads[6]);
        assert_eq!(client.payloads[2], client.payloads[7]);
        for (mut actual, case) in client.payloads.into_iter().zip(&fixtures) {
            actual.as_object_mut().unwrap().remove("kind");
            // Command stores an empty list; the wire body emits null for locking.
            if actual["rvcReqType"] == "1" {
                assert_eq!(actual["rvcParams"], serde_json::json!([]));
                actual["rvcParams"] = serde_json::Value::Null;
            }
            assert_eq!(actual, case["payload"]);
        }
        let text = String::from_utf8(output).unwrap();
        if fail_commands {
            assert_eq!(text.matches("Command failed").count(), 8);
        } else {
            assert_eq!(text.matches("physical position unverified").count(), 8);
        }
    }
}

#[tokio::test]
async fn invalid_lock_and_tailgate_arguments_never_send() {
    let invalid = [
        "lock",
        "lock vin doors",
        "unlock",
        "unlock vin tailgate",
        "tailgate-open",
        "tailgate-open vin close",
        "tailgate-control vin",
        "tailgate-control vin stop",
        "tailgate-control vin 1",
        "tailgate-control vin open extra",
        "locks-control vin unlock",
        "locks-control vin lock doors",
        "locks-control vin lock tailgate",
        "locks-control vin unlock all",
        "locks-control vin unlock doors extra",
    ];
    for line in invalid {
        assert!(parse_locks(line).is_err(), "{line}");
    }
    let mut client = FakeSession::default();
    run_session(&mut client, invalid.join("\n").as_bytes(), &mut Vec::new())
        .await
        .unwrap();
    assert_eq!(client.commands, 0);
    assert!(client.reads.is_empty());
}

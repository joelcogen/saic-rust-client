use super::*;

#[derive(Default)]
struct FakeSession {
    logins: usize,
    commands: usize,
    logged_in: bool,
    fail_on: Option<usize>,
}

impl Session for FakeSession {
    async fn command(&mut self, _vin: &str, _command: &Command) -> Result<CommandOutcome, Error> {
        self.commands += 1;
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

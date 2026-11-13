use super::*;
use crate::{
    test_support::{config, headers, vectors},
    Credentials,
};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::StatusCode;

fn client() -> SaicClient {
    SaicClient::new(config(&vectors()["requests"][0])).unwrap()
}

fn now() -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(vectors()["timestamp_ms"].as_u64().unwrap())
}

fn success() -> Result<protocol::LoginResult, Error> {
    let response = &vectors()["response"];
    protocol::parse_login(
        StatusCode::OK,
        &headers(&response["headers"]),
        response["body"].as_str().unwrap().as_bytes(),
    )
}

#[test]
fn builds_login_requests_matching_python_without_sending_them() {
    for vector in vectors()["requests"].as_array().unwrap() {
        let client = SaicClient::new(config(vector)).unwrap();
        let request = client.login_request(now()).unwrap();
        assert_eq!(request.method(), "POST");
        assert_eq!(
            request.url().as_str(),
            "https://example.invalid/api.app/v1/oauth/token"
        );
        assert_eq!(
            request.body().unwrap().as_bytes().unwrap(),
            vector["body"].as_str().unwrap().as_bytes()
        );
        for (name, value) in &headers(&vector["headers"]) {
            assert_eq!(request.headers().get(name), Some(value));
        }
        assert_eq!(
            request.headers()["authorization"],
            "Basic c3dvcmQ6c3dvcmRfc2VjcmV0"
        );
        assert!(request.headers()["authorization"].is_sensitive());
        assert!(!request.headers().contains_key("blade-auth"));
        assert!(!client.is_logged_in());
    }
}

#[test]
fn configuration_rejects_invalid_values_without_network_access() {
    for url in [
        "not a URL",
        "http://example.invalid/",
        "https://user:password@example.invalid/",
        "https://example.invalid/?token=secret",
        "https://example.invalid/#fragment",
    ] {
        assert!(matches!(
            SaicClient::new(config(&vectors()["requests"][0]).with_base_url(url)),
            Err(Error::Configuration(_))
        ));
    }
    for credentials in [
        Credentials::email("", "password"),
        Credentials::email("user", ""),
        Credentials::phone("123", "", "password"),
    ] {
        assert!(matches!(
            SaicClient::new(ClientConfig::mg_europe(credentials)),
            Err(Error::Configuration(_))
        ));
    }
    assert!(matches!(
        SaicClient::new(config(&vectors()["requests"][0]).with_timeout(Duration::ZERO)),
        Err(Error::Configuration(_))
    ));
    assert!(matches!(
        SaicClient::new(config(&vectors()["requests"][0]).with_api_identity(
            "eu\r\ninjected: true",
            "tenant",
            "app"
        )),
        Err(Error::Configuration(_))
    ));
}

#[test]
fn endpoint_keeps_its_path_and_profile_overrides_reach_the_request() {
    let settings = config(&vectors()["requests"][0])
        .with_base_url("https://example.invalid/custom/root")
        .with_api_identity("au", "1234", "example.app");
    let client = SaicClient::new(settings).unwrap();
    let request = client.login_request(now()).unwrap();
    assert_eq!(
        request.url().as_str(),
        "https://example.invalid/custom/root/oauth/token"
    );
    assert_eq!(request.headers()["region"], "au");
    assert_eq!(request.headers()["tenant-id"], "1234");
    assert_ne!(
        request.body().unwrap().as_bytes().unwrap(),
        vectors()["requests"][0]["body"]
            .as_str()
            .unwrap()
            .as_bytes()
    );
}

#[tokio::test]
async fn successful_login_stores_session_and_logout_clears_it() {
    let mut client = client();
    client
        .login_with(now(), |_| async { success() })
        .await
        .unwrap();
    assert!(client.is_logged_in());
    assert!(client.session_remaining().unwrap() <= Duration::from_secs(3600));
    let deadline = client.session.as_ref().unwrap().expires_at;
    assert!(client
        .remaining_at(deadline - Duration::from_nanos(1))
        .is_some());
    assert_eq!(client.remaining_at(deadline), None);
    assert_eq!(client.remaining_at(deadline + Duration::from_secs(1)), None);
    assert!(!format!("{client:?}").contains("synthetic-token"));
    client.logout();
    assert!(!client.is_logged_in());
    assert!(client.session.is_none());
}

#[tokio::test]
async fn failed_relogin_clears_old_session_and_never_retries() {
    for error in [
        Error::Timeout,
        Error::Transport,
        Error::Decryption,
        Error::InvalidResponse("test"),
    ] {
        let mut client = client();
        client
            .login_with(now(), |_| async { success() })
            .await
            .unwrap();
        let mut calls = 0;
        let result = client
            .login_with(now(), |request| {
                calls += 1;
                assert!(!request.headers().contains_key("blade-auth"));
                async { Err(error) }
            })
            .await;
        assert!(result.is_err());
        assert_eq!(calls, 1);
        assert!(client.session.is_none());
    }
}

#[tokio::test]
async fn rejected_login_leaves_no_session() {
    let mut client = client();
    client
        .login_with(now(), |_| async { success() })
        .await
        .unwrap();
    let result = client
        .login_with(now(), |_| async {
            protocol::parse_login(
                StatusCode::OK,
                &Default::default(),
                br#"{"code":401,"message":"invalid credentials"}"#,
            )
        })
        .await;
    assert!(matches!(result, Err(Error::Authentication(_))));
    assert!(!client.is_logged_in());
}

#[tokio::test]
async fn an_unpolled_login_does_nothing_and_cancelled_login_leaves_no_session() {
    let mut client = client();
    client
        .login_with(now(), |_| async { success() })
        .await
        .unwrap();
    drop(client.login());
    assert!(client.is_logged_in());
    let waiting = client.login_with(now(), |_| std::future::pending());
    assert!(tokio::time::timeout(Duration::from_millis(1), waiting)
        .await
        .is_err());
    assert!(client.session.is_none());
}

#[tokio::test]
async fn invalid_clock_and_overflowing_expiry_leave_no_session() {
    let mut client = client();
    let result = client
        .login_with(UNIX_EPOCH - Duration::from_secs(1), |_| async {
            panic!("must not send")
        })
        .await;
    assert!(matches!(result, Err(Error::Clock)));
    let result = client
        .login_with(now(), |_| async {
            Ok(protocol::LoginResult {
                token: Zeroizing::new("test-token".into()),
                expires_in: u64::MAX,
            })
        })
        .await;
    assert!(matches!(result, Err(Error::InvalidResponse(_))));
    assert!(client.session.is_none());
}

#[test]
fn debug_output_hides_credentials_and_unvalidated_urls() {
    let credentials = Credentials::email("private@example.invalid", "private-password");
    assert!(!format!("{credentials:?}").contains("private"));
    let config = ClientConfig::mg_europe(credentials)
        .with_base_url("https://private:password@example.invalid/");
    assert!(!format!("{config:?}").contains("private"));
    assert!(!format!("{:?}", SaicClient::new(config).unwrap_err()).contains("private"));
}

fn command() -> crate::Command {
    crate::Command::Charging {
        charging: 1,
        port_lock: 0,
        v2x: 0,
    }
}
fn reply(code: i64, event: Option<&str>, data: bool) -> crate::command::Reply {
    let mut headers = HeaderMap::new();
    if let Some(id) = event {
        headers.insert("event-id", HeaderValue::from_str(id).unwrap());
    }
    let body = if data {
        format!(r#"{{"code":{code},"data":{{"bmsChrgSts":1}}}}"#)
    } else {
        format!(r#"{{"code":{code}}}"#)
    };
    crate::command::Reply {
        status: StatusCode::OK,
        headers,
        body: Zeroizing::new(body.into_bytes()),
    }
}
async fn authenticated() -> SaicClient {
    let mut c = client();
    c.login_with(now(), |_| async { success() }).await.unwrap();
    c
}

#[tokio::test(start_paused = true)]
async fn command_success_and_continuation_only_use_the_command_endpoint() {
    for command in [
        command(),
        crate::Command::Vehicle {
            request_type: "0".into(),
            parameters: vec![],
        },
    ] {
        for pending in [false, true] {
            let mut client = authenticated().await;
            let mut calls = 0;
            let outcome = client
                .command_with(
                    "SYNTHETICVIN00001",
                    &command,
                    Default::default(),
                    std::future::pending(),
                    |request| {
                        calls += 1;
                        assert_eq!(request.method(), "POST");
                        assert_eq!(
                            request.url().path(),
                            format!("/api.app/v1{}", command.path())
                        );
                        assert_eq!(
                            request.headers()["event-id"],
                            if calls == 1 { "0" } else { "synthetic-event" }
                        );
                        assert!(request.headers()["blade-auth"].is_sensitive());
                        assert!(!request.headers().contains_key("authorization"));
                        let result = if pending && calls == 1 {
                            reply(1, Some("synthetic-event"), false)
                        } else {
                            reply(0, None, true)
                        };
                        async { Ok(result) }
                    },
                )
                .await
                .unwrap();
            assert!(matches!(outcome.state, crate::CommandState::Completed));
            assert_eq!(calls, if pending { 2 } else { 1 });
            assert_eq!(
                outcome.event_id.as_deref(),
                pending.then_some("synthetic-event")
            );
            assert_eq!(outcome.saic_code, Some(0));
            assert!(!format!("{outcome:?}").contains("synthetic-event"));
        }
    }
}

#[tokio::test(start_paused = true)]
async fn rejects_and_invalidates_without_replaying_or_refreshing() {
    for code in [2, 3, 7, 401, 403, 99] {
        let mut client = authenticated().await;
        let mut calls = 0;
        let out = client
            .command_with(
                "test-vin",
                &command(),
                Default::default(),
                std::future::pending(),
                |_| {
                    calls += 1;
                    async move { Ok(reply(code, None, false)) }
                },
            )
            .await
            .unwrap();
        assert_eq!(calls, 1);
        assert_eq!(out.saic_code, Some(code));
        if matches!(code, 401 | 403) {
            assert!(matches!(out.state, crate::CommandState::InvalidSession));
            assert!(!client.is_logged_in());
        } else {
            assert!(matches!(out.state, crate::CommandState::Rejected));
        }
    }
}

#[tokio::test(start_paused = true)]
async fn ambiguous_failures_never_replay_and_keep_event_context() {
    for after_pending in [false, true] {
        for kind in 0..4 {
            let mut client = authenticated().await;
            let mut calls = 0;
            let out = client
                .command_with(
                    "test-vin",
                    &command(),
                    Default::default(),
                    std::future::pending(),
                    |_| {
                        calls += 1;
                        let result = if after_pending && calls == 1 {
                            Ok(reply(1, Some("event-secret"), false))
                        } else {
                            match kind {
                                0 => Err(Error::Transport),
                                1 => Err(Error::Timeout),
                                2 => {
                                    let mut r = reply(0, None, true);
                                    r.body = Zeroizing::new(b"malformed".to_vec());
                                    Ok(r)
                                }
                                _ => Ok(reply(0, Some("changed-event"), false)),
                            }
                        };
                        async { result }
                    },
                )
                .await
                .unwrap();
            assert!(matches!(out.state, crate::CommandState::Unknown(_)));
            if after_pending {
                assert_eq!(out.event_id.as_deref(), Some("event-secret"));
                assert_eq!(out.saic_code, Some(1));
            }
            assert_eq!(
                calls,
                if after_pending {
                    2
                } else if kind == 3 {
                    30
                } else {
                    1
                }
            );
        }
    }
}

#[tokio::test(start_paused = true)]
async fn per_request_timeout_overall_deadline_and_cancellation_are_bounded() {
    use crate::{CommandOptions, CommandState, UnknownReason};
    for cancel_at in [None, Some(Duration::from_secs(2))] {
        let mut client = authenticated().await;
        let mut calls = 0;
        let cancellation = async {
            match cancel_at {
                Some(delay) => tokio::time::sleep(delay).await,
                None => std::future::pending().await,
            }
        };
        let out = client
            .command_with(
                "test-vin",
                &command(),
                CommandOptions::default(),
                cancellation,
                |_| {
                    calls += 1;
                    std::future::pending()
                },
            )
            .await
            .unwrap();
        assert_eq!(calls, 1);
        assert!(matches!(
            out.state,
            CommandState::Unknown(UnknownReason::Timeout | UnknownReason::Cancelled)
        ));
    }
    for cancel in [false, true] {
        let mut client = authenticated().await;
        let options = CommandOptions {
            deadline: Duration::from_secs(3),
            ..Default::default()
        };
        let mut calls = 0;
        let out = client
            .command_with(
                "test-vin",
                &command(),
                options,
                async {
                    if cancel {
                        tokio::time::sleep(Duration::from_millis(1500)).await;
                    } else {
                        std::future::pending().await
                    }
                },
                |_| {
                    calls += 1;
                    async { Ok(reply(4, Some("event-secret"), false)) }
                },
            )
            .await
            .unwrap();
        assert_eq!(calls, if cancel { 2 } else { 3 });
        assert_eq!(out.event_id.as_deref(), Some("event-secret"));
        assert_eq!(out.saic_code, Some(4));
        assert!(matches!(out.state, CommandState::Unknown(_)));
    }
}

#[tokio::test]
async fn dropping_command_future_stops_exchange_without_relogin() {
    let mut client = authenticated().await;
    let mut calls = 0;
    let cmd = command();
    let future = client.command_with(
        "vin",
        &cmd,
        Default::default(),
        std::future::pending(),
        |_| {
            calls += 1;
            std::future::pending()
        },
    );
    assert!(tokio::time::timeout(Duration::from_millis(1), future)
        .await
        .is_err());
    assert_eq!(calls, 1);
    assert!(client.is_logged_in());
}

#[tokio::test]
async fn malformed_event_and_http_errors_preserve_safe_context() {
    for kind in 0..5 {
        let mut client = authenticated().await;
        let out = client
            .command_with(
                "vin",
                &command(),
                Default::default(),
                std::future::pending(),
                |_| async {
                    let mut r = reply(7, Some("private-event"), false);
                    match kind {
                        0 => {
                            r.status = StatusCode::UNAUTHORIZED;
                            r.body = Zeroizing::new(b"not json".to_vec());
                        }
                        1 => {
                            r.status = StatusCode::INTERNAL_SERVER_ERROR;
                        }
                        2 => {
                            r.headers.insert("event-id", HeaderValue::from_static("0"));
                        }
                        3 => {
                            r.body = Zeroizing::new(b"not json".to_vec());
                        }
                        _ => {}
                    }
                    Ok(r)
                },
            )
            .await
            .unwrap();
        match kind {
            0 => assert!(matches!(out.state, crate::CommandState::InvalidSession)),
            4 => assert!(matches!(out.state, crate::CommandState::Rejected)),
            _ => assert!(matches!(out.state, crate::CommandState::Unknown(_))),
        }
        if kind != 2 {
            assert_eq!(out.event_id.as_deref(), Some("private-event"));
        }
        assert!(!format!("{out:?}").contains("private-event"));
    }
}

#[test]
fn seat_payloads_cannot_change_two_seats_or_supply_extra_state() {
    use crate::{Command, ControlParameter};
    for ids in [
        vec![],
        vec![17, 18, 255],
        vec![17, 20, 255],
        vec![17],
        vec![255],
    ] {
        let cmd = Command::Vehicle {
            request_type: "5".into(),
            parameters: ids
                .into_iter()
                .map(|id| ControlParameter {
                    id,
                    value: "AAAAAA==".into(),
                })
                .collect(),
        };
        assert!(cmd.body("vin").is_err());
    }
    for id in [17, 18, 25, 26] {
        let cmd = Command::Vehicle {
            request_type: "5".into(),
            parameters: vec![
                ControlParameter {
                    id,
                    value: "AQ==".into(),
                },
                ControlParameter {
                    id: 255,
                    value: "AAAAAA==".into(),
                },
            ],
        };
        let body: serde_json::Value = serde_json::from_str(&cmd.body("vin").unwrap()).unwrap();
        assert_eq!(body["rvcParams"].as_array().unwrap().len(), 2);
        assert_eq!(body["rvcParams"][0]["paramId"], id);
    }
}

#[tokio::test]
async fn command_validation_and_unpolled_futures_never_send() {
    let mut client = client();
    let cmd = command();
    assert!(client
        .command_with(
            "vin",
            &cmd,
            Default::default(),
            std::future::pending(),
            |_| async { panic!("must not send") }
        )
        .await
        .is_err());
    client
        .login_with(now(), |_| async { success() })
        .await
        .unwrap();
    drop(client.execute_command("vin", &cmd, Default::default()));
    assert!(client.is_logged_in());
    let invalid = crate::CommandOptions {
        deadline: Duration::ZERO,
        ..Default::default()
    };
    assert!(client
        .command_with("vin", &cmd, invalid, std::future::pending(), |_| async {
            panic!("must not send")
        })
        .await
        .is_err());
    assert!(client
        .command_with(
            "",
            &cmd,
            Default::default(),
            std::future::pending(),
            |_| async { panic!("must not send") }
        )
        .await
        .is_err());
}

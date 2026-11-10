use super::*;
use crate::{
    test_support::{config, headers, vectors},
    Credentials,
};
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

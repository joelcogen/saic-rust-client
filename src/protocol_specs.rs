use super::*;
use crate::test_support::{config, headers, vectors};
use serde_json::json;

#[test]
fn email_and_phone_forms_match_python_including_escaping_and_password_digest() {
    let vectors = vectors();
    let timestamp = vectors["timestamp_ms"].as_u64().unwrap() as u128;
    for vector in vectors["requests"].as_array().unwrap() {
        let form = login_form(&config(vector), timestamp);
        assert_eq!(form.as_str(), vector["form"].as_str().unwrap());
        assert!(!form.contains(vector["password"].as_str().unwrap()));
    }
}

#[test]
fn encryption_and_signature_match_independent_python_vectors() {
    let vectors = vectors();
    let timestamp = vectors["timestamp_ms"].to_string();
    for vector in vectors["requests"].as_array().unwrap() {
        let (body, signature) = encrypt_request(
            LOGIN_PATH,
            &timestamp,
            "459771",
            FORM,
            vector["form"].as_str().unwrap(),
        );
        assert_eq!(body, vector["body"].as_str().unwrap());
        assert_eq!(signature, vector["headers"]["APP-VERIFICATION-STRING"]);
    }
}

#[test]
fn reads_plain_and_python_encrypted_login_responses() {
    let vectors = vectors();
    let response = &vectors["response"];
    for (headers, body) in [
        (HeaderMap::new(), response["plaintext"].as_str().unwrap()),
        (
            headers(&response["headers"]),
            response["body"].as_str().unwrap(),
        ),
    ] {
        let result = parse_login(StatusCode::OK, &headers, body.as_bytes()).unwrap();
        assert_eq!(result.token.as_str(), "synthetic-token");
        assert_eq!(result.expires_in, 3600);
    }
}

#[test]
fn preserves_http_and_saic_errors_without_printing_server_messages() {
    for (status, code) in [
        (200, 401),
        (200, 403),
        (401, 401),
        (403, 403),
        (500, 401),
        (200, 8),
        (500, 6),
    ] {
        let body = json!({"code": code, "message": "private-account@example.invalid"});
        let error = parse_login(
            StatusCode::from_u16(status).unwrap(),
            &HeaderMap::new(),
            body.to_string().as_bytes(),
        )
        .err()
        .unwrap();
        let failure = match &error {
            Error::Authentication(failure) => {
                assert!(code == 401 || code == 403);
                failure
            }
            Error::Rejected(failure) => {
                assert_eq!((status, code), (200, 8));
                failure
            }
            Error::Http(failure) => {
                assert_eq!(status, 500);
                failure
            }
            other => panic!("wrong error: {other}"),
        };
        assert_eq!(failure.http_status, status);
        assert_eq!(failure.saic_code, Some(code));
        assert_eq!(failure.message(), Some("private-account@example.invalid"));
        for formatted in [format!("{error}"), format!("{error:?}")] {
            assert!(!formatted.contains("private-account"));
        }
    }
}

#[test]
fn non_json_http_errors_keep_their_status_and_redirects_are_errors() {
    for status in [302, 401, 403, 429, 503] {
        let error = parse_login(
            StatusCode::from_u16(status).unwrap(),
            &HeaderMap::new(),
            b"<html>private upstream error</html>",
        )
        .err()
        .unwrap();
        let failure = match error {
            Error::Authentication(failure) | Error::Http(failure) => failure,
            other => panic!("wrong error: {other}"),
        };
        assert_eq!(failure.http_status, status);
        assert_eq!(failure.saic_code, None);
        assert_eq!(failure.message(), None);
    }
}

#[test]
fn rejects_missing_malformed_or_unusable_login_data() {
    let invalid = [
        json!({}),
        json!({"code": "0", "data": {"access_token":"token", "expires_in":3600}}),
        json!({"code":0}),
        json!({"code":0,"data":null}),
        json!({"code":0,"data":{}}),
        json!({"code":0,"data":{"access_token":"token"}}),
        json!({"code":0,"data":{"access_token":null,"expires_in":3600}}),
        json!({"code":0,"data":{"access_token":" ","expires_in":3600}}),
        json!({"code":0,"data":{"access_token":"token","expires_in":0}}),
        json!({"code":0,"data":{"access_token":"token","expires_in":-1}}),
        json!({"code":0,"data":{"access_token":"token","expires_in":"3600"}}),
        json!({"code":0,"data":{"access_token":"token","expires_in":1.5}}),
    ];
    for value in invalid {
        assert!(matches!(
            parse_login(
                StatusCode::OK,
                &HeaderMap::new(),
                value.to_string().as_bytes()
            ),
            Err(Error::InvalidResponse(_))
        ));
    }
    assert!(matches!(
        parse_login(StatusCode::OK, &HeaderMap::new(), b"not JSON"),
        Err(Error::InvalidResponse(_))
    ));
}

#[test]
fn rejects_incomplete_encryption_headers_and_invalid_ciphertext() {
    let valid_headers = headers(&vectors()["response"]["headers"]);
    for body in ["not hex", "00", "00000000000000000000000000000000", ""] {
        assert!(matches!(
            parse_login(StatusCode::OK, &valid_headers, body.as_bytes()),
            Err(Error::Decryption)
        ));
    }
    for value in [
        json!({"APP-CONTENT-ENCRYPTED":"1"}),
        json!({"APP-SEND-DATE":"1735689600123"}),
        json!({"ORIGINAL-CONTENT-TYPE":"application/json"}),
    ] {
        assert!(matches!(
            parse_login(StatusCode::OK, &headers(&value), b"{}"),
            Err(Error::Decryption)
        ));
    }
}

#[test]
fn authenticated_command_matches_independent_python_vector() {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../tests/fixtures/command.json")).unwrap();
    let (body, mac) = encrypt_authenticated(
        v["path"].as_str().unwrap(),
        v["timestamp"].as_str().unwrap(),
        v["tenant"].as_str().unwrap(),
        v["token"].as_str().unwrap(),
        "application/json",
        v["plaintext"].as_str().unwrap(),
    );
    assert_eq!(body, v["body"]);
    assert_eq!(mac, v["verification"]);
    let command = crate::Command::Charging {
        charging: 1,
        port_lock: 0,
        v2x: 0,
    };
    assert_eq!(
        command.body(v["vin"].as_str().unwrap()).unwrap(),
        v["plaintext"]
    );
}

#[test]
fn empty_get_body_signature_includes_query_and_matches_python() {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../tests/fixtures/read.json")).unwrap();
    let (body, signature) = super::encrypt_authenticated(
        v["path"].as_str().unwrap(),
        v["timestamp"].as_str().unwrap(),
        v["tenant"].as_str().unwrap(),
        v["token"].as_str().unwrap(),
        "application/json",
        "",
    );
    assert!(body.is_empty());
    assert_eq!(signature, v["verification"]);
}

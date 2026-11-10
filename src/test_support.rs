use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;

use crate::{ClientConfig, Credentials};

pub fn vectors() -> Value {
    serde_json::from_str(include_str!("../tests/fixtures/login.json")).unwrap()
}

pub fn config(vector: &Value) -> ClientConfig {
    let username = vector["username"].as_str().unwrap();
    let password = vector["password"].as_str().unwrap();
    let credentials = match vector["country_code"].as_str() {
        Some(code) => Credentials::phone(username, code, password),
        None => Credentials::email(username, password),
    };
    ClientConfig::mg_europe(credentials).with_base_url("https://example.invalid/api.app/v1/")
}

pub fn headers(value: &Value) -> HeaderMap {
    value
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| {
            (
                HeaderName::from_bytes(key.as_bytes()).unwrap(),
                HeaderValue::from_str(value.as_str().unwrap()).unwrap(),
            )
        })
        .collect()
}

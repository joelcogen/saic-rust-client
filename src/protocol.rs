//! SAIC login wire format, ported from the pinned Python client.
//! See docs/references.md and THIRD_PARTY_NOTICES.md.

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use reqwest::{header::HeaderMap, StatusCode};
use serde::Deserialize;
use sha1::Sha1;
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::{ApiFailure, ClientConfig, Error};

pub(crate) const FORM: &str = "application/x-www-form-urlencoded";
pub(crate) const LOGIN_PATH: &str = "/oauth/token";

pub(crate) fn login_form(config: &ClientConfig, timestamp_ms: u128) -> Zeroizing<String> {
    let password_hash = Zeroizing::new(hex::encode(Sha1::digest(
        config.credentials.password.as_bytes(),
    )));
    let device_id = format!(
        "simulator*********************************************{}###{}",
        timestamp_ms / 1000,
        config.device_app_id
    );
    let mut form = url::form_urlencoded::Serializer::new(String::new());
    form.extend_pairs([
        ("grant_type", "password"),
        ("username", config.credentials.username.as_str()),
        ("password", password_hash.as_str()),
        ("scope", "all"),
        ("deviceId", &device_id),
        ("deviceType", "0"),
        ("language", "EN"),
    ]);
    if let Some(country_code) = &config.credentials.country_code {
        form.extend_pairs([("loginType", "1"), ("countryCode", country_code)]);
    } else {
        form.append_pair("loginType", "2");
    }
    // Match Python/httpx's urllib encoding (not WHATWG's * / ~ choices).
    Zeroizing::new(form.finish().replace('*', "%2A").replace("%7E", "~"))
}

fn md5_hex(value: impl AsRef<[u8]>) -> String {
    hex::encode(Md5::digest(value))
}

/// The HMAC key is the ASCII hex digest, whereas AES uses the decoded digest.
pub(crate) fn encrypt_request(
    path: &str,
    timestamp: &str,
    tenant: &str,
    content_type: &str,
    plaintext: &str,
) -> (String, String) {
    encrypt_authenticated(path, timestamp, tenant, "", content_type, plaintext)
}

pub(crate) fn encrypt_authenticated(
    path: &str,
    timestamp: &str,
    tenant: &str,
    token: &str,
    content_type: &str,
    plaintext: &str,
) -> (String, String) {
    let prefix = format!("{path}{tenant}{token}app");
    let key = Md5::digest(format!("{}{timestamp}1{content_type}", md5_hex(&prefix)));
    let iv = Md5::digest(timestamp);
    let ciphertext = cbc::Encryptor::<aes::Aes128>::new(&key, &iv)
        .encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());
    let body = if plaintext.is_empty() {
        String::new()
    } else {
        hex::encode(ciphertext)
    };
    let hmac_key = md5_hex(format!("{}{timestamp}", hex::encode(key)));
    let mut mac = Hmac::<Sha256>::new_from_slice(hmac_key.as_bytes())
        .expect("HMAC accepts keys of any length");
    mac.update(format!("{prefix}{timestamp}1{content_type}{body}").as_bytes());
    (body, hex::encode(mac.finalize().into_bytes()))
}

pub(crate) fn decrypt_response(
    headers: &HeaderMap,
    body: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Error> {
    let timestamp = headers.get("app-send-date");
    let content_type = headers.get("original-content-type");
    match (timestamp, content_type) {
        (None, None)
            if headers
                .get("app-content-encrypted")
                .is_none_or(|v| v == "0") =>
        {
            Ok(Zeroizing::new(body.to_vec()))
        }
        (Some(timestamp), Some(content_type)) => {
            let timestamp = timestamp.to_str().map_err(|_| Error::Decryption)?;
            let content_type = content_type.to_str().map_err(|_| Error::Decryption)?;
            if timestamp.is_empty() || content_type.is_empty() {
                return Err(Error::Decryption);
            }
            let body = std::str::from_utf8(body).map_err(|_| Error::Decryption)?;
            let ciphertext = hex::decode(body.trim()).map_err(|_| Error::Decryption)?;
            let key = Md5::digest(format!("{timestamp}1{content_type}"));
            let iv = Md5::digest(timestamp);
            cbc::Decryptor::<aes::Aes128>::new(&key, &iv)
                .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
                .map(Zeroizing::new)
                .map_err(|_| Error::Decryption)
        }
        _ => Err(Error::Decryption),
    }
}

#[derive(Deserialize)]
struct Envelope {
    code: i64,
    message: Option<String>,
    data: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct LoginData {
    access_token: String,
    expires_in: u64,
}

pub(crate) struct LoginResult {
    pub token: Zeroizing<String>,
    pub expires_in: u64,
}

pub(crate) fn parse_login(
    status: StatusCode,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<LoginResult, Error> {
    if !status.is_success() {
        // Like the Python client, HTTP error bodies aren't decrypted. HTML and
        // other non-JSON bodies still report their HTTP status.
        let envelope = serde_json::from_slice::<Envelope>(body).ok();
        let failure = ApiFailure {
            http_status: status.as_u16(),
            saic_code: envelope.as_ref().map(|e| e.code),
            message: envelope.and_then(|e| e.message),
        };
        return Err(
            if matches!(status.as_u16(), 401 | 403) || matches!(failure.saic_code, Some(401 | 403))
            {
                Error::Authentication(failure)
            } else {
                Error::Http(failure)
            },
        );
    }
    let plaintext = decrypt_response(headers, body)?;
    let envelope: Envelope = serde_json::from_slice(&plaintext)
        .map_err(|_| Error::InvalidResponse("expected a SAIC response envelope"))?;
    if envelope.code != 0 {
        let failure = ApiFailure {
            http_status: status.as_u16(),
            saic_code: Some(envelope.code),
            message: envelope.message,
        };
        return Err(if matches!(envelope.code, 401 | 403) {
            Error::Authentication(failure)
        } else {
            Error::Rejected(failure)
        });
    }
    let data: LoginData = serde_json::from_value(
        envelope
            .data
            .ok_or(Error::InvalidResponse("missing data"))?,
    )
    .map_err(|_| Error::InvalidResponse("missing or invalid token or expiry"))?;
    let token = Zeroizing::new(data.access_token);
    if token.trim().is_empty() || data.expires_in == 0 {
        return Err(Error::InvalidResponse("empty token or zero expiry"));
    }
    Ok(LoginResult {
        token,
        expires_in: data.expires_in,
    })
}

#[cfg(test)]
#[path = "protocol_specs.rs"]
mod specs;

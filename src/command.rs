//! Low-level command envelopes. Values are protocol values, not model capabilities.
use crate::{protocol, Error};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fmt, time::Duration};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlParameter {
    #[serde(rename = "paramId")]
    pub id: u8,
    /// Protocol base64 encoded bytes.
    #[serde(rename = "paramValue")]
    pub value: String,
}

/// Explicit caller-supplied payload. No existing values are fetched or filled in.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Command {
    Vehicle {
        #[serde(rename = "rvcReqType")]
        request_type: String,
        #[serde(rename = "rvcParams")]
        parameters: Vec<ControlParameter>,
    },
    Charging {
        #[serde(rename = "chrgCtrlReq")]
        charging: u8,
        #[serde(rename = "tboxEleccLckCtrlReq")]
        port_lock: u8,
        #[serde(rename = "tboxV2XReq")]
        v2x: u8,
    },
}

impl Command {
    pub(crate) fn path(&self) -> &'static str {
        match self {
            Self::Vehicle { .. } => "/vehicle/control",
            Self::Charging { .. } => "/vehicle/charging/control",
        }
    }
    pub(crate) fn body(&self, vin: &str) -> Result<String, Error> {
        if vin.trim().is_empty() {
            return Err(Error::Configuration("VIN must not be empty"));
        }
        if let Self::Vehicle {
            request_type,
            parameters,
        } = self
        {
            // Refresh is not a control supported by this transport. Seat commands
            // must carry exactly one seat and the protocol terminator.
            if request_type == "18" {
                return Err(Error::Configuration("refresh is not a command"));
            }
            let seats = parameters
                .iter()
                .filter(|p| matches!(p.id, 17 | 18 | 25 | 26))
                .count();
            if (request_type == "5" || seats > 0)
                && (seats != 1
                    || parameters.len() != 2
                    || !parameters
                        .iter()
                        .any(|p| p.id == 255 && p.value == "AAAAAA=="))
            {
                return Err(Error::Configuration(
                    "seat commands require one seat and the terminator",
                ));
            }
        }
        let mut body = serde_json::to_value(self).expect("command serializes");
        let map = body.as_object_mut().expect("command is an object");
        map.remove("kind");
        // The reference lock request has null parameters, not an empty array.
        if matches!(self, Self::Vehicle { request_type, parameters } if request_type == "1" && parameters.is_empty())
        {
            map.insert("rvcParams".into(), Value::Null);
        }
        map.insert(
            "vin".into(),
            Value::String(hex::encode(Sha256::digest(vin))),
        );
        Ok(body.to_string())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CommandOptions {
    pub deadline: Duration,
    pub request_timeout: Duration,
    pub continuation_delay: Duration,
}
impl Default for CommandOptions {
    fn default() -> Self {
        Self {
            deadline: Duration::from_secs(30),
            request_timeout: Duration::from_secs(10),
            continuation_delay: Duration::from_secs(1),
        }
    }
}
#[derive(Debug)]
pub enum UnknownReason {
    Timeout,
    Cancelled,
    Failure(Error),
}
#[derive(Debug)]
pub enum CommandState {
    /// The API completed the command; this does not prove physical position.
    Completed,
    Rejected,
    InvalidSession,
    Unknown(UnknownReason),
}

/// Identifiers and server data are available explicitly, but omitted from Debug.
pub struct CommandOutcome {
    pub state: CommandState,
    pub event_id: Option<String>,
    pub saic_code: Option<i64>,
    pub http_status: Option<u16>,
    pub data: Option<Value>,
    pub message: Option<String>,
}
impl Default for CommandOutcome {
    fn default() -> Self {
        Self {
            state: CommandState::Unknown(UnknownReason::Cancelled),
            event_id: None,
            saic_code: None,
            http_status: None,
            data: None,
            message: None,
        }
    }
}
impl fmt::Debug for CommandOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandOutcome")
            .field("state", &self.state)
            .field("saic_code", &self.saic_code)
            .field("http_status", &self.http_status)
            .finish_non_exhaustive()
    }
}
pub(crate) struct Reply {
    pub status: reqwest::StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub body: zeroize::Zeroizing<Vec<u8>>,
}
#[derive(Deserialize)]
struct Envelope {
    code: i64,
    message: Option<String>,
    #[serde(default)]
    data: Option<Value>,
}

/// Update context before parsing so malformed replies retain event IDs and status.
pub(crate) fn interpret(reply: Reply, outcome: &mut CommandOutcome) -> Result<bool, Error> {
    outcome.http_status = Some(reply.status.as_u16());
    if let Some(id) = reply.headers.get("event-id") {
        let id = id
            .to_str()
            .map_err(|_| Error::InvalidResponse("invalid event ID"))?;
        if id.trim().is_empty() || id == "0" {
            return Err(Error::InvalidResponse("invalid event ID"));
        }
        if outcome.event_id.as_ref().is_some_and(|old| old != id) {
            return Err(Error::InvalidResponse("event ID changed"));
        }
        outcome.event_id = Some(id.to_owned());
    }
    let plaintext = protocol::decrypt_response(&reply.headers, &reply.body);
    let envelope = plaintext.and_then(|p| {
        serde_json::from_slice::<Envelope>(&p)
            .map_err(|_| Error::InvalidResponse("expected a command envelope"))
    });
    if matches!(reply.status.as_u16(), 401 | 403) {
        if let Ok(e) = envelope {
            outcome.saic_code = Some(e.code);
            outcome.message = e.message;
            outcome.data = e.data;
        }
        outcome.state = CommandState::InvalidSession;
        return Ok(false);
    }
    let e = envelope?;
    outcome.saic_code = Some(e.code);
    outcome.message = e.message;
    outcome.data = e.data.clone();
    if matches!(e.code, 401 | 403) {
        outcome.state = CommandState::InvalidSession;
        return Ok(false);
    }
    if !reply.status.is_success() {
        return Err(Error::Http(crate::ApiFailure {
            http_status: reply.status.as_u16(),
            saic_code: Some(e.code),
            message: None,
        }));
    }
    if matches!(e.code, 2 | 3 | 7) || (e.code != 0 && outcome.event_id.is_none()) {
        outcome.state = CommandState::Rejected;
        return Ok(false);
    }
    if e.code != 0 || (e.data.is_none() && outcome.event_id.is_some()) {
        return Ok(true);
    }
    if !e.data.as_ref().is_some_and(Value::is_object) {
        return Err(Error::InvalidResponse("missing command result object"));
    }
    outcome.data = e.data;
    outcome.state = CommandState::Completed;
    Ok(false)
}

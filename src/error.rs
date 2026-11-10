use std::fmt;

/// Server failure details. The message is available explicitly, but omitted from
/// Display and Debug because a server can echo account data in it.
pub struct ApiFailure {
    pub http_status: u16,
    pub saic_code: Option<i64>,
    pub(crate) message: Option<String>,
}

impl ApiFailure {
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl fmt::Debug for ApiFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiFailure")
            .field("http_status", &self.http_status)
            .field("saic_code", &self.saic_code)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for ApiFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTTP {}, SAIC code {:?}",
            self.http_status, self.saic_code
        )
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("invalid configuration: {0}")]
    Configuration(&'static str),
    #[error("authentication failed ({0})")]
    Authentication(ApiFailure),
    #[error("SAIC rejected the request ({0})")]
    Rejected(ApiFailure),
    #[error("HTTP request failed ({0})")]
    Http(ApiFailure),
    #[error("request timed out")]
    Timeout,
    #[error("HTTP transport failed")]
    Transport,
    #[error("invalid login response: {0}")]
    InvalidResponse(&'static str),
    #[error("could not decrypt the response")]
    Decryption,
    #[error("system clock is before the Unix epoch")]
    Clock,
}

impl Error {
    pub(crate) fn transport(error: reqwest::Error) -> Self {
        // reqwest errors can contain URLs. Keep default errors free of account data.
        if error.is_timeout() {
            Self::Timeout
        } else {
            Self::Transport
        }
    }
}

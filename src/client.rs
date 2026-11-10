use std::{
    fmt,
    future::Future,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use reqwest::{header::HeaderValue, redirect::Policy, Client, Request, Url};
use zeroize::Zeroizing;

use crate::{protocol, ClientConfig, Error};

struct Session {
    token: Zeroizing<String>,
    expires_at: Instant,
}

/// A reusable HTTP client and its private login session.
///
/// `login` takes exclusive access so two logins can't race on this instance.
/// Construct it inside a Tokio runtime before making async requests.
pub struct SaicClient {
    config: ClientConfig,
    http: Client,
    login_url: Url,
    session: Option<Session>,
}

impl SaicClient {
    /// Validate configuration and prepare HTTP connections without sending a request.
    pub fn new(config: ClientConfig) -> Result<Self, Error> {
        let login_url = config.login_url()?;
        let http = Client::builder()
            .https_only(true)
            .redirect(Policy::none())
            .retry(reqwest::retry::never())
            .timeout(config.timeout)
            .build()
            .map_err(Error::transport)?;
        Ok(Self {
            config,
            http,
            login_url,
            session: None,
        })
    }

    /// Log in once. No automatic retries, refreshes, or vehicle requests.
    ///
    /// Once polled, this clears the previous local session before sending. Failure
    /// or cancellation leaves the client logged out. An unpolled future does nothing.
    pub async fn login(&mut self) -> Result<(), Error> {
        let http = self.http.clone();
        self.login_with(SystemTime::now(), |request| async move {
            let mut response = http.execute(request).await.map_err(Error::transport)?;
            let status = response.status();
            let headers = response.headers().clone();
            let mut body = Zeroizing::new(Vec::new());
            while let Some(chunk) = response.chunk().await.map_err(Error::transport)? {
                if body.len() + chunk.len() > 1024 * 1024 {
                    return Err(Error::InvalidResponse("response exceeds 1 MiB"));
                }
                body.extend_from_slice(&chunk);
            }
            protocol::parse_login(status, &headers, &body)
        })
        .await
    }

    /// True while the locally stored session has not expired. SAIC can invalidate
    /// a session sooner; this method doesn't contact the server to check.
    pub fn is_logged_in(&self) -> bool {
        self.session_remaining().is_some()
    }

    /// Remaining local lifetime, measured with a monotonic clock.
    pub fn session_remaining(&self) -> Option<Duration> {
        self.remaining_at(Instant::now())
    }

    /// Forget the local session. This doesn't revoke the token on SAIC's server.
    pub fn logout(&mut self) {
        self.session = None;
    }

    fn remaining_at(&self, now: Instant) -> Option<Duration> {
        let session = self.session.as_ref()?;
        let remaining = session.expires_at.checked_duration_since(now)?;
        (!session.token.is_empty() && !remaining.is_zero()).then_some(remaining)
    }

    fn login_request(&self, now: SystemTime) -> Result<Request, Error> {
        let timestamp = now.duration_since(UNIX_EPOCH).map_err(|_| Error::Clock)?;
        let form = protocol::login_form(&self.config, timestamp.as_millis());
        let timestamp = timestamp.as_millis().to_string();
        let (body, verification) = protocol::encrypt_request(
            protocol::LOGIN_PATH,
            &timestamp,
            &self.config.tenant_id,
            protocol::FORM,
            &form,
        );
        // This is the upstream app identity, not an account credential.
        let mut authorization = HeaderValue::from_static("Basic c3dvcmQ6c3dvcmRfc2VjcmV0");
        authorization.set_sensitive(true);
        self.http
            .post(self.login_url.clone())
            .header("authorization", authorization)
            .header("user-agent", "Europe/2.1.0 (iPad; iOS 18.5; Scale/2.00)")
            .header("content-type", format!("{};charset=utf-8", protocol::FORM))
            .header("accept", "application/json")
            .header("accept-encoding", "gzip")
            .header("region", &self.config.region)
            .header("tenant-id", &self.config.tenant_id)
            .header("user-type", "app")
            .header("app-language-type", "en")
            .header("app-send-date", timestamp)
            .header("app-content-encrypted", "1")
            .header("app-verification-string", verification)
            .header("original-content-type", protocol::FORM)
            .body(body)
            .build()
            .map_err(Error::transport)
    }

    // The same login flow is exercised offline by supplying a fake exchange.
    // This remains private; callers use the real transport through login().
    async fn login_with<F, Fut>(&mut self, now: SystemTime, exchange: F) -> Result<(), Error>
    where
        F: FnOnce(Request) -> Fut,
        Fut: Future<Output = Result<protocol::LoginResult, Error>>,
    {
        self.logout();
        let started = Instant::now();
        let response = exchange(self.login_request(now)?).await?;
        // Count request time against the lifetime instead of extending the session
        // by however long the response took to arrive.
        let expires_at = started
            .checked_add(Duration::from_secs(response.expires_in))
            .filter(|deadline| *deadline > Instant::now())
            .ok_or(Error::InvalidResponse(
                "expiry is out of range or already elapsed",
            ))?;
        self.session = Some(Session {
            token: response.token,
            expires_at,
        });
        Ok(())
    }
}

impl fmt::Debug for SaicClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SaicClient")
            .field("is_logged_in", &self.is_logged_in())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[path = "client_specs.rs"]
mod specs;

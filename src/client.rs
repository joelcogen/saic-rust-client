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

    /// Submit once and continue only the same server event. No status refresh or relogin.
    /// Dropping this future stops waiting; the server may still execute the command.
    pub async fn execute_command(
        &mut self,
        vin: &str,
        command: &crate::Command,
        options: crate::CommandOptions,
    ) -> Result<crate::CommandOutcome, Error> {
        self.execute_command_until(vin, command, options, std::future::pending())
            .await
    }

    /// Like execute_command, but cancellation returns an unknown outcome with the
    /// last event ID and code. Cancellation does not cancel execution at SAIC.
    pub async fn execute_command_until<C: Future<Output = ()>>(
        &mut self,
        vin: &str,
        command: &crate::Command,
        options: crate::CommandOptions,
        cancel: C,
    ) -> Result<crate::CommandOutcome, Error> {
        let operation = crate::reads::Operation {
            path: command.path().into(),
            body: Zeroizing::new(command.body(vin)?),
            method: reqwest::Method::POST,
            events: command.events(),
            allow_empty_result: !command.events(),
        };
        self.execute_operation(&operation, options, cancel).await
    }

    async fn execute_operation<C: Future<Output = ()>>(
        &mut self,
        operation: &crate::reads::Operation,
        options: crate::CommandOptions,
        cancel: C,
    ) -> Result<crate::CommandOutcome, Error> {
        let http = self.http.clone();
        self.operation_with(operation, options, cancel, |request| {
            let http = http.clone();
            async move {
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
                Ok(crate::command::Reply {
                    status,
                    headers,
                    body,
                })
            }
        })
        .await
    }

    pub(crate) async fn read<T: serde::de::DeserializeOwned>(
        &mut self,
        kind: crate::reads::Read,
        vin: &str,
        options: crate::CommandOptions,
    ) -> Result<T, Error> {
        let operation = kind.operation(vin)?;
        let outcome = self
            .execute_operation(&operation, options, std::future::pending())
            .await?;
        Self::read_result(outcome)
    }

    fn read_result<T: serde::de::DeserializeOwned>(
        outcome: crate::CommandOutcome,
    ) -> Result<T, Error> {
        use crate::{CommandState, UnknownReason};
        let failure = crate::ApiFailure {
            http_status: outcome.http_status.unwrap_or(0),
            saic_code: outcome.saic_code,
            message: outcome.message,
        };
        match outcome.state {
            CommandState::Completed => serde_json::from_value(
                outcome
                    .data
                    .ok_or(Error::InvalidResponse("missing read result"))?,
            )
            .map_err(|_| Error::InvalidResponse("invalid typed read result")),
            CommandState::Rejected => Err(Error::Rejected(failure)),
            CommandState::InvalidSession => Err(Error::Authentication(failure)),
            CommandState::Unknown(UnknownReason::Timeout) => Err(Error::Timeout),
            CommandState::Unknown(UnknownReason::Failure(error)) => Err(error),
            CommandState::Unknown(UnknownReason::Cancelled) => {
                Err(Error::InvalidResponse("read cancelled"))
            }
        }
    }

    #[cfg(test)]
    async fn command_with<F, Fut, C>(
        &mut self,
        vin: &str,
        command: &crate::Command,
        options: crate::CommandOptions,
        cancel: C,
        exchange: F,
    ) -> Result<crate::CommandOutcome, Error>
    where
        F: FnMut(Request) -> Fut,
        Fut: Future<Output = Result<crate::command::Reply, Error>>,
        C: Future<Output = ()>,
    {
        let operation = crate::reads::Operation {
            path: command.path().into(),
            body: Zeroizing::new(command.body(vin)?),
            method: reqwest::Method::POST,
            events: command.events(),
            allow_empty_result: !command.events(),
        };
        self.operation_with(&operation, options, cancel, exchange)
            .await
    }

    async fn operation_with<F, Fut, C>(
        &mut self,
        operation: &crate::reads::Operation,
        options: crate::CommandOptions,
        cancel: C,
        mut exchange: F,
    ) -> Result<crate::CommandOutcome, Error>
    where
        F: FnMut(Request) -> Fut,
        Fut: Future<Output = Result<crate::command::Reply, Error>>,
        C: Future<Output = ()>,
    {
        use crate::{CommandOutcome, CommandState, UnknownReason};
        if !self.is_logged_in() {
            return Err(Error::Configuration("login required"));
        }
        if options.deadline.is_zero()
            || options.request_timeout.is_zero()
            || options.continuation_delay.is_zero()
        {
            return Err(Error::Configuration("command time limits must be positive"));
        }
        let deadline = tokio::time::Instant::now()
            .checked_add(options.deadline)
            .ok_or(Error::Configuration("command deadline out of range"))?;
        let mut outcome = CommandOutcome::default();
        tokio::pin!(cancel);
        loop {
            let request = match self
                .operation_request(operation, outcome.event_id.as_deref().unwrap_or("0"))
            {
                Ok(request) => request,
                Err(error) if outcome.event_id.is_none() => return Err(error),
                Err(error) => {
                    outcome.state = CommandState::Unknown(UnknownReason::Failure(error));
                    break;
                }
            };
            let result = tokio::select! {
                biased;
                _ = &mut cancel => { outcome.state = CommandState::Unknown(UnknownReason::Cancelled); break; }
                _ = tokio::time::sleep_until(deadline) => { outcome.state = CommandState::Unknown(UnknownReason::Timeout); break; }
                result = tokio::time::timeout(options.request_timeout, exchange(request)) => result.unwrap_or(Err(Error::Timeout)),
            };
            let pending = result.and_then(|reply| {
                crate::command::interpret(reply, &mut outcome, operation.allow_empty_result)
            });
            match pending {
                Ok(false) => {
                    if matches!(outcome.state, CommandState::InvalidSession) {
                        self.logout();
                    }
                    break;
                }
                Err(error) => {
                    outcome.state = CommandState::Unknown(match error {
                        Error::Timeout => UnknownReason::Timeout,
                        other => UnknownReason::Failure(other),
                    });
                    break;
                }
                Ok(true) if !operation.events => {
                    outcome.state = CommandState::Unknown(UnknownReason::Failure(
                        Error::InvalidResponse("unexpected pending result for single operation"),
                    ));
                    break;
                }
                Ok(true) => {}
            }
            tokio::select! {
                biased;
                _ = &mut cancel => { outcome.state = CommandState::Unknown(UnknownReason::Cancelled); break; }
                _ = tokio::time::sleep_until(deadline) => { outcome.state = CommandState::Unknown(UnknownReason::Timeout); break; }
                _ = tokio::time::sleep(options.continuation_delay) => {}
            }
        }
        Ok(outcome)
    }

    fn operation_request(
        &self,
        operation: &crate::reads::Operation,
        event: &str,
    ) -> Result<Request, Error> {
        let path = &operation.path;
        let plaintext = &operation.body;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Clock)?
            .as_millis()
            .to_string();
        let token = &self.session.as_ref().expect("session checked").token;
        let (body, verification) = protocol::encrypt_authenticated(
            path,
            &timestamp,
            &self.config.tenant_id,
            token,
            "application/json",
            plaintext,
        );
        let url = self
            .login_url
            .join(&format!("../{}", path.trim_start_matches('/')))
            .map_err(|_| Error::Configuration("invalid command URL"))?;
        let mut request = self.post(url).build().map_err(Error::transport)?;
        *request.method_mut() = operation.method.clone();
        let headers = request.headers_mut();
        for (name, value) in [
            ("blade-auth", token.as_str()),
            ("event-id", event),
            ("app-verification-string", &verification),
            ("app-send-date", &timestamp),
            ("content-type", "application/json;charset=utf-8"),
            ("original-content-type", "application/json"),
        ] {
            let mut value = HeaderValue::from_str(value)
                .map_err(|_| Error::InvalidResponse("invalid command header"))?;
            value.set_sensitive(true);
            headers.insert(
                reqwest::header::HeaderName::from_bytes(name.as_bytes()).expect("static header"),
                value,
            );
        }
        if !operation.events {
            headers.remove("event-id");
        }
        if !body.is_empty() {
            *request.body_mut() = Some(body.into());
        }
        Ok(request)
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
        self.post(self.login_url.clone())
            .header("authorization", authorization)
            .header("content-type", format!("{};charset=utf-8", protocol::FORM))
            .header("app-send-date", timestamp)
            .header("app-verification-string", verification)
            .header("original-content-type", protocol::FORM)
            .body(body)
            .build()
            .map_err(Error::transport)
    }

    fn post(&self, url: Url) -> reqwest::RequestBuilder {
        self.http
            .post(url)
            .header("user-agent", "Europe/2.1.0 (iPad; iOS 18.5; Scale/2.00)")
            .header("accept", "application/json")
            .header("accept-encoding", "gzip")
            .header("region", &self.config.region)
            .header("tenant-id", &self.config.tenant_id)
            .header("user-type", "app")
            .header("app-language-type", "en")
            .header("app-content-encrypted", "1")
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

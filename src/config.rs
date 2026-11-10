use std::{fmt, time::Duration};

use reqwest::header::HeaderValue;
use url::Url;
use zeroize::Zeroizing;

use crate::Error;

/// Account credentials. Debug output hides both the username and password.
pub struct Credentials {
    pub(crate) username: Zeroizing<String>,
    pub(crate) password: Zeroizing<String>,
    pub(crate) country_code: Option<String>,
}

impl Credentials {
    pub fn email(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: Zeroizing::new(username.into()),
            password: Zeroizing::new(password.into()),
            country_code: None,
        }
    }

    /// Phone login requires a country code, in the form expected by SAIC.
    pub fn phone(
        number: impl Into<String>,
        country_code: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            username: Zeroizing::new(number.into()),
            password: Zeroizing::new(password.into()),
            country_code: Some(country_code.into()),
        }
    }
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Credentials { .. }")
    }
}

/// Connection settings. Validation happens in [`crate::SaicClient::new`].
pub struct ClientConfig {
    pub(crate) credentials: Credentials,
    pub(crate) base_url: String,
    pub(crate) region: String,
    pub(crate) tenant_id: String,
    pub(crate) device_app_id: String,
    pub(crate) timeout: Duration,
}

impl ClientConfig {
    pub fn mg_europe(credentials: Credentials) -> Self {
        Self {
            credentials,
            base_url: "https://gateway-mg-eu.soimt.com/api.app/v1/".into(),
            region: "eu".into(),
            tenant_id: "459771".into(),
            device_app_id: "com.saicmotor.europecar".into(),
            timeout: Duration::from_secs(10),
        }
    }

    /// Override the HTTPS API root. A missing trailing slash is added.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Override the profile together. Other profiles haven't been live-tested.
    pub fn with_api_identity(
        mut self,
        region: impl Into<String>,
        tenant_id: impl Into<String>,
        device_app_id: impl Into<String>,
    ) -> Self {
        self.region = region.into();
        self.tenant_id = tenant_id.into();
        self.device_app_id = device_app_id.into();
        self
    }

    /// Total HTTP request timeout, including reading the response body.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub(crate) fn login_url(&self) -> Result<Url, Error> {
        let mut url =
            Url::parse(&self.base_url).map_err(|_| Error::Configuration("invalid API base URL"))?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(Error::Configuration(
                "API base URL must be HTTPS without credentials, query, or fragment",
            ));
        }
        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }
        if self.credentials.username.trim().is_empty()
            || self.credentials.password.is_empty()
            || self
                .credentials
                .country_code
                .as_ref()
                .is_some_and(|code| code.trim().is_empty())
        {
            return Err(Error::Configuration(
                "account credentials must not be empty",
            ));
        }
        for value in [&self.region, &self.tenant_id, &self.device_app_id] {
            if value.trim().is_empty() || HeaderValue::from_str(value).is_err() {
                return Err(Error::Configuration("invalid API identity"));
            }
        }
        if self.timeout.is_zero() {
            return Err(Error::Configuration("timeout must be greater than zero"));
        }
        url.join("oauth/token")
            .map_err(|_| Error::Configuration("invalid login URL"))
    }
}

impl fmt::Debug for ClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Even an invalid URL could contain credentials. Don't print it.
        f.debug_struct("ClientConfig")
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

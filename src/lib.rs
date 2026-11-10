//! Async client for the SAIC MG iSMART API.
//!
//! Construction makes no network requests. Login is explicit; the client doesn't
//! retry, refresh tokens, poll vehicles, or send vehicle commands automatically.
//!
//! ```no_run
//! use saic_client::{ClientConfig, Credentials, SaicClient};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let credentials = Credentials::email(
//!     std::env::var("SAIC_USERNAME")?,
//!     std::env::var("SAIC_PASSWORD")?,
//! );
//! let mut client = SaicClient::new(ClientConfig::mg_europe(credentials))?;
//! client.login().await?;
//! assert!(client.is_logged_in());
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

mod client;
mod config;
mod error;
mod protocol;

pub use client::SaicClient;
pub use config::{ClientConfig, Credentials};
pub use error::{ApiFailure, Error};

#[cfg(test)]
mod test_support;

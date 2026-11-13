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
mod climate;
mod command;
mod config;
mod error;
mod find_car;
mod locks;
pub mod models;
mod protocol;
mod reads;

pub use client::SaicClient;
pub use climate::TemperatureProfile;
pub use command::{
    Command, CommandOptions, CommandOutcome, CommandState, ControlParameter, UnknownReason,
};
pub use config::{ClientConfig, Credentials};
pub use error::{ApiFailure, Error};
pub use find_car::FindCarMode;
pub use locks::{LockAction, TailgateAction, VehicleLockId};

#[cfg(test)]
mod test_support;

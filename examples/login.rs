//! Manual production check. Never run by cargo test.
use saic_client::{ClientConfig, Credentials, SaicClient};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args != ["--production"] {
        eprintln!("Usage: cargo run --example login -- --production");
        eprintln!("Authenticates once using SAIC_USERNAME and SAIC_PASSWORD. No vehicle calls.");
        std::process::exit(2);
    }
    let username = std::env::var("SAIC_USERNAME").map_err(|_| "SAIC_USERNAME is required")?;
    let password = std::env::var("SAIC_PASSWORD").map_err(|_| "SAIC_PASSWORD is required")?;
    let credentials = match std::env::var("SAIC_PHONE_COUNTRY_CODE") {
        Ok(code) => Credentials::phone(username, code, password),
        Err(std::env::VarError::NotPresent) => Credentials::email(username, password),
        Err(_) => return Err("SAIC_PHONE_COUNTRY_CODE must be valid Unicode".into()),
    };
    let mut client = SaicClient::new(ClientConfig::mg_europe(credentials))?;
    client.login().await?;
    println!("Login succeeded. No vehicle requests were sent.");
    Ok(())
}

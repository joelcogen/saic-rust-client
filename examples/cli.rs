//! Interactive manual checks. Starting this example logs in to production.
use std::{
    fs::File,
    io::{self, BufRead, Read, Write},
    process::ExitCode,
    time::Duration,
};

use saic_client::{ClientConfig, Credentials, Error, SaicClient};

const HELP: &str = "Commands:
  help    Show commands
  status  Show local session expiry (no request)
  login   Log in again
  logout  Forget the local session (no request)
  exit    Exit the CLI (quit and EOF also work)";

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--help"] || args == ["-h"] {
        println!("Usage: cargo run --example cli");
        println!("Reads .env in the current directory and logs in to MG Europe.");
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    if !args.is_empty() {
        eprintln!("Usage: cargo run --example cli [--help]");
        return ExitCode::from(2);
    }
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(".env").map_err(|_| "Couldn't open .env in the current directory. Copy .env.example and fill in your credentials.")?;
    let settings = read_settings(file)?;
    let credentials = match settings.country_code {
        Some(code) => Credentials::phone(settings.username, code, settings.password),
        None => Credentials::email(settings.username, settings.password),
    };
    let mut client = SaicClient::new(ClientConfig::mg_europe(credentials))?;
    run_session(&mut client, io::stdin().lock(), io::stdout().lock()).await
}

struct Settings {
    username: String,
    password: String,
    country_code: Option<String>,
}

fn read_settings(reader: impl Read) -> Result<Settings, &'static str> {
    let (mut username, mut password, mut country_code) = (None, None, None);
    for entry in dotenvy::from_read_iter(reader) {
        // dotenvy's errors can include the offending line, including passwords.
        let (key, value) =
            entry.map_err(|_| "Couldn't parse .env. Check its syntax and quoting.")?;
        match key.as_str() {
            "SAIC_USERNAME" => username = Some(value),
            "SAIC_PASSWORD" => password = Some(value),
            "SAIC_PHONE_COUNTRY_CODE" => country_code = Some(value),
            _ => {}
        }
    }
    let username = username
        .filter(|value| !value.trim().is_empty())
        .ok_or("SAIC_USERNAME is required in .env")?;
    let password = password
        .filter(|value| !value.is_empty())
        .ok_or("SAIC_PASSWORD is required in .env")?;
    if country_code
        .as_ref()
        .is_some_and(|code| code.trim().is_empty())
    {
        return Err("SAIC_PHONE_COUNTRY_CODE must be nonempty or omitted");
    }
    Ok(Settings {
        username,
        password,
        country_code,
    })
}

// Keep command handling testable without constructing a real HTTP client.
trait Session {
    async fn login(&mut self) -> Result<(), Error>;
    fn remaining(&self) -> Option<Duration>;
    fn logout(&mut self);
}

impl Session for SaicClient {
    async fn login(&mut self) -> Result<(), Error> {
        SaicClient::login(self).await
    }

    fn remaining(&self) -> Option<Duration> {
        self.session_remaining()
    }

    fn logout(&mut self) {
        SaicClient::logout(self);
    }
}

async fn run_session(
    client: &mut impl Session,
    mut input: impl BufRead,
    mut output: impl Write,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(output, "Logging in...")?;
    output.flush()?;
    client.login().await?;
    writeln!(output, "Logged in. Type help for commands.")?;

    loop {
        write!(output, "saic> ")?;
        output.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            writeln!(output)?;
            break;
        }
        match line.trim() {
            "" => {}
            "help" => writeln!(output, "{HELP}")?,
            "status" => match client.remaining() {
                Some(remaining) => writeln!(
                    output,
                    "Logged in locally; expires in {} seconds.",
                    remaining.as_secs()
                )?,
                None => writeln!(output, "Logged out or expired. Use login to authenticate.")?,
            },
            "login" => match client.login().await {
                Ok(()) => writeln!(output, "Logged in.")?,
                Err(error) => writeln!(output, "Login failed: {error}. Use login to retry.")?,
            },
            "logout" => {
                client.logout();
                writeln!(output, "Local session cleared.")?;
            }
            "exit" | "quit" => break,
            _ => writeln!(output, "Unknown command. Type help for commands.")?,
        }
    }
    client.logout();
    writeln!(output, "Bye.")?;
    Ok(())
}

#[cfg(test)]
#[path = "cli/specs.rs"]
mod specs;

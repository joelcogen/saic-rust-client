//! Interactive manual checks. Starting this example logs in to production.
use std::{
    fs::File,
    io::{self, BufRead, Read, Write},
    process::ExitCode,
    time::Duration,
};

use saic_client::{ClientConfig, Command, CommandOutcome, Credentials, Error, SaicClient};

const HELP: &str = "Commands:
  help    Show commands
  status  Show local session expiry (no request)
  login   Log in again
  logout  Forget the local session (no request)
  vehicle-command VIN JSON   Submit a Vehicle command payload
  charging-command VIN JSON  Submit a Charging command payload
  vehicles                   Discover vehicles and capabilities
  vehicle-cache VIN          Read cached vehicle status only
  vehicle-live VIN           Explicit live vehicle refresh
  charging-status VIN        Read charging status
  charging-management VIN    Read charging management data
  charging-settings VIN      Read charging settings
  battery-heating-schedule VIN  Read stored heating schedule
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
    async fn read(&mut self, operation: ReadCommand, vin: &str) -> Result<String, Error>;
    async fn command(&mut self, vin: &str, command: &Command) -> Result<CommandOutcome, Error>;
    async fn login(&mut self) -> Result<(), Error>;
    fn remaining(&self) -> Option<Duration>;
    fn logout(&mut self);
}

impl Session for SaicClient {
    async fn read(&mut self, operation: ReadCommand, vin: &str) -> Result<String, Error> {
        let options = Default::default();
        // Debug intentionally redacts identifiers, location and raw server data.
        Ok(match operation {
            ReadCommand::Vehicles => format!("{:?}", self.vehicles().await?),
            ReadCommand::Cached => format!("{:?}", self.vehicle_status_cached(vin, options).await?),
            ReadCommand::Live => format!("{:?}", self.vehicle_status_live(vin, options).await?),
            ReadCommand::Charging => format!("{:?}", self.charging_status(vin, options).await?),
            ReadCommand::Management => {
                format!("{:?}", self.charging_management(vin, options).await?)
            }
            ReadCommand::Settings => format!("{:?}", self.charging_settings(vin, options).await?),
            ReadCommand::BatteryHeating => {
                format!("{:?}", self.battery_heating_schedule(vin, options).await?)
            }
        })
    }
    async fn command(&mut self, vin: &str, command: &Command) -> Result<CommandOutcome, Error> {
        self.execute_command(vin, command, Default::default()).await
    }
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
            text if text.starts_with("vehicle-command ")
                || text.starts_with("charging-command ") =>
            {
                match parse_command(text) {
                    Ok((vin, command)) => match client.command(vin, &command).await {
                        Ok(outcome) => writeln!(output, "{outcome:?}")?,
                        Err(error) => writeln!(output, "Command failed: {error}")?,
                    },
                    Err(error) => writeln!(output, "{error}")?,
                }
            }
            text => match parse_read(text) {
                Ok((operation, vin)) => match client.read(operation, vin).await {
                    Ok(result) => writeln!(output, "{result}")?,
                    Err(error) => writeln!(output, "Read failed: {error}")?,
                },
                Err(error) => writeln!(output, "{error}")?,
            },
        }
    }
    client.logout();
    writeln!(output, "Bye.")?;
    Ok(())
}

#[cfg(test)]
#[path = "cli/specs.rs"]
mod specs;

// Payload kind is supplied by the command name, not by user JSON.
fn parse_command(line: &str) -> Result<(&str, Command), &'static str> {
    let mut parts = line.splitn(3, char::is_whitespace);
    let kind = match parts.next() {
        Some("vehicle-command") => "Vehicle",
        Some("charging-command") => "Charging",
        _ => return Err("Unknown command"),
    };
    let vin = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or("Expected VIN and JSON payload")?;
    let mut value: serde_json::Value =
        serde_json::from_str(parts.next().ok_or("Expected JSON payload")?)
            .map_err(|_| "Invalid command JSON")?;
    let object = value.as_object_mut().ok_or("Expected a JSON object")?;
    object.insert("kind".into(), kind.into());
    let command = serde_json::from_value(value).map_err(|_| "Invalid command fields")?;
    Ok((vin, command))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadCommand {
    Vehicles,
    Cached,
    Live,
    Charging,
    Management,
    Settings,
    BatteryHeating,
}

fn parse_read(line: &str) -> Result<(ReadCommand, &str), &'static str> {
    let mut parts = line.split_whitespace();
    let operation = match parts.next() {
        Some("vehicles") => ReadCommand::Vehicles,
        Some("vehicle-cache") => ReadCommand::Cached,
        Some("vehicle-live") => ReadCommand::Live,
        Some("charging-status") => ReadCommand::Charging,
        Some("charging-management") => ReadCommand::Management,
        Some("charging-settings") => ReadCommand::Settings,
        Some("battery-heating-schedule") => ReadCommand::BatteryHeating,
        _ => return Err("Unknown command. Type help for commands."),
    };
    let vin = if operation == ReadCommand::Vehicles {
        ""
    } else {
        parts.next().ok_or("Expected exactly one VIN")?
    };
    if parts.next().is_some() {
        return Err("Unexpected read arguments");
    }
    Ok((operation, vin))
}

# SAIC Rust client

A Rust client for the SAIC MG iSMART API. Login is implemented; vehicle data and
commands will follow. We'll build the web service in a separate repository and
use this crate as a dependency.

The package is `saic-client` (`saic_client` in Rust code). Publishing is disabled.
Login has offline test coverage but hasn't been checked against production yet.

## Login

Inside an async function running on Tokio:

```rust,no_run
use saic_client::{ClientConfig, Credentials, SaicClient};

let credentials = Credentials::email(username, password);
let mut client = SaicClient::new(ClientConfig::mg_europe(credentials))?;
client.login().await?;
```

Creating the client makes no requests. `login` authenticates once and stores the
token privately. `is_logged_in()` checks local expiry, `session_remaining()`
returns the remaining lifetime, and `logout()` forgets the local session.
There are no automatic retries, token refreshes, or vehicle calls.

Use `Credentials::phone(number, country_code, password)` for a phone account.
The config also supports `with_base_url`, `with_api_identity`, and `with_timeout`.

## Documentation

- [Login specification](docs/login.md): API behavior, failure cases, and tests.
- [Technical design](docs/technical-design.md): what belongs in the client,
  how the protocol works, and what to build and test first.
- [Reference sources](docs/references.md): Python client, MQTT gateway,
  community protocol documentation, and lessons from our existing service.

## Development

With Rust installed (checked with Rust 1.93.1):

```sh
cargo check --locked
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

Tests use synthetic data and never contact SAIC or read account credentials.
After dependencies are downloaded, add `--offline` to Cargo commands to prevent
registry access too. We don't have a mock-server integration suite.

The [login example](examples/login.rs) is for a manual production check, only
when you explicitly choose to run it. It requires `--production`, reads
`SAIC_USERNAME` and `SAIC_PASSWORD` from the environment, and optionally accepts
`SAIC_PHONE_COUNTRY_CODE`. It doesn't load an environment file or fetch vehicle
data. Keep account checks separate from vehicle wake-ups and commands.

The web service will handle MQTT if needed, HTTP routes, Notify!, polling, and
preheating rules.

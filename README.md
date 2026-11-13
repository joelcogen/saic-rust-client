# SAIC Rust client

A Rust client for the SAIC MG iSMART API. Login, vehicle discovery and status reads, and bounded vehicle/charging command
transport are implemented. Individual control helpers will follow. We'll build the web service in a separate repository and
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

- [Vehicle reads](docs/reads.md): cached/live separation, typed data and capabilities.
- [Command transport](docs/commands.md): envelopes, deadlines and unknown outcomes.
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

## Manual CLI

Copy [.env.example](.env.example) to `.env` in the repository root and fill in
`SAIC_USERNAME` and `SAIC_PASSWORD`. For a phone account, also set
`SAIC_PHONE_COUNTRY_CODE`. Single-quote passwords so `$` and `#` stay literal.
The `.env` file is ignored by Git.

Start the [CLI](examples/cli.rs) from that directory:

```sh
cargo run --example cli
```

Starting it logs in to MG Europe, then waits for commands on the same client:

| Command | Action |
| --- | --- |
| `help` | Show commands |
| `status` | Show local session expiry without making a request |
| `login` | Log in again using the same credentials |
| `vehicle-command VIN JSON` | Submit an explicit vehicle control payload |
| `charging-command VIN JSON` | Submit an explicit charging control payload |
| `vehicles` | Discover account vehicles and model capabilities |
| `vehicle-cache VIN` | Read SAIC's cached status only |
| `vehicle-live VIN` | Explicit live vehicle refresh |
| `charging-status VIN` | Read charging status |
| `charging-management VIN` | Read charging management data |
| `charging-settings VIN` | Read charging settings |
| `battery-heating-schedule VIN` | Read the stored battery-heating schedule |
| `logout` | Forget the local session without contacting SAIC |
| `exit` or `quit` | Exit; EOF and Ctrl-C also exit |

The CLI reads `.env` from the current directory without searching parent
directories or loading values into the process environment. It doesn't take
account credentials directly from shell variables. If startup login fails, it
exits; a failed interactive `login` leaves the prompt available for another try.
There are no automatic retries after transport failures or background requests.
Command continuation uses only the same server event ID; commands never refresh
vehicle or charging status. Unknown outcomes must not be blindly replayed.

Payload syntax (synthetic VIN; only execute against a vehicle when intended):

```text
vehicle-command SYNTHETICVIN00001 {"rvcReqType":"0","rvcParams":[{"paramId":1,"paramValue":"AQ=="},{"paramId":2,"paramValue":"AA=="},{"paramId":3,"paramValue":"AQ=="},{"paramId":255,"paramValue":"AAAAAA=="}]}
charging-command SYNTHETICVIN00001 {"chrgCtrlReq":1,"tboxEleccLckCtrlReq":0,"tboxV2XReq":0}
```

The vehicle example requests lights without horn; the charging example requests
charging. All fields are caller-supplied protocol values. The CLI prints the
outcome and code with identifiers and response data redacted. Read commands print
the returned type with data redacted; use the library fields to inspect values.
Cached reads never fall back to live reads. To refresh after several controls,
issue `vehicle-live VIN` explicitly when the sequence is finished.

Use `cargo run --example cli -- --help` to see usage without reading `.env` or
logging in. The old one-shot login example is replaced by this CLI.

The CLI's tests run with `cargo test` using a fake session. They never read your
`.env` or make requests. Only start the real CLI when you intend to log in.

The web service will handle MQTT if needed, HTTP routes, Notify!, polling, and
preheating rules.

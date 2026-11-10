# SAIC Rust client

A Rust client for the SAIC MG iSMART API. It will handle login, vehicle data,
and individual commands. We'll build the web service in a separate repository
and use this crate as a dependency.

For now, this is documentation and an empty crate. No API calls or vehicle
controls work yet. The package is `saic-client` (`saic_client` in Rust code).
Publishing is disabled.

## Documentation

- [Technical design](docs/technical-design.md): what belongs in the client,
  how the protocol works, and what to build and test first.
- [Reference sources](docs/references.md): Python client, MQTT gateway,
  community protocol documentation, and lessons from our existing service.

## Development

With a Rust toolchain installed:

```sh
cargo check
cargo fmt --check
cargo test
```

There are no dependencies or tests yet. We'll start with the protocol, then add
login, vehicle discovery, and cached status. Test these against a mock server
before trying them with a real account.

The web service will handle MQTT if needed, HTTP routes, Notify!, polling, and
preheating rules.

# Working on this client

Run offline unit tests for normal development. Don't run against SAIC production,
use real account credentials, or contact a vehicle unless the user explicitly
requests that specific check. Available credentials aren't permission to use
them. Login approval doesn't authorize vehicle reads or commands.

Keep credentials and real account data out of commits and tool output. Use
synthetic fixtures for tests. Follow the direct, concise style in the docs.

When adding an operation to the client, add a matching command to the interactive
CLI in `examples/cli.rs` in the same change. Update its help text, README command
list, and offline command-handling specs so the operation can be tested manually.
Adding a CLI command does not authorize running it against production.

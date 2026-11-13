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

Vehicle commands must never refresh vehicle or charging status, before or after
execution, including on failure. The caller owns refresh timing so it can chain
commands and refresh once. Protocol continuation to obtain the result of the
same command is allowed; it must not submit a new command or fetch vehicle status.
If a command requires existing values, accept explicit caller-provided values
rather than fetching them implicitly. Apply this rule to the interactive CLI too.

Send seat-heater changes separately for each seat. A request must contain only
the selected seat's parameter and any required protocol marker; never send a
default, cached value, or off value for the other seat. Setting both seats means
two independent commands, with no implicit status refresh between them.

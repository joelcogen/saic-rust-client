# Command transport

`SaicClient::execute_command(vin, &command, options)` submits a low-level
`Command::Vehicle` or `Command::Charging` envelope. These map to
`POST /vehicle/control` and `POST /vehicle/charging/control`. VINs are SHA-256
hashed internally. All payload fields are explicit; the transport does not
infer model capabilities or obtain existing state. Use the [climate builders](climate.md) for validated cabin climate payloads.
Other charging endpoints are outside this transport milestone.

Commands use the login session's `blade-auth` token in both the header and
request encryption/signature. An absent or locally expired session fails before
submission. A server authentication failure clears the session. Neither path
logs in automatically.

`CommandOptions` defaults to a 30-second overall deadline, a 10-second request
timeout, and a one-second continuation delay. All must be positive. The HTTP
client's configured timeout also applies, including reading the response body.
The overall deadline covers requests and delays. Responses are limited to 1 MiB.

The first request uses `event-id: 0`. A pending response must supply a nonempty,
nonzero event ID before a continuation is allowed. Continuations repeat only the
same endpoint and payload with that ID. A changed ID stops the operation with an
unknown outcome. Codes 2, 3 and 7 are explicit rejections; 401 and 403 invalidate
the session. Other nonzero codes with an established event remain pending, as in
the pinned Python implementation. Code zero with an object result completes the
exchange; it does not prove the car reached the desired physical state.

`CommandOutcome` distinguishes `Completed`, `Rejected`, `InvalidSession`, and
`Unknown`. It preserves the last event ID, SAIC code, HTTP status, server message
and result object where available. Response-specific fields remain in `data`
without assuming model-specific meanings. HTTP failures, malformed responses,
decryption errors and interrupted transport are unknown outcomes, never reasons
to replay the submission. Pre-submission validation errors return `Error`.

`execute_command_until` additionally accepts a cancellation future. When it
resolves, waiting stops and returns an unknown outcome with the last event
context. Dropping either command future also stops waiting, but cannot return an
outcome. Neither form cancels execution at SAIC. An unpolled future does nothing.
There are no background tasks, retries after transport failures, or automatic
vehicle/charging status requests, including after errors. The caller owns refresh
timing and any decision to submit another command.

Debug output omits event IDs and raw data/messages. Access those fields explicitly
only when needed; they can contain account or vehicle identifiers.

Vehicle parameters use protocol IDs and base64 byte strings. Seat requests must
contain exactly one seat parameter and the `255` marker (`AAAAAA==`). Changing
two seats requires two calls. Remote-refresh request type `18` is rejected.
This low-level interface leaves other protocol value and capability validation to
the caller; it does not supply defaults for parameters.

## Offline verification

Fake exchanges cover immediate completion, pending continuation, explicit
rejection, invalid sessions, malformed responses, changed/zero event IDs, HTTP
and transport errors, timeouts, cancellation and dropped futures. They assert
that requests stay on the chosen command endpoint and never restart an event.
The CLI uses a fake session to test both command forms and malformed input.
An independent synthetic Python vector checks authenticated encryption, signing,
VIN hashing and charging serialization. No production checks were performed.

## Sources

The transport follows [base.py](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/base.py),
[vehicle commands](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/vehicle/__init__.py),
[charging commands](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/vehicle_charging/__init__.py)
and `net/crypto.py` at the same revision. Local gateway HEAD
`ec814eca135792d8202dfd29d6e443b53746d2a1` plus `src/saic_api.py` was consulted
for partial seat payloads. See [third-party notices](../THIRD_PARTY_NOTICES.md).

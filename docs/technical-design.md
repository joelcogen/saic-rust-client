# SAIC Rust client technical design

We'll build an async Rust client that talks directly to SAIC, using the Python
client as the protocol reference. The web service will live in a separate
repository and handle polling, preheating rules, and notifications. This is the
design for that client. [Login](login.md) is implemented and tested offline;
[command transport](commands.md) and [vehicle reads](reads.md) are implemented;
individual control helpers are still planned.

## What belongs in the client

```text
Future Rust web service  -->  saic-client  -->  SAIC API  -->  vehicle
```

| Client library | Future web service |
| --- | --- |
| Account and regional configuration | Environment configuration and secret storage |
| Authentication and session lifecycle | HTTP authentication and routes |
| Request encoding, encryption, and response decoding | Dashboard and Notify! integration |
| Vehicle discovery, cached status, explicit live status | Polling schedule, application cache, and freshness policy |
| Individual vehicle commands and protocol continuations | Preheating sequence, temperature thresholds, and final refresh |
| Structured failures and operation outcomes | User-facing progress, partial workflow results, and recovery decisions |

Creating a client shouldn't contact SAIC or start background polling. The caller
chooses when to log in, which vehicle to use, and when to make a request that
could wake it. The crate doesn't need MQTT or a web server.

## Protocol findings

The behavior below comes from the [Python sources](references.md). It describes
that implementation, rather than an official SAIC specification.

### Authentication and configuration

The Python `api/base.py` submits a form to `POST /oauth/token`, uses a SHA-1
password digest, and records the returned access token and expiry. Account type
affects login fields; phone login includes a country code. Configuration also
includes the base URL, region, tenant ID, and device app ID.

Start with the MG Europe profile from `model.py`:
`https://gateway-mg-eu.soimt.com/api.app/v1/`, region `eu`, tenant `459771`, and
app ID `com.saicmotor.europecar`. Keep these values together as a profile and
allow overrides. We'll add other regions once we've tested them.

Concurrent callers shouldn't trigger competing logins. If a session becomes
invalid, return a typed authentication error. Logging in again must not silently
resubmit a command that may already have run.

### Transport and encryption

Python's `net/crypto.py` derives request
keys using the request path, tenant, token, timestamp, and content type. It uses
AES-CBC body encryption and an HMAC-SHA256 verification string, with MD5-derived
values in the protocol. Response decryption uses response headers.

Keep this in a private protocol module. Match byte encoding, padding, timestamp
units, form serialization, path/query handling, and content-type normalization
against fixed Python test vectors before contacting SAIC. Use maintained crypto
libraries for the algorithms the protocol requires, and keep HTTPS certificate
validation enabled.

### Vehicle reads and commands

The reviewed vehicle API uses these operations:

| Method and path | Python behavior |
| --- | --- |
| `GET /vehicle/list` | Returns account vehicles |
| `GET /vehicle/status/cache` | Reduced cached status with `statusTime`; documented by Python as not waking the vehicle |
| `GET /vehicle/status` | Requests vehicle status through the event-ID flow |
| `POST /vehicle/control` | Submits a typed control request through the event-ID flow |

The status and control implementations hash the VIN with SHA-256. The client
should handle that internally, so callers pass the original VIN. Keep cached
status separate from live status, with its observation timestamp and any missing
fields. A cached read must never silently fall back to a live request.

Before adding charging data and climate controls, check their endpoints and
payloads in the Python client. MQTT topic names don't define those payloads.

### Event continuation and uncertain outcomes

Python starts event-based operations with `event-id: 0` and can repeat the request
with a server-provided event ID while awaiting data. Its base implementation
uses a 30-second retry window, but the vehicle isn't guaranteed to finish within
it. Continuing an existing event is different from submitting a new command
after a connection failure.

Keep the event ID and set both a deadline for the whole operation and a timeout
for each request. Callers need to distinguish these outcomes:

- SAIC reports completion, with the decoded response.
- SAIC explicitly rejects the operation, with its code and message.
- Submission or continuation is interrupted and the outcome is unknown.

A successful response doesn't prove the car has reached the requested state.
The service decides when to fetch status to check. Dropping a future or timing
out stops us waiting; it doesn't cancel a command SAIC has already accepted.
Don't automatically retry a command when we don't know whether it ran.

### Partial control updates

Changing one seat must leave the others alone. Python's current single-seat
method sends one seat parameter plus the terminator. Our local gateway change
does the same. Use a seat selector and a validated level, or a patch type that
distinguishes an omitted seat from one explicitly turned off.

Start with climate start/stop, front seat heating, steering-wheel heating,
defrost, and finding the car with its lights. The Python find-car method defaults
the horn on, so require an explicit choice or provide a lights-only method.
Check supported features and value ranges for each vehicle model.

## Proposed Rust structure

Keep one library crate, split into these modules:

| Module | Responsibility |
| --- | --- |
| `config` | Profiles, credentials, and timeout options |
| `client` | Public asynchronous operations and session ownership |
| `transport` | HTTP requests, deadlines, and injectable test transport |
| `protocol` | Encryption, serialization, envelopes, and event continuation |
| `models` | Vehicle identifiers, status, capabilities, and command parameters |
| `error` | Typed failures and diagnostic context |

Keep types used only for API payloads private where possible. Public types should
document units and represent missing measurements explicitly. Preserve unknown
status values instead of treating them as off or zero. Validate control values
before sending anything.

Login uses reqwest with rustls for HTTP, Serde for serialization, thiserror for
errors, and RustCrypto for hashes, HMAC, and AES-CBC. Tokio runs the async examples
and tests. Exact dependency versions are in `Cargo.lock`. The code has been
checked with Rust 1.93.1; we haven't set a minimum supported release version yet.

Use separate error types or variants for configuration, authentication,
transport, HTTP errors, SAIC rejection, malformed responses, decryption failures,
and unknown outcomes. Keep the HTTP status, SAIC code, and event ID as separate
fields so callers don't have to parse error messages. Default logs and debug
output must omit passwords, tokens, VINs, coordinates, and raw request or response
bodies.

The service coordinates sequences of commands, including calls from different
processes. The library protects its session state and documents which calls can
run concurrently. A sequence of vehicle commands can still finish only partly.

## Implementation milestones

1. Pin the Python reference, create synthetic test vectors, and
   implement configuration, encoding, encryption, and response envelopes.
   Validate bytes and headers without contacting SAIC.
2. Add login, discovery, and cached status. Mock
   expiry, invalid credentials, malformed data, and multiple vehicles. Confirm
   that cached reads never fall back to a wake-up request.
3. Add live status with event continuation and charging
   reads, with bounded deadlines and preserved timestamps. Verify continuation
   reuses the event ID and does not restart the operation.
4. Add the initial controls and payload tests,
   including omitted seats, lights without horn, rejected commands, and lost
   responses. Expose unknown outcomes without automatic replay.
5. Document the public API and examples; let the separate
   service depend on a pinned Git revision initially. Move application workflows
   only after the client behavior is validated.

## Validation and open decisions

Use fixed test inputs for crypto and serialization, fake exchanges for session
behavior, and sanitized fixtures for missing or unknown data. We aren't building
a mock-server integration suite for now; production checks will be manual.
Use the Python tests as a source of cases, retaining attribution if copied.
Automated tests must run without credentials or a real vehicle.

Only run a production check when the user explicitly requests that specific
check. Start with login; vehicle reads and controls are separate steps. Record
what worked and on which model without committing personal identifiers. Keep
the existing application tests as references for the later service.

Login follows the pinned Python revision in [Reference sources](references.md)
and starts with the MG Europe profile. Before adding controls, decide which
model capabilities to support, the default deadlines, and how the API returns
outcomes. Publishing also needs a license,
minimum Rust version, and release policy.

# Login specification

Login is an explicit async operation on `SaicClient`. It makes one request to
`POST /oauth/token` and stores the returned token privately. It doesn't fetch
vehicles, wake the car, or start a background task to refresh the session.
Production behavior hasn't been verified yet.

## Public API

- `Credentials::email(username, password)` selects email login.
- `Credentials::phone(number, country_code, password)` selects phone login.
- `ClientConfig::mg_europe(credentials)` supplies the MG Europe endpoint and app
  identity. The endpoint, identity, and timeout can be overridden.
- `SaicClient::new(config)` validates the config without contacting SAIC.
- `client.login().await` authenticates and returns `Result<(), Error>`.
- `client.is_logged_in()` and `client.session_remaining()` inspect local expiry.
- `client.logout()` drops the local session; it doesn't revoke it on the server.

Login takes `&mut self`, so callers can't run competing logins on the same
instance. Creating a login future does nothing until it is polled. Once it
starts, it clears the previous session. A failed or cancelled login leaves the
client logged out, even if an older session was valid.

Expiry uses a monotonic clock and starts counting at request submission, so time
spent waiting for the response doesn't extend the token's lifetime. SAIC can
invalidate a token earlier; the local expiry check can't detect that.

## Request and response

The form includes the account name, SHA-1 password digest, login type, app device
ID, scope, device type, and language. Phone login also includes the country code.
The form body is encrypted with AES-128-CBC and PKCS#7 padding, then hex encoded.
The request includes SAIC's timestamp, tenant, region, and verification headers.
Login uses an empty prior token when deriving the encryption key and signature.

The implementation follows the [pinned Python client](references.md), including
its form encoding and app identity. The response may be plain JSON or encrypted
JSON identified by SAIC's response headers. A successful envelope must have code
zero, a nonempty access token, and a positive integer lifetime. Unknown response
fields are ignored. Missing or malformed required fields are errors.

The default timeout is 10 seconds for the whole HTTP request, including the body.
HTTPS and certificate validation are required. Redirects and automatic retries
are disabled. The response body is limited to 1 MiB after HTTP decompression.

## Errors and account data

| Error | Meaning |
| --- | --- |
| `Configuration` | Invalid endpoint, empty credentials, invalid identity, or zero timeout |
| `Authentication` | HTTP or SAIC code 401 or 403 |
| `Rejected` | Another nonzero SAIC code in a successful HTTP response |
| `Http` | Another unsuccessful HTTP status, including redirects |
| `Timeout` or `Transport` | The HTTP exchange failed |
| `Decryption` | Invalid encryption headers, ciphertext, or padding |
| `InvalidResponse` | Malformed envelope, unusable token or expiry, or oversized response |
| `Clock` | The system clock is before the Unix epoch |

Server failures expose HTTP status and SAIC code separately. Their message is
available through `ApiFailure::message()`, but default error formatting omits it
because SAIC could echo account data. Debug output also hides credentials and
tokens. The client doesn't log request or response bodies or persist sessions.
Stored credentials and tokens use buffers that are cleared when dropped; this
isn't a guarantee that every temporary copy in the process is erased.

## Offline specs

`cargo test --locked` runs only offline unit tests. The request vectors come from
the Python implementation using invented accounts and a fixed timestamp.
Tests check the form, ciphertext, signature, headers, and decrypted response.
Session tests use a fake exchange inside the same login flow as production.
They cover success, rejection, transport failure, cancellation, expiry, and
re-login without retrying or carrying forward an old token.

There is no mock-server integration suite and no production test in Cargo's test
targets. These specs don't verify SAIC availability, real credentials, or the
complete HTTP exchange. Those checks remain manual.

## Manual checks

The [interactive CLI](../examples/cli.rs) reads `.env` from the current directory,
logs in once, and waits for commands until exit. Starting it is the explicit
production action; no extra `--production` flag is needed. `--help` exits before
reading credentials or contacting SAIC. See the [README](../README.md#manual-cli)
for setup and commands.

The prompt uses the same client throughout. `status` and `logout` are local
operations; `login` makes another authentication request. The CLI doesn't make
vehicle requests or automatically renew expired sessions. Its unit tests use a
fake session and synthetic `.env` contents, so `cargo test` never starts a real
login.

Only run against production when the user explicitly requests that specific
check. Available credentials aren't permission to use them. Future vehicle
reads and commands need their own explicit instruction.

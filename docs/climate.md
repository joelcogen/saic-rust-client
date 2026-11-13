# Cabin climate controls

Build a `Command` and submit it with `SaicClient::execute_command(vin, &command,
options)`. Builders validate locally and make no requests. Submission uses the
existing authenticated, bounded command transport and returns `CommandOutcome`.

```rust,no_run
use saic_client::{Command, CommandOptions, TemperatureProfile};
# async fn example(client: &mut saic_client::SaicClient) -> Result<(), saic_client::Error> {
let command = Command::start_ac(TemperatureProfile::Eh32, 22)?;
let outcome = client.execute_command("SYNTHETICVIN", &command, CommandOptions::default()).await?;
# Ok(())
# }
```

| Builder | Behavior |
| --- | --- |
| `start_ac(profile, celsius)` | Fan 2, encoded target, AC flag omitted |
| `stop_ac()` | Fan 0, temperature omitted, AC off |
| `start_ac_blowing()` | Fan 1, temperature index 0, AC off |
| `start_front_defrost()` | Fan 5, temperature index 8, AC on |
| `stop_front_defrost()` | Same command as `stop_ac()` |
| `set_ac_temperature(profile, celsius)` | Reissue `start_ac` at the supplied target |
| `control_climate(fan, ac, index)` | Explicit protocol parameters |

Temperature changes always submit AC start, including repeated targets and when
AC was previously off. They do not store preferences, suppress unchanged targets,
or inspect current climate state. Applications that want to store a target while
AC is off must manage that preference themselves.

The caller must select the temperature profile from known vehicle context:

| Profile / CLI spelling | Celsius range, inclusive | Protocol encoding |
| --- | --- | --- |
| `Eh32` / `eh32` | 17–31 | Celsius − 17 + 3 |
| `Mzs3e` / `mzs3e` | 16–31 | Celsius − 16 + 2 |
| `Standard` / `standard` | 16–28 | Celsius − 16 + 2 |

These match the gateway's EH32 and MZS3E series prefixes and its fallback.
There is no default profile or automatic discovery. Targets are whole degrees;
out-of-range values are rejected rather than silently clamped as in the gateway.

`control_climate` accepts fan and temperature index values in 0..=255, matching
the Python client's one-byte encoding. These are protocol values, not Celsius
or a guarantee of vehicle support. The caller supplies supported values.
`ac` is `Some(true)`, `Some(false)`, or `None` to omit that parameter. Fan zero
forces AC off and omits temperature regardless of supplied values, as upstream
does. Both numeric arguments are validated before this normalization.

All modes use request type `6` on `/vehicle/control`, parameter IDs 19 (fan),
20 (temperature), 22 (AC), and the required 255 marker. No command reads vehicle
or charging status, even on failure. Continuation only obtains the result of the
same command. Callers own refresh timing and any subsequent submission.

## Sources and offline checks

Payload behavior follows the pinned Python client
[`api/vehicle/climate/__init__.py`](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/vehicle/climate/__init__.py).
Temperature encoding and front-defrost stop follow local gateway HEAD
`ec814eca135792d8202dfd29d6e443b53746d2a1`, `src/vehicle_info.py`, and
`src/handlers/command/climate/`. Local `src/saic_api.py` changes affect seats,
not cabin climate. Gateway cached-state suppression and refreshes are not ported.

Synthetic Python fixtures cover modes, omitted/on/off AC, and protocol/profile
boundaries. Rust tests cover invalid values, all Celsius targets, command
continuation and transport failure; CLI fake-session tests check dispatch,
repeated targets, malformed arguments, and absence of implicit reads. No
production calls are needed. See [third-party notices](../THIRD_PARTY_NOTICES.md).

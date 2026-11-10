# Reference sources

Reviewed on 10 November 2026. These links follow upstream branches, so their
contents can change. When porting an operation or importing test vectors, pin
the source commit and record it with the fixture. Keep credentials and raw live
captures out of this repository.

## Python client

The login implementation and its test vectors follow commit
[`8ab0634b292e64f5198d979636e01616ea6af34f`](https://github.com/SAIC-iSmart-API/saic-python-client-ng/tree/8ab0634b292e64f5198d979636e01616ea6af34f).
The source map below still links to the moving branch for general research.
See the [login spec](login.md) for the implemented behavior and
[fixture notes](../tests/fixtures/README.md) for how the vectors were generated.

Start with
[SAIC-iSmart-API/saic-python-client-ng](https://github.com/SAIC-iSmart-API/saic-python-client-ng).
Our local MQTT gateway declares `saic-ismart-client-ng >=0.10.0,<0.11.0` in its
`pyproject.toml`. The upstream `master` branch may contain behavior beyond that
dependency range, so check the installed version when comparing behavior.

| Source on master | What to consult it for |
| --- | --- |
| [README](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/master/README.md) | Supported features and regional configuration presets |
| [model.py](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/master/src/saic_ismart_client_ng/model.py) | Base URL, region, tenant, app identity, account type, and timeouts |
| [api/base.py](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/master/src/saic_ismart_client_ng/api/base.py) | Login, token expiry, response envelopes, and event-ID continuation |
| [net/crypto.py](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/master/src/saic_ismart_client_ng/net/crypto.py) | Request encryption, verification headers, and response decryption |
| [crypto_utils.py](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/master/src/saic_ismart_client_ng/crypto_utils.py) | Hashing, hexadecimal encoding, AES-CBC, and padding |
| [Vehicle API](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/master/src/saic_ismart_client_ng/api/vehicle/__init__.py) | Discovery, live and cached status, control submission, and lights/horn options |
| [Climate API](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/master/src/saic_ismart_client_ng/api/vehicle/climate/__init__.py) | Climate, seat, steering-wheel, and rear-window controls |
| [Tests](https://github.com/SAIC-iSmart-API/saic-python-client-ng/tree/master/tests) | Crypto and decoding cases, cached status, individual seats, and errors |

The older [saic-python-client](https://github.com/SAIC-iSmart-API/saic-python-client)
is useful for historical context. We'll base this library on `-ng`.

The reviewed `model.py` Git blob was
`888a4377c766916cf3986c5ecc8127281d45d9bc` and `crypto_utils.py` was
`3ca91e27b402d16113794692f4e3d63836c6d557`. These identify individual files, not
a pinned release of the whole client.

## MQTT gateway

[SAIC Python MQTT Gateway](https://github.com/SAIC-iSmart-API/saic-python-mqtt-gateway)
shows how an application uses the client. Use it to understand polling and
command handling. We won't depend on it, and its MQTT topics don't define the
SAIC HTTP API.

The local checkout at `../saic-python-mqtt-gateway` has HEAD
`ec814eca135792d8202dfd29d6e443b53746d2a1` and uncommitted changes at review time.
Its remote is [joelcogen/saic-python-mqtt-gateway](https://github.com/joelcogen/saic-python-mqtt-gateway).
Some local changes aren't included in that commit or upstream.

Useful local files:

- `src/handlers/vehicle.py` and `src/vehicle.py`: refresh decisions and vehicle state.
- `src/handlers/vehicle_command.py` and `src/handlers/relogin.py`: command handling,
  authentication recovery, and refresh requests.
- `src/handlers/command/climate/`: translation from MQTT commands to client methods.
- `src/status_publisher/`: interpretation and publication of vehicle and charging data.
- `src/saic_api.py` and `tests/test_saic_api.py`: local, uncommitted adaptation
  ensuring that a single-seat request omits the other seat.

The upstream client already provides `control_heated_seat` for one seat.
Its older plural `control_heated_seats` method still defaults both front seats to
zero and sends both parameters. In Rust, changing one seat should leave the
other alone.

## Existing web service

The local `../saic-mqtt-service` repository, reviewed at HEAD
`84ae878120c565ac3c50434e1a4a204269b53a8b`, captures the behavior we want to keep:

| Files | Behavior to preserve in the future service |
| --- | --- |
| `README.md`, `app.py` | Preheating, lights-only location, explicit refresh, and conservative polling |
| `test_climate_batch.py` | Correlate results; distinguish partial or unknown outcomes; do not blindly replay after disconnect |
| `test_climate_refresh.py` | Refresh behavior around climate operations |
| `test_temperature_override.py` | Temperature override validation and preheating thresholds |
| `test_climate_activity.py`, `notify_bridge.py` | Notify! and Live Activity behavior |

The service README references
[gateway PR 518](https://github.com/SAIC-iSmart-API/saic-python-mqtt-gateway/pull/518)
for ordered MQTT batches and a final refresh. Check the PR's status before
relying on it. In Rust, the service will coordinate individual client calls;
we can't assume a batch will either complete fully or do nothing.

## Community protocol documentation

[SAIC-iSmart-API/documentation](https://github.com/SAIC-iSmart-API/documentation)
contains a [protocol reference](https://github.com/SAIC-iSmart-API/documentation/blob/main/README.adoc)
and [ASN.1 schemas](https://github.com/SAIC-iSmart-API/documentation/tree/main/ASN.1%20schema).
Use these to investigate fields and older protocol variants. Check the current
endpoints before deciding whether we need ASN.1 support.

The login protocol is ported from the Python client. Its attribution and license
are in [Third party notices](../THIRD_PARTY_NOTICES.md). Keep those notices with
the code and fixtures. We still need to choose this project's license before
publishing.

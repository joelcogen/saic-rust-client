# Find-car lights and horn

Build `Command::find_my_car(mode)` using `FindCarMode`, then submit it with
`SaicClient::execute_command` and an explicit VIN. `LightsOnly` is the enum's
default. `HornOnly` and `HornAndLights` explicitly request an audible horn.
`Stop` disables find-car and sets both horn and lights to off. These four modes
exclude ambiguous combinations such as enabling find-car with neither effect.

The CLI accepts `find-car VIN [lights-only|horn-only|horn-and-lights|stop]`.
Omitting the mode selects lights only. Unknown modes, extra arguments and
attempts to specify a duration fail locally. The pinned current client exposes
no duration, repetition, volume or light-pattern settings.

All modes use request type `0`, parameters 1 (enable), 2 (horn), 3 (lights),
and the four-zero-byte parameter 255 marker. Flags are single bytes encoded
as base64. No command reads vehicle or charging status before, after, or on
failure. Only continuation of the same command event is automatic. Callers own
refresh timing. API completion does not verify the physical lights or horn.

## Sources and offline verification

Gateway behavior comes from `src/handlers/command/location/location_find_my_car.py`
at `../saic-python-mqtt-gateway`, branch `fix/single-seat-heating`, revision
`ec814eca135792d8202dfd29d6e443b53746d2a1`. Its local `src/saic_api.py` adaptation
concerns seats. The gateway's automatic refresh is intentionally not ported.
Its `activate` mode maps to explicit `HornAndLights` here.

Payloads follow `control_find_my_car` and `schema.py` in the pinned
[current Python vehicle API](https://github.com/SAIC-iSmart-API/saic-python-client-ng/tree/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/vehicle).
The Python default enables the horn; this client's default is lights only.
No legacy operations or transport are used.

Synthetic fixtures cover all four modes. Offline specs check default horn
suppression, invalid CLI arguments, dispatch without reads on success/failure,
and same-event protocol continuation. No production calls are needed.

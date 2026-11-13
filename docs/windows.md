# Windows and sunroof

Build a command, then pass it to `SaicClient::execute_command` with the original
VIN and command options:

```rust,no_run
use saic_client::{Command, DoorWindowsAction, VehicleWindowId};

let driver = Command::close_driver_window();
let sunroof = Command::control_sunroof(true);
let selected = Command::control_windows(false, &[
    VehicleWindowId::Passenger, VehicleWindowId::RearLeft,
])?;
let doors = Command::control_door_windows(DoorWindowsAction::Ventilate);
# Ok::<(), saic_client::Error>(())
```

`control_windows` accepts open/close and a nonempty slice of unique selectors:
`Sunroof`, `Driver`, `Passenger`, `RearLeft`, `RearRight`. Front selectors are
roles, rear selectors are physical sides. Empty or duplicate selections fail
locally. Input order does not affect protocol order. Old Python aliases
`WINDOW_2/3/4` are not exposed; use the descriptive names.

All four operations use the current JSON `/vehicle/control` request type `3`.
Parameters 8–12 select windows with byte 1; byte 0 leaves a window unselected,
not closed. Parameter 13 supplies the shared action. No terminator is appended.
Selected-window open uses action 3 and close uses 0. `control_sunroof` selects
only the sunroof; `close_driver_window` selects only the driver window.

`control_door_windows` always selects all four door windows and excludes the
sunroof. Its actions are `Close` (0), `Ventilate` (1) and `Open` (2). It cannot
be combined with individual selectors. Ventilation for a single window or
sunroof, percentage positions and tilt are not supported by these builders.

## Capability limits

The pinned Python source reports that MG S6 EV uses all-door action 2 to open
and does not accept single-window control through this API. Selected-window
action 3 remains available for other models. This is upstream evidence, not
live validation of this Rust client. Support varies by model and market; the
caller must establish capability before submitting commands. There is no
implicit capability discovery or fallback to a different action/transport.
Upstream also reports that ventilated and fully open windows share status
values, so a later status read cannot distinguish those positions.

No command reads existing state or refreshes vehicle/charging status before,
after or on failure. Only continuation of the same command event is allowed.
The caller can chain commands and explicitly refresh once afterwards. API
completion does not verify physical movement; unknown outcomes must not be
blindly replayed. The CLI follows the same rules.

## Offline verification and sources

Synthetic fixtures cover all 31 nonempty selector sets for both actions, all
helpers and door actions, and invalid selections/CLI combinations. Fake command
exchanges cover same-event continuation and transport failure; fake CLI sessions
check dispatch, payloads and absence of reads on success and failure. No
production calls or live vehicle validation were performed.

Payloads follow the pinned current Python
[window API](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/vehicle/windows/__init__.py),
[window schema](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/vehicle/windows/schema.py)
and [vehicle schema](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/vehicle/schema.py).
Gateway `ec814eca135792d8202dfd29d6e443b53746d2a1` and its local
`src/saic_api.py` override were inspected; they add no window controls.
No legacy ASN.1 endpoints are used. See [third-party notices](../THIRD_PARTY_NOTICES.md).

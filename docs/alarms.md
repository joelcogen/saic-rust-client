# Alarm switches

Use `SaicClient::get_alarm_switch(vin, options)` to read switches and
`Command::set_alarm_switches(&switches)` with `execute_command` to set them.
The read uses `GET /vehicle/alarmSwitch?vin=SHA256`; the command uses
`PUT /vehicle/alarmSwitch` with a hashed VIN and `alarmSwitchList`.
Both are single requests without event continuation.

```rust,no_run
use saic_client::{AlarmSwitch, AlarmType, Command};

let command = Command::set_alarm_switches(&[
    AlarmSwitch {
        alarm_type: AlarmType::Geofence,
        alarm_enabled: false,
        function_enabled: false,
    },
    AlarmSwitch {
        alarm_type: AlarmType::VehicleFault,
        alarm_enabled: true,
        function_enabled: true,
    },
])?;
# Ok::<(), saic_client::Error>(())
```

Supported types are `VehicleFault` (0), `Geofence` (2), and `VehicleStart` (3).
Every entry requires both `alarm_enabled` and `function_enabled`. These encode
as `alarmSwitch` and `functionSwitch`, with true = 1 and false = 0. There are
no default flags or implicit existing-state reads. Empty lists and duplicate
types reject locally; only supplied entries are transmitted. The current
source does not establish the server's treatment of omitted types, so callers
requiring a complete configuration should supply all three explicitly.

The pinned Python helper enables both flags. Rust exposes the current schema's
two integer switches as explicit booleans to allow disabling and mixed flags.
These cases are verified offline, not against a vehicle. Support and the
physical effect of each flag depend on the server and vehicle.

Read responses use `models::AlarmSwitchResp` and `AlarmSwitchStatus`.
Missing/null fields remain `None`, unknown integer codes remain available,
and unknown fields are preserved in `extra`; Debug redacts response data.
Writes accept only the three supported types and boolean flags.

No command refreshes vehicle or charging status before, after, or on failure.
No PIN, account timezone, capability lookup, or existing switch read is needed.
An empty success result is valid. Rejection and invalid-session outcomes are
preserved; transport failures, timeouts and unexpected pending responses leave
an unknown outcome and never cause a retry. Completion means API completion,
not verified alarm behavior.

## Interactive CLI

```text
alarm-switches SYNTHETICVIN
alarm-switches-set SYNTHETICVIN geofence off off
alarm-switches-set SYNTHETICVIN vehicle-fault on on geofence on on vehicle-start on on
```

The setter takes one or more `TYPE ALARM FUNCTION` triples, using the type names
above in kebab-case and `on`/`off` for each flag. The read prints a redacted
response. Both commands have offline parsing and dispatch specs.

## Sources and legacy intent

The current [alarm API](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/vehicle/alarm/__init__.py)
and [schema](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/vehicle/alarm/schema.py)
define the endpoint, method, hashed VIN, flags and type codes.
Gateway `ec814eca135792d8202dfd29d6e443b53746d2a1`,
`src/mqtt_gateway.py::__register_alarm_switches`, registers all current alarm
types with both flags enabled. Its local `src/saic_api.py` override changes
seat heating only.

Legacy [saic_api.py](https://github.com/SAIC-iSmart-API/saic-python-client/blob/690506138497d3dc519aef4153f1b7a24d3f2156/src/saic_ismart_client/saic_api.py)
`set_geofence_alarm_switch` enables a legacy `REGION` switch and supplies a
hard-coded PIN. Its geofence intent is represented here by `AlarmType::Geofence`
and explicit flags in the general setter. The legacy method name and `REGION`
alias are not exposed: the current enum names this switch `GEOFENCE`, and its
JSON request has no PIN. No legacy transport or credentials are copied.
This controls the alarm switch, not geofence geometry or boundaries.

Synthetic `alarm.json` fixtures cover every supported type and all four flag
combinations, invalid inputs, and missing/unknown read fields. Fake exchanges
verify GET/PUT paths, hashed VINs, encrypted bodies, single-request behavior,
rejection, invalid sessions, malformed responses, and transport/timeout errors.

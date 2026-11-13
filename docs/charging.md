# Charging and cable lock

Build `Command::control_charging(stop_charging)` or
`Command::control_charging_port_lock(unlock)` and pass it to
`SaicClient::execute_command(vin, &command, options)`.

| Builder argument | Action | Charging field | Port-lock field |
| --- | --- | --- | --- |
| `stop_charging: false` | Start charging | 1 | 0 |
| `stop_charging: true` | Stop charging | 2 | 0 |
| `unlock: false` | Lock cable | 0 | 1 |
| `unlock: true` | Unlock cable | 0 | 2 |

Both builders set V2X to zero (no action). Requests use
`POST /vehicle/charging/control`. Stopping does not unlock the cable; unlocking
does not stop charging. Callers may submit those commands independently and
choose when to refresh. No command reads status before, after or on failure.
Only continuation of the same command with its server event ID is allowed.

`CommandOutcome` preserves the result data, server message, code, HTTP status and
event ID, including rejected and pending response details. Response fields such
as `bmsChrgSts` and `bmsChrgSpRsn`, plus unknown fields, remain available in `data`.
Completion means API completion, not verified charging or cable state. A timeout
may leave an unknown outcome; it must not cause an automatic resubmission.

The interactive commands are `charging-start VIN`, `charging-stop VIN`,
`cable-lock VIN` and `cable-unlock VIN`.

## Sources and offline verification

Payloads follow `control_charging` and `control_charging_port_lock` in
[the pinned Python charging API](https://github.com/SAIC-iSmart-API/saic-python-client-ng/blob/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/vehicle_charging/__init__.py).
Gateway `ec814eca135792d8202dfd29d6e443b53746d2a1` handlers
`drivetrain_charging.py` and `drivetrain_charging_cable_lock.py` confirm boolean
direction. The local `src/saic_api.py` changes concern seats only. Gateway refresh
and cached-state updates belong to the application and are not ported here.

Synthetic fixtures cover all four payloads and completed, rejected and pending
responses. Offline exchanges verify exact payloads, same-command continuation,
retained response details and no secondary controls or status reads, including
transport failures and timeouts. CLI specs cover dispatch and invalid arguments.

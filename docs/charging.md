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

## Target SoC and current limit

Use `Command::set_target_battery_soc(TargetBatterySoc, supported)` or
`Command::set_charge_current_limit(ChargeCurrentLimit, supported)` with
`execute_command`. Supply `supported` from the caller's model capability
information for that setting. False rejects locally; unknown support must not
be treated as true. The client does not discover capabilities or fetch existing
settings. These constructors do not imply that every model supports every value.

| Target percentage | Protocol code |
| --- | --- |
| 40 | 1 |
| 50 | 2 |
| 60 | 3 |
| 70 | 4 |
| 80 | 5 |
| 90 | 6 |
| 100 | 7 |

`TargetBatterySoc::from_percentage` rejects all other percentages, without
rounding. Parse `ChargeCurrentLimit` from `6A`, `8A`, `16A` or `MAX`
(case-insensitive); their protocol codes are 1, 2, 3 and 4 respectively.
`MAX` means the vehicle maximum, not a fixed amperage.

Both commands POST to `/vehicle/charging/setting`. Target updates set
`onBdChrgTrgtSOCReq` and send `altngChrgCrntReq: 0`; current updates set
`altngChrgCrntReq` and send `onBdChrgTrgtSOCReq: 0`. Both send
`tboxV2XSpSOCReq: 0`. Zero is the pinned client's **ignore** code: it preserves
that setting rather than replacing it with a default. No companion value is
required for these supported partial updates. To change both settings, submit
the two commands independently and choose when to refresh. All-ignore payloads
are settings reads and are rejected by command transport.

CLI examples (synthetic VIN; do not execute against production as a test):

```text
charge-target SYNTHETICVIN 80 supported
charge-current SYNTHETICVIN 8A supported
```

The final argument is mandatory caller-supplied capability information:
`supported` or `unsupported`. Unsupported settings reject without sending.
Neither command reads status before, after or on failure. Continuation uses
only the same settings payload and event ID. Results have the same completion
and unknown-outcome semantics as other commands.

Mappings and ignore semantics follow `TargetBatteryCode`,
`ChargeCurrentLimitCode` and `set_target_battery_soc` in the pinned Python
client linked above. Gateway handlers `drivetrain_soc_target.py` and
`drivetrain_chargecurrent_limit.py` at the recorded gateway revision confirm
mapping and target capability checks. Cached companion values and refreshes
from the gateway are application behavior; the supported ignore codes let this
client preserve companions without cached state.

`charge_settings.json` covers every supported mapping, unchanged companion
fields and invalid user values. Offline transport specs exercise each payload
through completion, rejection, continuation, transport failure and timeout;
CLI specs check capability rejection, required arguments and absence of reads.

## Charging schedules

Use `Command::set_schedule_charging(start, end, mode, supports_target_soc)` and
`execute_command` to configure the vehicle's own charging schedule. Parse each
time as `ScheduleTime` (`HH:MM`), or construct it with `ScheduleTime::new(hour,
minute)`. Hours are 0–23 and minutes 0–59. Seconds, dates and timezone offsets
are not accepted. The wire format contains only hours and minutes: supply the
wall-clock times expected by the vehicle. The client performs no host timezone,
UTC or daylight-saving conversion and creates no host scheduler.

| `ScheduledChargingMode` | CLI mode | Protocol code |
| --- | --- | --- |
| `Disabled` | `disabled` | 2 |
| `UntilConfiguredTime` | `until-configured-time` | 1 |
| `UntilConfiguredSoc` | `until-configured-soc` | 3 |

Both start and end must be supplied in every mode, including disabled and
until-configured-SoC; the client never fetches existing values. A start later
than the end (for example 22:30–06:15) represents an overnight window and is
sent unchanged. Midnight (00:00) is valid; 24:00 is not. Equal bounds also pass
through, matching the pinned client; their physical interpretation belongs to
the vehicle and is not assumed to mean either a full day or an empty window.
Until-configured-SoC uses the vehicle's already configured target, without
changing or reading it. The end time is still transmitted in that mode.

`supports_target_soc` is explicit caller-supplied capability information, not a
request to discover capabilities. Unknown support must be false. Selecting
`UntilConfiguredSoc` with false returns `Error::Unsupported` before any request.
Disabled and until-configured-time do not require target SoC support. This
matches the gateway restriction, but returns an error instead of silently
ignoring the request.

```text
charge-schedule SYNTHETICVIN 22:30 06:15 until-configured-time unsupported
charge-schedule SYNTHETICVIN 00:00 06:00 until-configured-soc supported
charge-schedule SYNTHETICVIN 22:30 06:15 disabled unsupported
```

The last argument declares target SoC support. The command posts to
`/vehicle/charging/reservation`, with `rsvanStHour`, `rsvanStMintue`,
`rsvanSpHour`, `rsvanSpMintue`, `tboxAdpPubChrgSttnReq: 1` and
`tboxReserCtrlReq` set to the mode code. The `Mintue` spellings are protocol
field names. Capability information is never sent to SAIC. Raw
`Command::ChargingSchedule` payloads also validate times, mode, marker and
capability before transport.

No schedule command refreshes vehicle or charging status before, after or on
failure. Only same-command event-ID continuation is allowed. Completion means
API completion, not a verified schedule or charging state; unknown outcomes
must not cause automatic replay.

Sources: `set_schedule_charging`, `ScheduledChargingRequest` and
`ScheduledChargingMode` in the pinned Python charging API linked above;
gateway `drivetrain_charging_schedule.py` at
`ec814eca135792d8202dfd29d6e443b53746d2a1`. The recorded local `src/saic_api.py`
changes add no schedule override. Gateway cached-state updates and refreshes
remain application responsibilities. Synthetic `charge_schedule.json` fixtures
cover every mode, midnight, overnight and equal bounds, plus invalid times.
Offline exchanges and CLI specs verify payloads, failures and no implicit reads.

## Battery heating

Use `Command::control_battery_heating(enable)` to switch heating on or off.
It sends `ptcHeatReq: 1` or `2` to `POST /vehicle/charging/ptcHeat`.
Only continuation of the same server event is allowed. The outcome retains
server response fields, including heating stop reasons; completion does not
verify the heater's physical state.

Use `Command::enable_schedule_battery_heating(start, &timezone, now)` with a
validated `ScheduleTime`, a chrono timezone (IANA `chrono_tz::Tz` or
`chrono::FixedOffset`), and an explicit `DateTime<Utc>` clock value. No account
lookup or system-local timezone fallback occurs. The conversion matches the
pinned Python client's `enable_schedule_battery_heating`: replace today's local
hour and minute, zero seconds, and advance one local calendar day if that wall
time has passed. Equality stays today; seconds after that minute advance tomorrow.
The payload contains UTC epoch milliseconds in `startTime` and `status: 1`.

Named zones apply the offset for the selected date. During a repeated autumn
hour, today's occurrence retains the current fold; advancing a day resets to the
first occurrence. Nonexistent spring times use the offset before the gap, as
Python `zoneinfo` does. This intentionally preserves reference behavior rather
than rejecting DST gaps. Fixed offsets never apply DST.

`Command::disable_schedule_battery_heating()` sends `startTime: 0, status: 0`;
it needs no start time, timezone, existing schedule, or status read. Both schedule
commands use a single `POST /charging/batteryHeating`, without event continuation
or retries, even on failure or an unexpected pending response. Submit all three
builders through `execute_command`.

`SaicClient::get_user_timezone()` explicitly reads `GET /user/timezone` without
a VIN. `UserTimezoneResp.timezone` preserves missing, null and unknown names;
callers choose how to parse it. Commands never invoke this method themselves.

CLI commands are `battery-heat VIN on|off`,
`battery-heat-enable VIN HH:MM TIMEZONE`, `battery-heat-disable VIN`, and
`user-timezone`. Enable accepts an IANA name such as `Europe/Brussels` or a fixed
offset such as `+05:30`, and uses the current UTC clock. Like other reads, the CLI
redacts account timezone data; library callers can access the response fields.

Sources are the pinned charging and user APIs and gateway
`drivetrain_battery_heating.py` / `drivetrain_battery_heating_schedule.py` at the
revision above. Gateway refresh and cached-state suppression remain application
responsibilities. Synthetic Python datetime fixtures exercise offsets, midnight,
DST gaps (including a half-hour transition), both autumn folds, and invalid input.
Offline transport and CLI specs verify independent dispatch and failure behavior.

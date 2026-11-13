# Command coverage audit

Reviewed 13 November 2026. This is source coverage, not verified vehicle support.
No production calls were made. All 24 registered gateway handlers are mapped
below: 17 vehicle handlers and seven application state/polling handlers.

## Evidence

- Primary: local `../saic-python-mqtt-gateway`, branch `fix/single-seat-heating`,
  HEAD `ec814eca135792d8202dfd29d6e443b53746d2a1`. Registration is in
  `src/handlers/command/__init__.py` and each category's `__init__.py`.
- Include the local `src/saic_api.py` override, SHA-256
  `2d70af475dbe618f7450acdd57c6c84bc8f9490f31a73a8ee0c9a275c1d89e09`:
  omitted seat arguments are omitted from the payload, not defaulted to zero.
- [Current Python API](https://github.com/SAIC-iSmart-API/saic-python-client-ng/tree/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api)
  is pinned to `5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e`.
- [Legacy API](https://github.com/SAIC-iSmart-API/saic-python-client/blob/690506138497d3dc519aef4153f1b7a24d3f2156/src/saic_ismart_client/saic_api.py)
  is comparison only, pinned to `690506138497d3dc519aef4153f1b7a24d3f2156`.

## Registered gateway handlers

Paths are relative to `src/handlers/command/`. Rust controls are `Command`
builders submitted with `SaicClient::execute_command`. Gateway refresh results,
cached-value suppression and state updates belong to the application; Rust
submits explicit intent without fetching state or suppressing unchanged values.

| Handler (source module) | Rust mapping | CLI |
| --- | --- | --- |
| ClimateBackWindowHeatCommand (`climate/climate_back_window_heat.py`) | `set_rear_window_heater` | `rear-window-heat` |
| ClimateSteeringWheelHeatCommand (`climate/climate_steering_wheel_heat.py`) | `set_steering_wheel_heater` | `steering-wheel-heat` |
| ClimateFrontWindowHeatCommand (`climate/climate_front_window_heat.py`) | `start_front_defrost`, `stop_front_defrost` (stops AC) | `defrost-start`, `defrost-stop` |
| ClimateRemoteClimateStateCommand (`climate/climate_remote_climate_state.py`) | `start_ac`, `stop_ac`, `start_ac_blowing`, `start_front_defrost` | `ac-start`, `ac-stop`, `ac-blowing`, `defrost-start` |
| ClimateRemoteTemperatureCommand (`climate/climate_remote_temperature.py`) | `set_ac_temperature`; explicit profile and temperature | `ac-temperature` |
| ClimateHeatedSeatsFrontLeftLevelCommand (`climate/climate_heated_seats_level.py`) | `control_heated_seat(FrontLeft, level)` | `seat-heat VIN front-left LEVEL` |
| ClimateHeatedSeatsFrontRightLevelCommand (same module) | `control_heated_seat(FrontRight, level)` | `seat-heat VIN front-right LEVEL` |
| DoorsBootCommand (`doors/doors_boot.py`) | `open_tailgate`; gateway true/close is unsupported | `tailgate-open` |
| DoorsLockedCommand (`doors/doors_locked.py`) | `lock_vehicle`, `unlock_vehicle` | `lock`, `unlock` |
| DrivetrainBatteryHeatingCommand (`drivetrain/drivetrain_battery_heating.py`) | `control_battery_heating` | `battery-heat` |
| DrivetrainBatteryHeatingScheduleCommand (`drivetrain/drivetrain_battery_heating_schedule.py`) | `enable_schedule_battery_heating`, `disable_schedule_battery_heating`; explicit timezone | `battery-heat-enable`, `battery-heat-disable` |
| DrivetrainChargeCurrentLimitCommand (`drivetrain/drivetrain_chargecurrent_limit.py`) | `set_charge_current_limit`; zero sentinel preserves target | `charge-current` |
| DrivetrainChargingCommand (`drivetrain/drivetrain_charging.py`) | `control_charging` | `charging-start`, `charging-stop` |
| DrivetrainHVBatteryActiveCommand (`drivetrain/drivetrain_hv_battery_active.py`) | Application state: updates HV activity, no vehicle command | None |
| DrivetrainSoCTargetCommand (`drivetrain/drivetrain_soc_target.py`) | `set_target_battery_soc`; zero sentinel preserves current limit | `charge-target` |
| DrivetrainTotalBatteryCapacitySetCommand (`drivetrain/drivetrain_total_battery_capacity.py`) | Application state: capacity override, no vehicle command | None |
| DrivetrainChargingScheduleCommand (`drivetrain/drivetrain_charging_schedule.py`) | `set_schedule_charging` | `charge-schedule` |
| DrivetrainChargingCableLockCommand (`drivetrain/drivetrain_charging_cable_lock.py`) | `control_charging_port_lock` | `cable-lock`, `cable-unlock` |
| RefreshModeCommand (`gateway/refresh_mode.py`) | Application polling mode; explicit reads remain caller-owned | No mode command; `vehicle-live` for explicit refresh |
| RefreshPeriodActiveCommand (`gateway/refresh_period.py`) | Application polling interval | None |
| RefreshPeriodInactiveCommand (same module) | Application polling interval | None |
| RefreshPeriodInactiveGraceCommand (same module) | Application polling grace period | None |
| RefreshPeriodAfterShutdownCommand (same module) | Application polling after shutdown | None |
| LocationFindMyCarCommand (`location/location_find_my_car.py`) | `find_my_car` with explicit mode | `find-car` (defaults to lights only) |

No supported registered gateway vehicle action remains unmapped. The Rust CLI
requires intent and capability arguments instead of reading gateway state.

## Current Python extras and read mappings

Paths below are relative to the pinned current Python `api/` directory. Shared
transport helpers map to Rust envelopes rather than separate public aliases.

| Python source and operations | Rust mapping | CLI |
| --- | --- | --- |
| `vehicle/__init__.py`: `vehicle_list`, `get_vehicle_status`, `get_vehicle_status_cached` | `vehicles`, `vehicle_status_live`, `vehicle_status_cached` | `vehicles`, `vehicle-live`, `vehicle-cache` |
| Same: `send_vehicle_control_command`, `control_find_my_car` | `execute_command(Command::Vehicle)`, `find_my_car` | `vehicle-command`, `find-car` |
| `vehicle/climate/__init__.py`: `start_ac`, `stop_ac`, `start_ac_blowing`, `start_front_defrost`, `control_climate` | Same-named builders | `ac-start`, `ac-stop`, `ac-blowing`, `defrost-start`, `climate-control` |
| Same: `control_heated_seat` (four positions) | `control_heated_seat`, `HeatedSeat`; rear restricted to 0/3 | `seat-heat` |
| Same: plural `control_heated_seats` | Two independent single-seat commands; unsafe default-both-off behavior omitted | Two `seat-heat` commands |
| Same: `control_heated_steering_wheel`, `control_rear_window_heat` | `set_steering_wheel_heater`, `set_rear_window_heater` | `steering-wheel-heat`, `rear-window-heat` |
| `vehicle/locks/__init__.py`: `lock_vehicle`, `unlock_vehicle`, `open_tailgate`, `control_tailgate`, `control_vehicle_locks` | Same-named builders | `lock`, `unlock`, `tailgate-open`, `tailgate-control`, `locks-control` |
| `vehicle/windows/__init__.py`: `close_driver_window`, `control_sunroof`, `control_windows`, `control_door_windows` | Same-named builders | `driver-window-close`, `sunroof-control`, `windows-control`, `door-windows-control` |
| `vehicle_charging/__init__.py`: `get_vehicle_charging_status`, `get_vehicle_charging_management_data`, `get_vehicle_charging_settings` | `charging_status`, `charging_management`, `charging_settings` | `charging-status`, `charging-management`, `charging-settings` |
| Same: `send_vehicle_charging_control`, `control_charging`, `control_charging_port_lock`, `control_v2x` | `Command::Charging` and same-named builders | `charging-command`, `charging-start/stop`, `cable-lock/unlock`, `v2x-start/stop` |
| Same: `send_vehicle_charging_settings`, `set_target_battery_soc`, `set_v2x_target_battery_soc` | `Command::ChargingSettings`, `set_target_battery_soc`, `set_charge_current_limit`, `set_v2x_target_battery_soc` | `charge-target`, `charge-current`, `v2x-target` |
| Same: `send_vehicle_charging_reservation`, `set_schedule_charging` | `Command::ChargingSchedule`, `set_schedule_charging` | `charge-schedule` |
| Same: `send_vehicle_charging_ptc_heat`, `control_battery_heating` | `Command::BatteryHeating`, `control_battery_heating` | `battery-heat` |
| Same: `get_vehicle_battery_heating_schedule`, `send_vehicle_battery_heating_schedule`, `enable_schedule_battery_heating`, `disable_schedule_battery_heating` | `battery_heating_schedule`, schedule builders; no implicit timezone read | `battery-heating-schedule`, `battery-heat-enable/disable` |
| `vehicle/alarm/__init__.py`: `get_alarm_switch`, `set_alarm_switches` | Same-named read/builder | `alarm-switches`, `alarm-switches-set` |
| `message/__init__.py`: `get_message_list`, `get_alarm_list`, `get_command_list`, `get_news_list`, `get_unread_messages_count` | Same-named reads | `messages`, `alarm-list`, `command-list`, `news-list`, `messages-unread` |
| Same: `read_message`, `delete_message`, `delete_all_alarms`, `delete_all_commands`, `delete_all_news`, `update_message_status` | `update_message_status(MessageAction)` with explicit group/ID | `message-status read/delete/delete-all` |
| `user/__init__.py`: `get_user_timezone` | `get_user_timezone` | `user-timezone` |
| `base.py`: login and event continuation | `login`, bounded same-command continuation | `login`; no automatic relogin or command replay |

## Deliberate omissions and support questions

Legacy `saic_api.py::unknown_engine_control` sends request type `0x11` with
parameter 16=1, despite its engine-related name. There is no matching
current API method or registered gateway handler. Omit this name and do not infer
engine start/stop behavior from it; its name does not establish supported engine behavior.

Legacy `open_door_locks` duplicates unlock intent; `start_rear_window_heat` and
`stop_rear_window_heat` become one boolean heater builder. `find_my_car` maps to
the current explicit modes. `start_charging` becomes `control_charging(false)`.
Legacy `*_with_retry`, `handle_retry` and ASN.1 transport helpers are omitted:
current event continuation is bounded and never resubmits a new command.
Legacy `set_geofence_alarm_switch` maps to an explicit geofence `AlarmSwitch`,
not a hard-coded account mutation. `stop_front_defrost` remains a documented
stop-AC convenience, not a distinct engine or windshield action.

The gateway steering-wheel payload omits marker 255; current Python includes it.
Rust deliberately follows the primary gateway (type 8, parameter 24 only).
Whether all models require/accept the marker remains unresolved. Rear seat
availability, window actions, powered tailgate, V2X, target SoC and schedule
support are model-dependent and unverified on vehicles. Physical left/right
seat positions do not imply driver/passenger on every market. Rear-seat levels
1/2 are rejected in the typed builder: Python documents rear seats as on/off
with 3 for on, though its validator accepts 0–3. See the operation-specific docs
for further limitations. API completion never proves physical state.

## Offline regression contract

`src/client_specs.rs` exercises real request construction and bounded transport
through injected fake exchanges. A mixed control sequence, including two separate
seat requests, success, pending continuation and failure, is followed by exactly
one caller-issued live read. Every control exchange asserts its expected control
endpoint; zero vehicle/charging status reads occur during commands. Existing
endpoint-specific specs cover rejection, invalid session, malformed response,
transport errors, timeouts and cancellation.

`examples/cli/specs.rs` exercises command parsing/dispatch with a fake session,
including failed commands followed by one explicit refresh, seat payload isolation,
invalid levels and help/README command-name parity. Tests use synthetic data and
never load `.env`. The protocol fake is not a network mock server.

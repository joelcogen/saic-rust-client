# Vehicle discovery and reads

All reads require an explicit login. They never log in automatically, send a
control, or start background polling. The caller chooses when to refresh after
controls. Each operation returns its own typed response from `saic_client::models`.

| Method | Response | Protocol |
| --- | --- | --- |
| `vehicles()` | `VehicleListResp` | GET `/vehicle/list` |
| `vehicle_status_cached(vin, options)` | `VehicleStatusCacheResp` | GET `/vehicle/status/cache` |
| `vehicle_status_live(vin, options)` | `VehicleStatusResp` | GET `/vehicle/status` with event continuation |
| `charging_status(vin, options)` | `ChargeStatusResp` | GET `/vehicle/charging/status` with event continuation |
| `charging_management(vin, options)` | `ChrgMgmtDataResp` | GET `/vehicle/charging/mgmtData` with event continuation |
| `charging_settings(vin, options)` | `ChargingSettingResp` | POST `/vehicle/charging/setting` with event continuation |
| `battery_heating_schedule(vin, options)` | `ScheduledBatteryHeatingResp` | GET `/charging/batteryHeating` |

Pass the original VIN; the library hashes it internally. Both vehicle status
requests include `vehStatusReqType=2`. GET signatures include the complete query
string and have no encrypted body. Charging settings use the reference's read
payload: `altngChrgCrntReq`, `onBdChrgTrgtSOCReq`, and `tboxV2XSpSOCReq` are all
zero (ignore). No existing settings are fetched or supplied to this request.

`options` is `CommandOptions`, with the same 30-second overall deadline,
10-second request timeout and one-second continuation delay as command transport.
Discovery uses those defaults. Single-request operations never continue a pending
response. In particular, an empty, stale, malformed or failed cache response
never falls back to live status. An empty cache object yields missing fields.
Live vehicle and charging reads may wake the vehicle.

Event reads continue only the same endpoint, query/body and server event ID.
Changed or invalid IDs, transport failures and timeouts stop the read without
restarting it. Server rejection and authentication errors use `Error::Rejected`
and `Error::Authentication`; authentication errors clear the local session.
Dropping a read future stops waiting without cancelling the server operation.
The HTTP timeout and 1 MiB response limit also apply.

## Data and capabilities

Fields use Rust snake_case names and preserve the original wire representation.
Numeric measurements retain protocol scaling; they are not automatically
converted to volts, amps or degrees. Numeric enum/status codes remain integers,
including unknown codes. No unknown value is interpreted as off or zero.
`ProtocolValue` retains fields that can be either integers or encoded strings.
Missing or null fields are `None`, including absent collections. Unknown fields
are retained in each object's `extra` map.

`status_time` on vehicle and charging status retains epoch seconds.
The battery-heating schedule's `start_time` retains UTC epoch milliseconds;
no local-time conversion is performed. Other timestamp fields remain unmodified.
Cached `online_status` is the server flag, not a reliable indication that the
vehicle is awake.

Discovery includes `vehicle_model_configuration`. `VinInfo::capability(code)`
looks up a raw declaration without assuming support when it is absent. It returns
the declaration even if its value is missing or unknown. For example, the gateway
uses `HeatedSeat` values `1` and `2` for level and on/off seats, and `BType=1` for
target SOC support. The library does not extrapolate these to all models or infer
capabilities from a missing status field. Callers must handle unknown support.

Debug output redacts all response data. Fields and explicit Serde serialization
remain available to applications; treat identifiers, locations and account data
as private. The CLI reports the returned response type with data redacted.

## Sources and offline checks

Behavior follows the local gateway at
`ec814eca135792d8202dfd29d6e443b53746d2a1`, including its local `src/saic_api.py`
changes, and the pinned Python client at
`5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e` (`api/vehicle/`,
`api/vehicle_charging/`, `api/schema.py`, `net/crypto.py`). See
[reference sources](references.md) and [third-party notices](../THIRD_PARTY_NOTICES.md).

Synthetic offline specs cover all seven operations, VIN hashing, settings ignore
values, GET query signing, missing data, unknown codes/capabilities, multiple
vehicles, cache failures without fallback, bounded continuation, and CLI parsing
and dispatch. Command transport regression tests ensure controls never fetch
status, including on failure. No SAIC production checks were run.

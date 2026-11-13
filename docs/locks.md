# Door locks and tailgate

Build a `Command`, then submit it with `SaicClient::execute_command` and an
explicit VIN. No control reads vehicle or charging status, including on failure.
Only continuation of the same command event is automatic. The caller chooses
whether and when to refresh.

| Builder | Request |
| --- | --- |
| `Command::lock_vehicle()` | Lock vehicle; type 1, null parameters |
| `Command::unlock_vehicle()` | Unlock doors; type 2, lock ID 3 |
| `Command::open_tailgate()` | Release tailgate latch; type 2, lock ID 2 |
| `Command::control_vehicle_locks(action, target)` | Explicit `LockAction` and optional `VehicleLockId` |
| `Command::control_tailgate(action)` | Powered open or close; type 9, parameter 31 = 1 or 2, no marker |

The generic `Command::Vehicle` stores locking parameters as an empty vector;
submission encodes that vector as JSON null for request type 1 to match the
reference payload.

`LockAction::Lock` requires no target. `Unlock` requires `Some(Doors)` or
`Some(Tailgate)`. Other combinations fail locally. Targeted locking is rejected
instead of silently ignoring the target as Python does. `TailgateAction` exposes
only `Open` and `Close`; there is no stop/toggle or inferred action.

Latch release does not request powered opening. Powered movement requires model
support: the reference reports testing only on MG S6 EV (UK). These methods do
not discover capabilities or promise support on other models. There is no
separate tailgate-lock operation.

`CommandState::Completed` means API completion, not proof that doors locked or
that the tailgate reached a physical position. Unknown outcomes must not be
blindly replayed. The CLI reports API results with physical position unverified.

## Sources and offline coverage

Primary behavior: gateway `../saic-python-mqtt-gateway`, branch
`fix/single-seat-heating`, commit `ec814eca135792d8202dfd29d6e443b53746d2a1`,
`src/handlers/command/doors/doors_locked.py` and `doors_boot.py`. Its local
`src/saic_api.py` changes affect seats only. Gateway refresh behavior is not
ported: callers own refresh timing.

Payloads and supported enums follow the current
[Python locks API](https://github.com/SAIC-iSmart-API/saic-python-client-ng/tree/5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e/src/saic_ismart_client_ng/api/vehicle/locks),
including `schema.py`, plus `api/vehicle/schema.py` and `api/base.py` (null
parameters are retained by dataclass serialization).
The supplied legacy reference, `saic-python-client` revision
`690506138497d3dc519aef4153f1b7a24d3f2156`, is historical only; no legacy
transport or operations are added.

Synthetic fixtures distinguish locking, door unlocking, latch release, powered
opening and powered closing. Offline specs cover invalid combinations, CLI
routing, and same-event continuation on success and failure without status reads.
No production checks were performed.

"""Synthetic vectors ported from Python client 5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e.

api/vehicle_charging/__init__.py: control_charging, control_charging_port_lock.
Gateway ec814eca135792d8202dfd29d6e443b53746d2a1 confirms boolean direction.
Response values are synthetic, not captures or assumptions about physical state.
"""
import json
from pathlib import Path

cases = []
for cli, charging, lock in [
    ("charging-start", 1, 0), ("charging-stop", 2, 0),
    ("cable-lock", 0, 1), ("cable-unlock", 0, 2),
]:
    cases.append({"cli": cli, "payload": {
        "chrgCtrlReq": charging, "tboxEleccLckCtrlReq": lock, "tboxV2XReq": 0,
    }})
responses = {
    "completed": {"code": 0, "message": "synthetic completion", "data": {
        "bmsChrgSts": 1, "bmsChrgSpRsn": 4, "futureField": {"value": 12}}},
    "rejected": {"code": 7, "message": "synthetic rejection", "data": {"reason": 42}},
    "pending": {"code": 1, "message": "synthetic pending", "data": {"progress": 5}},
}
Path(__file__).with_name("charging.json").write_text(
    json.dumps({"actions": cases, "responses": responses}, indent=2) + "\n")

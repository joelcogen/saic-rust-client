"""Offline synthetic vectors; no vehicle data or production calls.
Client 5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e: api/vehicle_charging/
__init__.py control_v2x / set_v2x_target_battery_soc and schema.TargetBatteryCode.
Gateway ec814eca135792d8202dfd29d6e443b53746d2a1 has no V2X handler.
"""
import json
from pathlib import Path

cases = []
for name, code in [("v2x-start", 1), ("v2x-stop", 2)]:
    cases.append({"cli": name, "args": "supported", "payload": {
        "chrgCtrlReq": 0, "tboxEleccLckCtrlReq": 0, "tboxV2XReq": code}})
for percentage, code in [(40, 1), (50, 2), (60, 3), (70, 4), (80, 5), (90, 6), (100, 7)]:
    cases.append({"cli": "v2x-target", "value": percentage,
                  "args": f"{percentage} supported", "payload": {
        "onBdChrgTrgtSOCReq": 0, "altngChrgCrntReq": 0, "tboxV2XSpSOCReq": code}})
Path(__file__).with_name("v2x.json").write_text(json.dumps({
    "actions": cases,
    "invalid_targets": [-32768, -1, 0, 1, 7, 39, 41, 55, 99, 101, 255, 32767],
}, indent=2) + "\n")

"""Offline synthetic vectors from client 5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e.
api/vehicle_charging/schema.py enums and __init__.py set_target_battery_soc.
Zero is IGNORE, not a default setting. No production data.
"""
import json
from pathlib import Path

cases = []
for percentage, code in [(40, 1), (50, 2), (60, 3), (70, 4), (80, 5), (90, 6), (100, 7)]:
    cases.append({"cli": "charge-target", "value": str(percentage), "payload": {
        "onBdChrgTrgtSOCReq": code, "altngChrgCrntReq": 0, "tboxV2XSpSOCReq": 0}})
for limit, code in [("6A", 1), ("8A", 2), ("16A", 3), ("MAX", 4)]:
    cases.append({"cli": "charge-current", "value": limit, "payload": {
        "onBdChrgTrgtSOCReq": 0, "altngChrgCrntReq": code, "tboxV2XSpSOCReq": 0}})
Path(__file__).with_name("charge_settings.json").write_text(json.dumps({
    "actions": cases,
    "invalid_targets": [-1, 0, 39, 41, 55, 99, 101, 255],
    "invalid_limits": ["", "0", "6", "7A", "10A", "32A", "IGNORE"],
}, indent=2) + "\n")

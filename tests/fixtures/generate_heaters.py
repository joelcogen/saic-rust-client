"""Synthetic heater vectors; standard library only, no network or accounts.

Rear window: Python client 5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e,
api/vehicle/climate/__init__.py and api/vehicle/schema.py.
Steering wheel: gateway ec814eca135792d8202dfd29d6e443b53746d2a1,
src/handlers/command/climate/climate_steering_wheel_heat.py (no marker).
"""
import base64
import json
from pathlib import Path

vectors = []
for heater, request_type, parameter_id, marker in [
    ("rear-window", "32", 23, True),
    ("steering-wheel", "8", 24, False),
]:
    for enabled in [False, True]:
        params = [{"paramId": parameter_id,
                   "paramValue": base64.b64encode(bytes([enabled])).decode()}]
        if marker:
            params.append({"paramId": 255,
                           "paramValue": base64.b64encode(bytes(4)).decode()})
        vectors.append({"heater": heater, "enabled": enabled, "payload": {
            "rvcReqType": request_type, "rvcParams": params}})
Path(__file__).with_name("heaters.json").write_text(json.dumps(vectors, indent=2) + "\n")

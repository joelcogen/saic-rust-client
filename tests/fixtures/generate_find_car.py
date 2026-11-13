"""Offline vectors from Python client 5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e.

api/vehicle/__init__.py control_find_my_car and api/vehicle/schema.py.
Gateway ec814eca135792d8202dfd29d6e443b53746d2a1 supplies the four modes.
"""
import base64
import json
from pathlib import Path

vectors = []
for mode, stop, horn, lights in [
    ("lights-only", False, False, True),
    ("horn-only", False, True, False),
    ("horn-and-lights", False, True, True),
    ("stop", True, False, False),
]:
    values = [(1, bytes([not stop])), (2, bytes([horn])),
              (3, bytes([lights])), (255, bytes(4))]
    vectors.append({"mode": mode, "payload": {"rvcReqType": "0", "rvcParams": [
        {"paramId": i, "paramValue": base64.b64encode(value).decode()}
        for i, value in values]}})
Path(__file__).with_name("find_car.json").write_text(json.dumps(vectors, indent=2) + "\n")

"""Offline synthetic vectors from Python client 5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e.

api/vehicle/locks/{__init__,schema}.py and api/vehicle/schema.py.
Gateway ec814eca135792d8202dfd29d6e443b53746d2a1 uses the first three operations.
"""
import base64
import json
from pathlib import Path


def param(number, data):
    return {"paramId": number, "paramValue": base64.b64encode(data).decode()}


vectors = [{"cli": "lock", "payload": {"rvcReqType": "1", "rvcParams": None}}]
for cli, target in [("unlock", 3), ("tailgate-open", 2)]:
    vectors.append({"cli": cli, "payload": {"rvcReqType": "2", "rvcParams": [
        *[param(i, bytes(1)) for i in (4, 5, 6)],
        param(7, bytes([target])), param(255, bytes(4))]}})
for action, value in [("open", 1), ("close", 2)]:
    vectors.append({"cli": "tailgate-control", "action": action, "payload": {
        "rvcReqType": "9", "rvcParams": [param(31, bytes([value]))]}})
Path(__file__).with_name("locks.json").write_text(json.dumps(vectors, indent=2) + "\n")

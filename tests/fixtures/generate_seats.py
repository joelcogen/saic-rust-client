"""Synthetic single-seat payloads from gateway src/saic_api.py local changes.

Gateway base ec814eca135792d8202dfd29d6e443b53746d2a1, fix/single-seat-heating.
Uses standard-library encoding only; no accounts or network requests.
"""
import base64
import json
from pathlib import Path

vectors = []
for seat, parameter_id in [("left", 17), ("right", 18)]:
    for level in range(4):
        vectors.append({"seat": seat, "level": level, "payload": {
            "rvcReqType": "5", "rvcParams": [
                {"paramId": parameter_id,
                 "paramValue": base64.b64encode(bytes([level])).decode()},
                {"paramId": 255,
                 "paramValue": base64.b64encode(bytes(4)).decode()},
            ]}})
Path(__file__).with_name("seats.json").write_text(json.dumps(vectors, indent=2) + "\n")

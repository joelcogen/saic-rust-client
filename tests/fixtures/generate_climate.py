"""Synthetic vectors from pinned Python control_climate; no network or accounts.

Source: saic-python-client-ng 5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e,
api/vehicle/climate/__init__.py. See THIRD_PARTY_NOTICES.md.
"""
import base64
import json
from pathlib import Path


def payload(fan, ac, temperature):
    if fan == 0:
        ac = False
        temperature = 8
    params = [(19, fan)]
    if fan > 0 or temperature == 0:
        params.append((20, temperature))
    if ac is not None:
        params.append((22, int(ac)))
    return {"rvcReqType": "6", "rvcParams": [
        {"paramId": key, "paramValue": base64.b64encode(bytes([value])).decode()}
        for key, value in params
    ] + [{"paramId": 255, "paramValue": "AAAAAA=="}]}


cases = [(fan, ac, temp) for fan in [0, 1, 2, 5, 255]
         for ac in [None, False, True] for temp in [0, 2, 3, 8, 14, 17, 255]]
vectors = [{"args": args, "payload": payload(*args)} for args in cases]
# The Rust boundary rejects invalid bytes even when fan zero omits temperature.
for invalid in [-32768, -1, 256, 32767]:
    for args in [(invalid, None, 8), (0, None, invalid), (2, True, invalid)]:
        vectors.append({"args": args, "invalid": True})
Path(__file__).with_name("climate.json").write_text("[\n" + ",\n".join(
    json.dumps(vector) for vector in vectors
) + "\n]\n")

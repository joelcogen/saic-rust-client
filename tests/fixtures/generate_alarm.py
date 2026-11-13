"""Synthetic current JSON alarm vectors; no network or account data.
Source: client-ng 5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e,
api/vehicle/alarm/{__init__,schema}.py. Explicit zero flags extend the
upstream enable-only helper using the current schema's switch fields.
"""
import json
from pathlib import Path

cases = []
for name, code in [('vehicle-fault', 0), ('geofence', 2), ('vehicle-start', 3)]:
    for alarm in [0, 1]:
        for function in [0, 1]:
            cases.append({'args': f"{name} {['off', 'on'][alarm]} {['off', 'on'][function]}",
                          'switch': {'alarmType': code, 'alarmSwitch': alarm, 'functionSwitch': function}})
Path(__file__).with_name('alarm.json').write_text(json.dumps({
    'cases': cases,
    'response': {'alarmSwitchList': [c['switch'] for c in cases[::4]] + [
        {'alarmType': 99, 'alarmSwitch': 7, 'functionSwitch': None, 'future': 'synthetic'}, {}]},
    'invalid': [{'alarmType': 1, 'alarmSwitch': 1, 'functionSwitch': 1},
                {'alarmType': 2, 'alarmSwitch': 2, 'functionSwitch': 1},
                {'alarmType': 2, 'alarmSwitch': 1, 'functionSwitch': -1},
                {'alarmType': 2, 'alarmSwitch': 1}],
    'reply_codes': [0, 1, 500, 401],
}, indent=2) + '\n')

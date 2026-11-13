"""Synthetic offline window vectors, standard library only.

Payload rules: saic-python-client-ng 5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e,
api/vehicle/windows/{__init__,schema}.py and api/vehicle/schema.py.
Empty/duplicate selection rejection is a Rust API validation choice.
"""
import base64
import hashlib
import json
from pathlib import Path

NAMES = ['sunroof', 'driver', 'passenger', 'rear-left', 'rear-right']


def payload(indices, action):
    values = [int(i in indices) for i in range(5)] + [action]
    return {'rvcReqType': '3', 'rvcParams': [
        {'paramId': i + 8, 'paramValue': base64.b64encode(bytes([v])).decode()}
        for i, v in enumerate(values)]}


selections = []
for mask in range(1, 32):
    indices = [i for i in range(5) if mask & (1 << i)]
    for opened in [False, True]:
        selections.append({'indices': indices, 'open': opened,
            'cli': 'windows-control SYNTHETICVIN ' + ('open' if opened else 'close') + ' ' + ' '.join(NAMES[i] for i in indices),
            'payload': payload(indices, 3 if opened else 0)})
for indices in [[], *[[i, i] for i in range(5)]]:
    for opened in [False, True]:
        selections.append({'indices': indices, 'open': opened, 'invalid': True})
helpers = [
    {'cli': 'driver-window-close SYNTHETICVIN', 'payload': payload([1], 0)},
    *[{'cli': 'sunroof-control SYNTHETICVIN ' + action, 'payload': payload([0], value)}
      for action, value in [('close', 0), ('open', 3)]],
    *[{'cli': 'door-windows-control SYNTHETICVIN ' + action, 'payload': payload([1, 2, 3, 4], value)}
      for action, value in [('close', 0), ('ventilate', 1), ('open', 2)]],
]
invalid_cli = [
    'driver-window-close', 'driver-window-close vin open',
    'sunroof-control', 'sunroof-control vin', 'sunroof-control vin ventilate',
    'sunroof-control vin open extra', 'windows-control', 'windows-control vin open',
    'windows-control vin ventilate driver', 'windows-control vin open all',
    'windows-control vin close driver driver', 'windows-control vin open window-2',
    'door-windows-control', 'door-windows-control vin', 'door-windows-control vin 3',
    'door-windows-control vin open driver', 'door-windows-control vin ventilate sunroof',
]
Path(__file__).with_name('windows.json').write_text(json.dumps({
    'vin_hash': hashlib.sha256(b'SYNTHETICVIN').hexdigest(),
    'selections': selections, 'helpers': helpers, 'invalid_cli': invalid_cli,
}, indent=2) + '\n')

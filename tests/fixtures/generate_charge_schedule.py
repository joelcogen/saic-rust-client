"""Synthetic reservation vectors, no accounts or vehicle data.
Client 5ffd4e2bbb501d1afbfb272d1f546c0dd2030d0e set_schedule_charging/schema.
Gateway ec814eca135792d8202dfd29d6e443b53746d2a1 restricts mode 3 to target SoC support.
"""
import json
from pathlib import Path

cases = []
for mode, code in [('disabled', 2), ('until-configured-time', 1), ('until-configured-soc', 3)]:
    for start, end in [('00:00', '23:59'), ('23:59', '00:00'), ('22:30', '06:15'), ('00:00', '00:00')]:
        sh, sm = map(int, start.split(':'))
        eh, em = map(int, end.split(':'))
        cases.append({'cli': 'charge-schedule', 'start': start, 'end': end, 'mode': mode,
                      'payload': {'rsvanStHour': sh, 'rsvanStMintue': sm, 'rsvanSpHour': eh,
                                  'rsvanSpMintue': em, 'tboxAdpPubChrgSttnReq': 1, 'tboxReserCtrlReq': code}})
Path(__file__).with_name('charge_schedule.json').write_text(json.dumps({
    'actions': cases,
    'invalid_times': ['', '24:00', '00:60', '-1:00', '12:-1', '99:99', '1:00', '00:0', '12:30:00', '12:30Z', '12:30+01:00', 'ab:cd'],
}, indent=2) + '\n')

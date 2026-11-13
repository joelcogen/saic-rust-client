"""Offline synthetic vectors matching client 5ffd4e2's heating conversion.
Uses Python datetime/zoneinfo only; no accounts or network.
"""
import json
from datetime import datetime, timedelta, timezone
from pathlib import Path
from zoneinfo import ZoneInfo

cases = []
for zone, now, start in [
    ('UTC', '2026-01-01T06:00:00+00:00', '07:30'),
    ('UTC', '2026-01-01T07:30:00+00:00', '07:30'),
    ('UTC', '2026-01-01T07:30:01+00:00', '07:30'),
    ('+05:30', '2026-01-01T22:00:00+00:00', '03:00'),
    ('-07:00', '2026-01-01T01:00:00+00:00', '23:59'),
    ('Europe/Brussels', '2026-01-01T05:00:00+00:00', '07:30'),
    ('Europe/Brussels', '2026-07-01T05:00:00+00:00', '07:30'),
    ('Europe/Brussels', '2026-03-28T23:00:00+00:00', '02:30'),
    ('Europe/Brussels', '2026-03-28T20:00:00+00:00', '07:00'),
    ('Europe/Brussels', '2026-10-24T22:00:00+00:00', '02:30'),
    ('Europe/Brussels', '2026-10-25T00:45:00+00:00', '02:30'),
    ('Europe/Brussels', '2026-10-25T01:15:00+00:00', '02:30'),
    ('Europe/Brussels', '2026-10-25T01:45:00+00:00', '02:30'),
    ('Australia/Lord_Howe', '2026-10-03T14:00:00+00:00', '02:15'),
]:
    tz = timezone(timedelta(hours=5, minutes=30)) if zone == '+05:30' else timezone(timedelta(hours=-7)) if zone == '-07:00' else ZoneInfo(zone)
    local = datetime.fromisoformat(now).astimezone(tz)
    hour, minute = map(int, start.split(':'))
    date = local.replace(hour=hour, minute=minute, second=0, microsecond=0)
    if date < local:
        date += timedelta(days=1)
    cases.append(dict(zone=zone, now=now, start=start, payload=dict(startTime=int(date.timestamp())*1000, status=1)))
Path(__file__).with_name('battery_heating.json').write_text(json.dumps(dict(
    actions=[dict(cli='battery-heat', state='on', payload=dict(ptcHeatReq=1)), dict(cli='battery-heat', state='off', payload=dict(ptcHeatReq=2))],
    disabled=dict(startTime=0, status=0), schedules=cases,
    invalid_times=['24:00', '00:60', '7:00', '12:30:01', '-1:00', 'NaN'],
    invalid_zones=['Mars/Olympus', '+25:00', '+01:60', 'local'],
), indent=2)+'\n')

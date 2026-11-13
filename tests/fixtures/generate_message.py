"""Synthetic message vectors from client-ng 5ffd4e2, api/message. No network."""
import json
from pathlib import Path

lists = []
for group in ['ALARM', 'COMMAND', 'NEWS']:
    for page in [1, 2, 3]:
        lists.append({'group': group.lower(), 'page': page, 'size': 2,
                      'path': f'/message/list?pageNum={page}&pageSize=2&messageGroup={group}',
                      'response': {'messages': ([{'messageId': '001-synthetic', 'readStatus': 0,
                                                 'messageTime': '13/11/2026 12:00:00', 'vin': 'synthetic-vin'},
                                                {'messageId': 9007199254740993, 'readStatus': 9}] if page == 1
                                               else [{}] if page == 2 else []),
                                   'recordsNumber': 3, 'totalNumber': 3}})
actions = []
for action in ['READ', 'DELETE']:
    for mid in ['001-synthetic', 9007199254740993]:
        actions.append({'body': {'actionType': action, 'messageId': mid}})
for group in ['ALARM', 'COMMAND', 'NEWS']:
    actions.append({'body': {'actionType': 'DELETE_' + group}})
Path(__file__).with_name('message.json').write_text(json.dumps({
    'lists': lists, 'actions': actions,
    'unread': {'alarmNumber': 2, 'commandNumber': 3, 'newsNumber': 0, 'totalNumber': 5},
    'empty': [None, {}, {'messages': None}, {'messages': []}],
    'reply_codes': [0, 1, 500, 401]
}, indent=2) + '\n')

"""Synthetic GET signature vector using pinned net/crypto.py's algorithm.
No network or account data; Python standard library only.
"""
import hashlib
import hmac
import json
from pathlib import Path

path = '/vehicle/status/cache?vin=' + hashlib.sha256(b'SYNTHETICVIN00001').hexdigest() + '&vehStatusReqType=2'
timestamp = '1700000000123'
tenant = '459771'
token = 'synthetic-token'
md5 = lambda value: hashlib.md5(value.encode()).hexdigest()
prefix = path + tenant + token + 'app'
key = md5(md5(prefix) + timestamp + '1application/json')
verification = hmac.new(md5(key + timestamp).encode(), (prefix + timestamp + '1application/json').encode(), hashlib.sha256).hexdigest()
Path(__file__).with_name('read.json').write_text(json.dumps(dict(path=path, timestamp=timestamp, tenant=tenant, token=token, verification=verification), indent=2) + '\n')

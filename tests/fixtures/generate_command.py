import hashlib,hmac,json
from cryptography.hazmat.primitives.ciphers import Cipher,algorithms,modes
from cryptography.hazmat.primitives.padding import PKCS7
p='/vehicle/charging/control';ts='1700000000000';tenant='459771';token='synthetic-command-token';vin='SYNTHETICVIN00001';ct='application/json'
body=json.dumps({'chrgCtrlReq':1,'tboxEleccLckCtrlReq':0,'tboxV2XReq':0,'vin':hashlib.sha256(vin.encode()).hexdigest()},separators=(',',':'),sort_keys=True)
md5=lambda s:hashlib.md5(s.encode()).hexdigest()
prefix=p+tenant+token+'app';key=md5(md5(prefix)+ts+'1'+ct);iv=md5(ts)
pad=PKCS7(128).padder();padded=pad.update(body.encode())+pad.finalize();enc=Cipher(algorithms.AES(bytes.fromhex(key)),modes.CBC(bytes.fromhex(iv))).encryptor();cipher=(enc.update(padded)+enc.finalize()).hex()
mac=hmac.new(md5(key+ts).encode(),(prefix+ts+'1'+ct+cipher).encode(),hashlib.sha256).hexdigest()
json.dump(dict(path=p,timestamp=ts,tenant=tenant,token=token,vin=vin,plaintext=body,body=cipher,verification=mac),open('tests/fixtures/command.json','w'),indent=2)

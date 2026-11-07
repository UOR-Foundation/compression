# Validation oracle only; no production hashing implementation.
import json,re,hashlib,pathlib,sys
root=pathlib.Path(__file__).resolve().parent;out=pathlib.Path(sys.argv[1]) if len(sys.argv)>1 else root.parent/'client';out.mkdir(parents=True,exist_ok=True);rows=[]
def h(b):return hashlib.sha256(b).digest()
def digest(f,b):
 if f==3:return h(b'UORC-SHA256-STREAM-MAX-v1\0'+b'\0'+b+b'\1'+len(b).to_bytes(8,'big')).hex()
 d=b'UORC-SHA256-CHAIN-v1\0';s=h(d+b'\0');i=0
 while len(b)-i>=64:s=h(d+b'\1'+s+b[i:i+64]);i+=64
 t=b[i:];return h(d+b'\2'+s+bytes([len(t)])+t).hex()
def pref(f,p):
 if p=='archivePreimage':return b''
 if p=='inputBytes':return f'UORC-INPUT-BYTES-F{f}-M1-v1\0'.encode()
 if p=='basisObjects':return b'UORC-BASIS-V1\0' if f==2 else b'UORC-BASIS-F3-M1-v1\0'
 return f'UORC-CHECKPOINT-F{f}-M1-v1\0'.encode()
m=json.load(open(root/'binding-fixtures-model.json'))
for d in m['declarations']:
 if d['kind']=='definition' and d['body']['kind']=='record' and d['body']['type']['name']=='Fixture':
  f={v['field']:v['value']['value'] for v in d['body']['fields']};computed=digest(int(f['format']),pref(int(f['format']),f['purpose'])+bytes.fromhex(f['inputHex']));assert computed==f['expectedHex'];rows.append((f['name'],f['format'],f['purpose'],f['inputHex'],computed))
for fmt in [2,3]:
 for p in ['archivePreimage','inputBytes','basisObjects','checkpointBytes']:
  for n in [0,1,29,30,31,32,55,56,63,64,65,127,128,129,1000]:
   b=bytes((i*73+19)%256 for i in range(n));rows.append((f'boundary_f{fmt}_{p}_{n}',str(fmt),p,b.hex(),digest(fmt,pref(fmt,p)+b)))
for name in ['Short','Long']:
 text=(root/f'SHA256{name}Msg.rsp').read_text()
 for i,(bits,msg,md) in enumerate(re.findall(r'Len = (\d+)\s+Msg = ([0-9a-f]+)\s+MD = ([0-9a-f]+)',text)):
  b=bytes.fromhex(msg)[:int(bits)//8];assert h(b).hex()==md;rows.append((f'NIST_{name}_{i}','sha','raw',b.hex(),md))
(out/'vectors.tsv').write_text(''.join('\t'.join(r)+'\n' for r in rows));print('vectors',len(rows),'binding',20,'boundary',120,'nist',129)

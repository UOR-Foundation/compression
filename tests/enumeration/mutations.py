"""Planted exact-source-IR diagnostic defects; not SDK/kernel qualification."""
import copy
import diagnostic as e
M='Uorc.Search.OwnedEnumeration';H='Uorc.Core.Heap';BB='Uorc.Core.RuntimeByteBuilder';MAX=(1<<64)-1
T=e.tag

def call(n,*a):return e.call(M,n,*a)
def ctype(n):return T(M+'.Phase.'+n)
def getbyte(a,h,index):
 walk=h
 while True:
  r=call('readStep',a,walk,index)
  if r[0].endswith('.byte'):return r[1][0]
  assert r[0].endswith('.descend'),r
  walk,index=r[1]
def value(a,t):return bytes(getbyte(a,t['sequence'],i) for i in range(t['length']))
def offer(a,c):
 for _ in range(1000):
  r=call('perform',a,c);a,c=r['arena'],r['cursor']
  if r['outcome'][0].endswith('.produced'):return a,c,r['outcome'][1][0]
  assert not r['outcome'][0].endswith('.unresolved'),r['outcome']
 raise AssertionError('offer did not finish')
def ack(c,t):
 r=call('acknowledge',c,t);assert r[0]=='Result.ok';return r[1][0]
def fixture(data):
 a=e.call(H,'emptyArena',1000000);c=call('start');r=call('perform',a,c);a,c=r['arena'],r['cursor']
 b=e.call(BB,'begin',a)[1][0];a=b['arena'];builder=b['outcome'][1][0]
 for byte in data:
  b=e.call(BB,'beginAppendOwned',a,builder,byte)[1][0]
  while b['outcome'][0].endswith('.pending'):b=e.call(BB,'step',b['arena'],b['cursor'])[1][0]
  a,builder=b['arena'],b['outcome'][1][0]
 b=e.call(BB,'beginFinishOwned',a,builder)[1][0]
 while b['outcome'][0].endswith('.pending'):b=e.call(BB,'step',b['arena'],b['cursor'])[1][0]
 a=b['arena'];c=dict(c,current=b['outcome'][1][0],length=len(data))
 return a,c

def probe_omission():
 a=e.call(H,'emptyArena',1000000);c=call('start')
 for want in [b'',b'\0',b'\1']:
  a,c,t=offer(a,c);assert value(a,t)==want,'byte omitted by successor';c=ack(c,t)
def probe_pivot():
 a,c=fixture(b'\0\0');a,c,t=offer(a,c);c=ack(c,t);a,c,t=offer(a,c)
 assert value(a,t)==b'\0\1','latest pivot skipped'
def probe_refusal():
 a,c=fixture(b'\0');a,c,t=offer(a,c);c=ack(c,t)
 for _ in range(1000):
  bad=dict(a,exhausted=True);r=call('perform',bad,c)
  if r['outcome'][0].endswith('.unresolved'):
   assert r['cursor']==c and r['arena']==bad,'refusal advanced/excluded predecessor';return
  r=call('perform',a,c);a,c=r['arena'],r['cursor']
 raise AssertionError('no actual allocation refusal reached')
def probe_null_ack():
 a=e.call(H,'emptyArena',1000000);a,c,t=offer(a,call('start'));c=ack(c,t)
 assert call('acknowledge',c,t)[0]=='Result.error','stale empty ticket was reaccepted'
def walk(x):
 if isinstance(x,dict):
  yield x
  for y in x.values():yield from walk(y)
 elif isinstance(x,list):
  for y in x:yield from walk(y)
def omission(d):
 atoms=[x for x in walk(d['body']) if x.get('kind')=='primitive' and x.get('operation')=='checked_add' and x['result']=={'kind':'option','value':{'kind':'uint8'}}]
 assert len(atoms)==1;atoms[0]['arguments'][1]['value']='2'
def pivot(d):
 updates=[x for x in walk(d['body']) if x.get('kind')=='record' and x.get('type',{}).get('name')=='Cursor']
 matched=0
 for r in updates:
  for f in r['fields']:
   if f['field'] in ['pivot','pivotValue'] and f['value']['kind']=='if':
    f['value']['condition']={'kind':'project','value':{'kind':'var','name':'cursor'},'field':'pivotFound'};matched+=1
 assert matched==2

def refusal(d):
 branch=d['body']['branches'][0];assert branch['constructor']['name']=='Result.error'
 cursorField=next(f for f in branch['body']['fields'] if f['field']=='cursor')
 fields=e.mods[M];shape=next(x for x in fields if x['name']=='Cursor')['fields']
 cursorField['value']={'kind':'record','type':{'name':'Cursor'},'fields':[{'field':f['name'],'value':{'kind':'constructor','constructor':{'name':'Phase.offer'},'arguments':[]} if f['name']=='phase' else {'kind':'project','value':{'kind':'var','name':'cursor'},'field':f['name']}} for f in shape]}
def null_ack(d):
 body=d['body'];assert body['kind']=='and';d['body']=body['right']

for probe in [probe_omission,probe_pivot,probe_refusal,probe_null_ack]:probe()
print('restored exact-source diagnostic probes passed')
for name,definition,mutate,probe in [('omission','buildNextStep',omission,probe_omission),('skip-latest-pivot','scanStep',pivot,probe_pivot),('refusal-as-exclusion','builderResult',refusal,probe_refusal),('stale-empty-ack','ticketMatches',null_ack,probe_null_ack)]:
 key=M+'.'+definition;original=e.funcs[key];mutant=copy.deepcopy(original);mutate(mutant);e.funcs[key]=mutant
 try:probe()
 except AssertionError as ex:print('REJECTED',name,str(ex))
 else:raise AssertionError('planted defect escaped: '+name)
 finally:e.funcs[key]=original
 probe()
print('all four planted defects rejected and restored probes passed')

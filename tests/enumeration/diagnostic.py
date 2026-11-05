# Diagnostic evaluator only; never a production implementation or SDK/Lean acceptance.
import pathlib,json,sys,hashlib,re
REPO=pathlib.Path(__file__).resolve().parents[2]
mods={}
for p in (REPO/'src/Uorc').rglob('*.lex.tex'):
 text=p.read_text()
 if '\\semanticdata{' in text:
  m='Uorc.'+str(p.relative_to(REPO/'src/Uorc')).replace('/','.').removesuffix('.lex.tex')
  mods[m]=json.JSONDecoder().raw_decode(text.split('\\semanticdata{')[1])[0]['declarations']
artifact=REPO/'tests/heap/semantic-cache/runtime-owners.json'
assert hashlib.sha256(artifact.read_bytes()).hexdigest()=='19388f9d77aae3d04486816e25eb48e968a0dbc9f7b7c45b92666a3df79afb8e'
for cached in json.loads(artifact.read_text())['modules']:
 assert hashlib.sha256((REPO/cached['source_path']).read_bytes()).hexdigest()==cached['source_sha256']
 mods[cached['module']]=cached['parsed_module']['declarations']
def key(r,m):return r.get('module',m)+'.'+r['name']
funcs={m+'.'+d['name']:d for m,ds in mods.items() for d in ds if d['kind']=='definition'}
def tag(n,*a):return (n,tuple(a))
none=tag('Option.none')
def some(x):return tag('Option.some',x)
def typwidth(t):
 while t['kind']=='option':t=t['value']
 return {'uint8':8,'uint32':32,'uint64':64}.get(t['kind'])
steps=0
def ev(x,m,env):
 global steps;steps+=1
 if steps>100000000:raise RuntimeError('reference evaluation 10M term limit')
 k=x['kind']
 if k=='var':return env[x['name']]
 if k in ['nat','integer']:return int(x['value'])
 if k in ['bool','string']:return x['value']
 if k=='bytes':return bytes.fromhex(x['hex'])
 if k=='unit':return None
 if k=='nil':return []
 if k=='cons':return [ev(x['head'],m,env)]+ev(x['tail'],m,env)
 if k=='record':return {'type':key(x['type'],m),**{f['field']:ev(f['value'],m,env) for f in x['fields']}}
 if k=='project':return ev(x['value'],m,env)[x['field']]
 if k=='constructor':
  n=x['constructor']['name'];a=[ev(y,m,env) for y in x['arguments']]
  if n=='Nat.zero':return 0
  if n=='Nat.succ':return a[0]+1
  if n=='List.nil':return []
  if n=='List.cons':return [a[0]]+a[1]
  return tag(n if n.startswith(('Option.','Result.')) else key(x['constructor'],m),*a)
 if k=='let':
  ee=env.copy();ee[x['binder']['name']]=ev(x['value'],m,env);return ev(x['body'],m,ee)
 if k=='call':
  q=key(x['function'],m);d=funcs[q];a=[ev(y,m,env) for y in x['arguments']];return ev(d['body'],q.rsplit('.',1)[0],dict(zip([p['name'] for p in d['parameters']],a)))
 if k=='lambda':return ('lambda',x,m,env.copy())
 if k=='apply':
  _,xx,mm,ee=ev(x['function'],m,env);ee=ee.copy();ee.update(zip([p['name'] for p in xx['parameters']],[ev(y,m,env) for y in x['arguments']]));return ev(xx['body'],mm,ee)
 if k=='if':return ev(x['then_value'] if ev(x['condition'],m,env) else x['else_value'],m,env)
 if k=='match':
  s=ev(x['scrutinee'],m,env)
  for b in x['branches']:
   n=b['constructor']['name'];args=None
   if n=='Nat.zero' and s==0:args=[]
   elif n=='Nat.succ' and isinstance(s,int) and s>0:args=[s-1]
   elif n=='List.nil' and s==[]:args=[]
   elif n=='List.cons' and isinstance(s,list) and s:args=[s[0],s[1:]]
   elif isinstance(s,tuple) and s[0]==(n if n.startswith(('Option.','Result.')) else key(b['constructor'],m)):args=list(s[1])
   if args is not None:
    ee=env.copy();ee.update(zip(b['binders'],args));return ev(b['body'],m,ee)
  raise RuntimeError(('unmatched',m,s,x['scrutinee'],[b['constructor'] for b in x['branches']]))
 if k=='implies':return (not ev(x['premise'],m,env)) or ev(x['conclusion'],m,env)
 if k=='not':return not ev(x['value'],m,env)
 if k in ['and','or','prop_and']:
  a=ev(x['left'],m,env)
  return (a and ev(x['right'],m,env)) if k!='or' else (a or ev(x['right'],m,env))
 if k in ['eq','beq','le','ble','lt','blt','add']:
  a=ev(x['left'],m,env);b=ev(x['right'],m,env)
  return a==b if k in ['eq','beq'] else a<=b if k in ['le','ble'] else a<b if k in ['lt','blt'] else a+b
 if k=='primitive':
  a=[ev(y,m,env) for y in x['arguments']];op=x['operation'];w=typwidth(x['result']);mask=(1<<w)-1 if w else None
  if op=='length':return len(a[0])
  if op=='append':return a[0]+a[1]
  if op=='slice':
   if a[1]>len(a[0]) or a[2]>len(a[0])-a[1]:return none
   z=a[0][a[1]:a[1]+a[2]];return some(z) if x['result']['kind']=='option' else z
  if op=='index':return some(a[0][a[1]]) if a[1]<len(a[0]) else none
  if op=='equal':return a[0]==a[1]
  if op=='less_than':return a[0]<a[1]
  if op=='to_nat':return a[0]
  if op=='subtract':return max(0,a[0]-a[1]) if not w else (a[0]-a[1])&mask
  if op=='multiply':return a[0]*a[1] if not w else a[0]*a[1]&mask
  if op=='bit_and':return a[0]&a[1]
  if op=='bit_or':return a[0]|a[1]
  if op=='bit_xor':return a[0]^a[1]
  if op=='bit_not':return (~a[0])&mask
  if op=='shift_left':
   z=(a[0]<<a[1])&mask if a[1]<w else 0;return (some(z) if a[1]<w else none) if x['result']['kind']=='option' else z
  if op=='shift_right':
   z=a[0]>>a[1] if a[1]<w else 0;return (some(z) if a[1]<w else none) if x['result']['kind']=='option' else z
  if op in ['divide','quotient','remainder']:
   if not a[1]:return a[2] if len(a)>2 else 0
   return a[0]%a[1] if op=='remainder' else a[0]//a[1]
  if op.startswith('checked_'):
   if op=='checked_add':v=a[0]+a[1]
   elif op=='checked_subtract':v=a[0]-a[1]
   elif op=='checked_multiply':v=a[0]*a[1]
   elif op=='checked_quotient':
    if not a[1]:return none
    v=a[0]//a[1]
   elif op=='checked_convert':v=a[0]
   else:raise RuntimeError(op)
   return some(v) if v>=0 and (mask is None or v<=mask) else none
  raise RuntimeError(('primitive',op))
 raise RuntimeError(('term',k))
def call(mod,name,*args):
 global steps;steps=0;d=funcs[mod+'.'+name];return ev(d['body'],mod,dict(zip([p['name'] for p in d['parameters']],args)))

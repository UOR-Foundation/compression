import diagnostic as e, time,sys
M='Uorc.Search.OwnedEnumeration'; F='Uorc.Search.EnumerationFunding';H='Uorc.Core.Heap';RR='Uorc.Core.RuntimeResources';BB='Uorc.Core.RuntimeByteBuilder';SQ='Uorc.Core.RuntimeSequence'
T=lambda n,*a:e.tag(n,*a)
MAX=(1<<64)-1
budget={'type':RR+'.Budget','maxWork':MAX,'maxOwnedBytes':1000000,'maxOutputBytes':MAX,'maxFrames':MAX}
a=e.call(H,'emptyArena',budget['maxOwnedBytes']);u=e.call(RR,'zeroUsage');c=e.call(M,'start')
adv=T(F+'.Request.advance')
def fund(request=adv,credit=MAX,bb=None):
 global a,u,c
 t=e.call(F,'fund',a,bb or budget,u,c,request,credit);a,u,c=t['arena'],t['usage'],t['cursor'];return t

def bytesof(root):
 z=[]
 def go(h):
  if h['id']==0:return
  r=e.call(H,'lookup',a,h);assert r[0]=='Result.ok';n=r[1][0]['node'];typ,args=n
  if typ.endswith('sequenceLeaf'):
   image,start,length=args;im=e.call(H,'lookup',a,image)[1][0]['node'][1][1];z.extend(im[start:start+length])
  elif typ.endswith('sequenceBranch'):go(args[2]);go(args[3])
  else:raise AssertionError(typ)
 go(root);return bytes(z)

def nextoffer():
 for _ in range(10000):
  t=fund();out=t['outcome']
  if out[0].endswith('.produced'):return out[1][0]
  assert not out[0].endswith('unresolved'),out
 raise AssertionError('did not produce')
start=time.time();seen=[]
for n in range(260):
 ticket=nextoffer();b=bytesof(ticket['sequence']);seen.append(b)
 expected=b'' if n==0 else bytes([n-1]) if n<=256 else bytes([0,n-257])
 assert b==expected,(n,b,expected,c)
 # exact waiting replay, wrong owner/stale phase leave all state intact
 saved=(a,u,c);bad=dict(ticket,owner=dict(ticket['owner'],id=ticket['owner']['id']+1))
 t=fund(T(F+'.Request.acknowledge',bad));assert (a,u,c)==saved;assert t['outcome'][0].endswith('unresolved')
 t=fund(T(F+'.Request.acknowledge',ticket));assert not t['outcome'][0].endswith('unresolved'),t['outcome']
 if n in [0,1,255,256,259]:print('word',n,b.hex(),'events terms',e.steps,'time',round(time.time()-start,2),flush=True)
assert len(set(seen))==260
print('complete empty+all256 one-byte universe, carry to 0000/0001/0002 passed',flush=True)
# Cancel at every saved operation prefix for a multibyte transition: no hidden roots.
for n in range(100):
 sa,su,sc=a,u,c
 t=e.call(F,'cancel',a,budget,u,c,MAX)
 ca,cu,cc=t['arena'],t['usage'],t['cursor']
 for j in range(10000):
  if t['outcome'][0].endswith('.cancelled'):break
  assert not t['outcome'][0].endswith('.unresolved'),(sc['phase'],t['outcome'])
  t=e.call(F,'fund',ca,budget,cu,cc,adv,MAX);ca,cu,cc=t['arena'],t['usage'],t['cursor']
 else:raise AssertionError('cancel did not finish')
 assert ca['liveEntries']==0 and ca['ownedBytes']==0,(sc['phase'],ca)
 assert e.call(M,'rootInventory',cc)==[]
 t=fund()
 if t['outcome'][0].endswith('.produced'):break
print('all successor ownership prefixes cancel to empty arena',n+1,flush=True)
# Re-enter every active event with zero credit, a refused budget, and a split
# prepayment; compare the achieved exact source prefix, roots, Usage and Arena.
events=0
for n in range(200):
 savedA,savedU,savedC=a,u,c
 quote=e.call(M,'requiredWork',c)
 assert quote[0]=='Option.some'
 fee=quote[1][0]
 if fee:
  z=e.call(F,'fund',a,budget,u,c,adv,0)
  assert (z['arena'],z['usage'],z['cursor'])==(a,u,c)
  limited=dict(budget,maxWork=u['work']+fee-1)
  refused=e.call(F,'fund',a,limited,u,c,adv,fee)
  assert (refused['arena'],refused['usage'],refused['cursor'])==(a,u,c)
  assert refused['remainingCredits']==fee and refused['chargedWork']==0
  assert refused['outcome'][0].endswith('.unresolved')
  full=e.call(F,'fund',a,budget,u,c,adv,fee)
  part=e.call(F,'fund',a,budget,u,c,adv,fee-1)
  assert part['arena']==a and part['cursor']['phase']==c['phase']
  split=e.call(F,'fund',part['arena'],budget,part['usage'],part['cursor'],adv,1)
  assert (split['arena'],split['usage'],split['cursor'],split['outcome'])==(full['arena'],full['usage'],full['cursor'],full['outcome'])
  assert part['chargedWork']+split['chargedWork']==fee
  # Repeated cancellation must not reset an in-flight cleanup prepayment.
  cancelled=e.call(M,'cancel',part['cursor'])
  assert e.call(M,'cancel',cancelled)==cancelled
  events+=1
 t=fund()
 if t['outcome'][0].endswith('.produced'):
  ticket=t['outcome'][1][0];fund(T(F+'.Request.acknowledge',ticket))
  if events>=100:break
print('zero/refusal/split-prepayment/repeated-cancel prefixes passed',events,flush=True)
# Addressability refusal leaves empty predecessor unresolved, never complete.
ca=e.call(H,'emptyArena',0);cc=e.call(M,'start');uu=e.call(RR,'zeroUsage');zeroBudget=dict(budget,maxOwnedBytes=0)
r=e.call(F,'fund',ca,zeroBudget,uu,cc,adv,MAX)
assert r['arena']==ca and r['cursor']==cc
assert r['outcome'][0].endswith('.unresolved')
restored=dict(r['arena'],maxOwnedBytes=budget['maxOwnedBytes'])
r=e.call(F,'fund',restored,budget,r['usage'],r['cursor'],adv,MAX)
assert r['cursor']['phase'][0].endswith('.offer')
print('failed control allocation resumes exact empty predecessor',flush=True)
# A real failing allocation in each allocating successor event must preserve its
# entire unexecuted owner/source prefix. A retry with the unchanged live arena
# must agree with the non-refused path.
failures=0
for n in range(150):
 quote=e.call(M,'requiredWork',c)[1][0]
 if quote:
  badArena=dict(a,exhausted=True)
  refused=e.call(F,'fund',badArena,budget,u,c,adv,MAX)
  if refused['outcome'][0].endswith('.unresolved'):
   assert refused['cursor']==c,(c['phase'],refused['cursor']['phase'],refused['outcome'])
   assert refused['arena']==badArena
   retry=e.call(F,'fund',a,budget,refused['usage'],refused['cursor'],adv,MAX)
   expected=e.call(F,'fund',a,budget,refused['usage'],c,adv,MAX)
   assert retry==expected
   failures+=1
 t=fund()
 if t['outcome'][0].endswith('.produced'):
  fund(T(F+'.Request.acknowledge',t['outcome'][1][0]))
  if failures>=3:break
assert failures>=3
print('real allocation refusal and exact retry prefixes passed',failures,flush=True)
# checked length overflow is not a completed universe or a semantic exclusion.
cc=e.call(M,'start');cc.update(phase=T(M+'.Phase.scanRead'),length=MAX,position=MAX,pivotFound=False)
ov=e.call(M,'perform',e.call(H,'emptyArena',1000000),cc)
assert ov['cursor']==cc and ov['outcome'][0].endswith('.unresolved')
assert ov['outcome'][1][0][0].endswith('.addressSpaceExhausted')
print('address overflow retains unresolved predecessor',flush=True)

#!/usr/bin/env python3
"""Hash-guarded read-only comparison of two official inspect-b output directories.
No SDK call, regeneration, verification, source mutation, or compiler build.
Only stdlib. Numeric function IDs are normalized by official origin names.
Core bytes are partitioned by earliest declaration reaching each printed block root;
this allocation is global/order-dependent and is not isolated causal row cost.
"""
from pathlib import Path
import argparse,collections,hashlib,json,re,sys
H=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def J(p):return json.loads(Path(p).read_text())
def origins(p):
 out={}
 for r in J(p/'function-origins.json'):
  m=re.search(r'instance: "([^"]+)"',r['origin']);assert m,r
  out[r['index']]=m.group(1)
 return out

def normalized(x,ids):
 if isinstance(x,list):return[normalized(v,ids)for v in x]
 if not isinstance(x,dict):return x
 y={k:normalized(v,ids)for k,v in x.items()}
 if y.get('kind')in ['call','closure']:y['function']=ids[y['function']]
 return y

def kids(n):
 if n['k']=='a':return[n['f'],n['x']]
 if n['k']in ['l','p']:return[n['t'],n['v']]
 if n['k']=='e':return[n['t'],n['v'],n['body']]
 if n['k']in ['d','j']:return[n['x']]
 if n['k']in ['b','s','c','n','t']:return[]
 raise ValueError('unknown Core node kind '+repr(n['k']))

def core(p,ids):
 data=J(p/'B.core.json');ns=data['nodes'];ds=data['declarations'];text=(p/'B.lex.tex').read_bytes();n=len(ns)
 # Exact partition by node allocation interval. This is NOT exclusive semantic dependency cost.
 matches=list(re.finditer(rb'(?m)^@(\d+) :=',text));endnodes=text.index(b'\\end{coremodule}')
 blocks={int(m.group(1)):(matches[i+1].start()if i+1<len(matches)else endnodes)-m.start()for i,m in enumerate(matches)}
 heads=[];used_as_f=[False]*n
 for i,t in enumerate(ns):
  if t['k']=='a':heads.append(heads[t['f']]);used_as_f[t['f']]=True
  else:heads.append(t.get('n')if t['k']=='c'else None)
 # Declaration inclusive reach and exclusive ownership via one reverse DAG propagation.
 masks=[0]*n
 for j,d in enumerate(ds):masks[d['type']]|=1<<j;masks[d['value']]|=1<<j
 for i in range(n-1,-1,-1):
  for c in kids(ns[i]):assert c<i;masks[c]|=masks[i]
 inclusive=[0]*len(ds);exclusive=[0]*len(ds);owned_counts=[collections.Counter()for _ in ds];owned_bytes=[0]*len(ds);owned_blocks=[0]*len(ds);unattributed_bytes=0
 for idx,m in enumerate(masks):
  if m:
   owner=(m&-m).bit_length()-1;owned_counts[owner][ns[idx]['k']]+=1;owned_bytes[owner]+=blocks.get(idx,0);owned_blocks[owner]+=int(idx in blocks)
  else:unattributed_bytes+=blocks.get(idx,0)
  if m and not(m&(m-1)):exclusive[m.bit_length()-1]+=1
  while m:
   bit=m&-m;inclusive[bit.bit_length()-1]+=1;m-=bit
 declstarts=list(re.finditer(rb'(?m)^(?:definition|theorem) ([^\s:]+)',text));assert len(declstarts)==len(ds)
 firstnode=matches[0].start();rows=[];prev=-1
 for j,d in enumerate(ds):
  name=d['name'].split('.')[-1];fmatch=re.fullmatch(r'fun(\d+)',name);origin=ids[int(fmatch.group(1))]if fmatch else name
  end=max(d['type'],d['value']);assert end>=prev
  kinds=collections.Counter(t['k']for t in ns[prev+1:end+1]);lam=collections.Counter(t['n']for t in ns[prev+1:end+1]if t['k']=='l')
  hb=(declstarts[j+1].start()if j+1<len(ds)else firstnode)-declstarts[j].start()
  nodebytes=sum(blocks.get(i,0)for i in range(prev+1,end+1))
  rows.append({'row':name,'origin':origin,'type_root':d['type'],'value_root':d['value'],'allocated_start':prev+1,'allocated_end':end,'allocated_node_count':end-prev,'allocated_kind_counts':dict(kinds),'allocated_lambda_binders':dict(lam),'notation_header_bytes':hb,'notation_node_block_bytes_by_allocation_interval':nodebytes,'notation_partition_bytes':hb+nodebytes,'notation_earliest_reach_owned_bytes':hb+owned_bytes[j],'earliest_reach_owned_node_kind_counts':dict(owned_counts[j]),'earliest_reach_owned_printed_blocks':owned_blocks[j],'inclusive_DAG_nodes':inclusive[j],'exclusive_DAG_nodes':exclusive[j]});prev=end
 assert prev==n-1
 pre=declstarts[0].start();tail=len(text)-endnodes
 assert pre+tail+sum(r['notation_partition_bytes']for r in rows)==len(text)
 assert pre+tail+unattributed_bytes+sum(r['notation_earliest_reach_owned_bytes']for r in rows)==len(text)
 appheads=collections.Counter(heads[i]or'<nonconstant>'for i,t in enumerate(ns)if t['k']=='a');maxheads=collections.Counter(heads[i]or'<nonconstant>'for i,t in enumerate(ns)if t['k']=='a'and not used_as_f[i]);consts=collections.Counter(t['n']for t in ns if t['k']=='c')
 return {'file_sha256':H(p/'B.core.json'),'notation_sha256':H(p/'B.lex.tex'),'notation_bytes':len(text),'node_count':n,'declaration_count':len(ds),'node_kind_counts':dict(collections.Counter(t['k']for t in ns)),'printed_node_definitions':len(matches),'notation_prelude_bytes':pre,'notation_trailer_bytes':tail,'unattributed_printed_block_bytes':unattributed_bytes,'rows':rows,'application_DAG_nodes_by_head_constant':dict(appheads),'maximal_application_DAG_spines_by_head_constant':dict(maxheads),'constant_names':dict(consts),'measurement_caveat':'Earliest-declaration reachability ownership of full printed blocks gives the primary exact byte partition. Allocation intervals are supplemental numeric-order diagnostics, not a writer-causal attribution. Inclusive/exclusive reach follows explicit term children; named constant definitions are not expanded.'}
def validate(directory,expected):
    required=['identity.json','program.json','function-origins.json','program.rs','B.core.json','B.lex.tex']
    verified={}
    for name in required:
        wanted=expected.get('artifacts',expected.get('artifact_sha256',{}))[name];wanted={'sha256':wanted}if isinstance(wanted,str)else wanted;p=directory/name
        if not p.is_file()or H(p)!=wanted['sha256']or ('bytes'in wanted and p.stat().st_size!=wanted['bytes']):
            raise ValueError('artifact hash mismatch: '+name)
        verified[name]={'sha256':wanted['sha256'],'bytes':p.stat().st_size}
    return verified

def compare(baseline,candidate,findings,case_key,baseline_case,candidate_case):
    expected=J(findings)[case_key];dirs=[baseline,candidate];labels=[baseline_case,candidate_case];cases={};programs=[];maps=[]
    for label,p in zip(labels,dirs):
        verified=validate(p,expected[label]);identity=J(p/'identity.json');ids=origins(p);program=J(p/'program.json')
        if len(ids)!=len(program['functions']):raise ValueError('origin/function count mismatch')
        context=expected[label].get('result',{}).get('identity',expected[label])
        if identity['checked_modules']!=context['checked_modules']:raise ValueError('checked module list mismatch')
        if identity['root']!=context['root']:raise ValueError('root mismatch')
        cases[label]={'verified_artifacts':verified,'identity':identity,'target_function_count':len(ids),'ADT_sha256':hashlib.sha256(json.dumps(program['adts'],sort_keys=True,separators=(',',':')).encode()).hexdigest(),'official_measures':{k:v for k,v in expected[label]['official_measures'].items()if k!='program_path'},'official_measures_source':'Recorded SDK metrics copied from supplied findings; not recomputed by this analyzer','B':core(p,ids)}
        # Numeric interval counts are private diagnostics, not primary attribution.
        for row in cases[label]['B']['rows']:
            for field in list(row):
                if field.startswith('allocated_')or field in ['notation_partition_bytes','notation_node_block_bytes_by_allocation_interval']:
                    del row[field]
        cases[label]['B']['measurement_caveat']='Each complete printed node block belongs to the earliest declaration whose explicit type/value DAG reaches that block root. Includes inlined children in that block. Named constants are not definition-expanded. Shared nodes, global printing, and declaration order couple the rows; no isolated causal per-row cost is asserted.'
        programs.append(program);maps.append(ids)
    fa={maps[0][i]:normalized(fn,maps[0])for i,fn in enumerate(programs[0]['functions'])};fb={maps[1][i]:normalized(fn,maps[1])for i,fn in enumerate(programs[1]['functions'])};common=set(fa)&set(fb);ra={r['origin']:r for r in cases[labels[0]]['B']['rows']};rb={r['origin']:r for r in cases[labels[1]]['B']['rows']};delta=[]
    for name in sorted(set(ra)|set(rb)):
        a=ra.get(name,{});b=rb.get(name,{})
        delta.append({'origin':name,'before':a,'after':b,'notation_bytes_delta':b.get('notation_earliest_reach_owned_bytes',0)-a.get('notation_earliest_reach_owned_bytes',0),'owned_node_kind_delta':{k:b.get('earliest_reach_owned_node_kind_counts',{}).get(k,0)-a.get('earliest_reach_owned_node_kind_counts',{}).get(k,0)for k in ['c','a','n','b','l','p','s','t','e','d','j']}})
    a=cases[labels[0]]['B'];b=cases[labels[1]]['B'];byte_delta=b['notation_bytes']-a['notation_bytes'];overhead_delta=sum(b[k]-a[k]for k in ['notation_prelude_bytes','notation_trailer_bytes','unattributed_printed_block_bytes']);assert sum(r['notation_bytes_delta']for r in delta)+overhead_delta==byte_delta
    return {'schema':'uorc/hash-guarded-b-structure/1','findings_sha256':H(findings),'cases':cases,'normalization':{'method':'Replace target Call/Closure function IDs by official origin instance; preserve every other target JSON field','common_count':len(common),'changed_common':[n for n in sorted(common)if fa[n]!=fb[n]],'added':sorted(set(fb)-set(fa)),'removed':sorted(set(fa)-set(fb)),'ADTs_exact':programs[0]['adts']==programs[1]['adts']},'row_byte_delta':delta,'source_byte_reconciliation':{'B_delta':byte_delta,'rows_delta':sum(r['notation_bytes_delta']for r in delta),'prelude_trailer_unattributed_delta':overhead_delta,'exact':True},'limits':['Diagnostic attribution of actual official files, not a new certificate or acceptance result.','No retained Lean LCNF/SSA dump is inferred from target calculus or printed Rust.','Global sharing/formatting/order makes reach ownership a reproducible convention, not an isolated causal price.']}

def selftest():
    assert kids({'k':'e','t':0,'v':1,'body':2})==[0,1,2]
    assert kids({'k':'d','x':7})==[7]and kids({'k':'j','x':8})==[8]
    try:kids({'k':'unknown'})
    except ValueError:pass
    else:raise AssertionError('unknown node was accepted')

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--baseline',required=True,type=Path);p.add_argument('--candidate',required=True,type=Path);p.add_argument('--findings',required=True,type=Path);p.add_argument('--output',required=True,type=Path);p.add_argument('--cases-key',default='full_matched_cases');p.add_argument('--baseline-case',default='baseline-fc419');p.add_argument('--candidate-case',default='candidate-415d8');args=p.parse_args();selftest()
    result=compare(args.baseline,args.candidate,args.findings,args.cases_key,args.baseline_case,args.candidate_case);args.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result['source_byte_reconciliation']))
if __name__=='__main__':
    try:main()
    except (AssertionError,KeyError,ValueError,OSError)as error:
        print('analysis rejected: '+str(error),file=sys.stderr);raise SystemExit(2)

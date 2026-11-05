# In-memory diagnostic mutants only; this is not a proof or compiler gate.
import sys,pathlib,copy
if sys.flags.optimize != 0 or not __debug__:
 raise RuntimeError('optimized Python is not allowed for source mutants')
if sys.argv[1:]:
 raise RuntimeError('filter, selection, and other arguments are not allowed')
import diagnostic as d
M='Uorc.Search.OwnedEnumerationModel'
def theorem(n):return next(x for x in d.mods[M] if x['name']==n)
def check(thm,env):d.steps=0;return d.ev(theorem(thm)['statement'],M,env)
checks=[('modelWordsOfBytes','model_unsigned_byte_word_round_trip',{'bytes':[1]},'constant-zero embedding'),('modelNormalize','model_normalization_preserves_every_list_cell',{'words':[0]},'drop-zero normalization'),('modelReverseSuccessor','model_reverse_successor_increments_rank',{'words':[0]},'successor plus two')]
for func,thm,env,label in checks:
 f=d.funcs[M+'.'+func];saved=copy.deepcopy(f['body']);assert check(thm,env)
 if func=='modelWordsOfBytes':f['body']['branches'][1]['body']['head']={'kind':'nat','value':'0'}
 elif func=='modelNormalize':
  body=f['body']['branches'][1]['body'];f['body']['branches'][1]['body']={'kind':'if','condition':{'kind':'beq','left':body['head'],'right':{'kind':'nat','value':'0'}},'then_value':body['tail'],'else_value':body}
 else:f['body']['branches'][1]['body']['then_value']['head']['right']['value']='2'
 try:assert not check(thm,env),label+' escaped'
 finally:f['body']=saved
 assert check(thm,env)
 print('REJECTED',label,'by',thm,'and restored')

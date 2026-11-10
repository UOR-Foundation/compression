# Design prototype of the uorc LZ kernel (LZMA-class). Scratch only; the product is LexLean.
import sys
TOP=1<<24; BITS=11; MOVE=5; HALF=1<<10
class Enc:
    def __init__(s): s.low=0; s.rng=0xFFFFFFFF; s.cache=0; s.csize=1; s.out=bytearray()
    def shift(s):
        if s.low < 0xFF000000 or s.low >= (1<<32):
            carry = s.low>>32
            t=s.cache
            while True:
                s.out.append((t+carry)&0xFF); t=0xFF; s.csize-=1
                if s.csize==0: break
            s.cache=(s.low>>24)&0xFF
        s.csize+=1; s.low=(s.low&0x00FFFFFF)<<8
    def bit(s,p,i,b):
        v=p[i]; bound=(s.rng>>BITS)*v
        if b==0: s.rng=bound; p[i]=v+(((1<<BITS)-v)>>MOVE)
        else: s.low+=bound; s.rng-=bound; p[i]=v-(v>>MOVE)
        while s.rng<TOP: s.rng=(s.rng<<8)&0xFFFFFFFF; s.shift()
    def direct(s,v,n):
        for k in range(n-1,-1,-1):
            s.rng>>=1
            if (v>>k)&1: s.low+=s.rng
            while s.rng<TOP: s.rng=(s.rng<<8)&0xFFFFFFFF; s.shift()
    def finish(s):
        for _ in range(5): s.shift()
        return bytes(s.out)
class Dec:
    def __init__(s,data): s.d=data; s.pos=5; s.rng=0xFFFFFFFF; s.code=int.from_bytes(data[1:5],'big')
    def nb(s):
        b=s.d[s.pos] if s.pos<len(s.d) else 0; s.pos+=1; return b
    def bit(s,p,i):
        v=p[i]; bound=(s.rng>>BITS)*v
        if s.code<bound: s.rng=bound; p[i]=v+(((1<<BITS)-v)>>MOVE); b=0
        else: s.code-=bound; s.rng-=bound; p[i]=v-(v>>MOVE); b=1
        while s.rng<TOP: s.rng=(s.rng<<8)&0xFFFFFFFF; s.code=((s.code<<8)|s.nb())&0xFFFFFFFF
        return b
    def direct(s,n):
        r=0
        for _ in range(n):
            s.rng>>=1
            b = 1 if s.code>=s.rng else 0
            if b: s.code-=s.rng
            r=(r<<1)|b
            while s.rng<TOP: s.rng=(s.rng<<8)&0xFFFFFFFF; s.code=((s.code<<8)|s.nb())&0xFFFFFFFF
        return r
# model layout
LC=3; NST=12; POSB=0
def model():
    m={}
    for k,n in [('isMatch',NST),('isRep',NST),('isRepG0',NST),('isRepG1',NST),('isRepG2',NST),('isRep0Long',NST),
                ('lit',0x300<<LC),('slot',4*64),('spec',115),('align',16)]: m[k]=[HALF]*n
    for L in ('len','rlen'): m[L]={'c':[HALF,HALF],'low':[HALF]*(8*1),'mid':[HALF]*(8*1),'high':[HALF]*256}
    return m
# low/mid trees sized 8 (3 bits) ; we use pos-state count 1 so index base 0
def bt_enc(e,p,base,nbits,v):
    m=1
    for k in range(nbits-1,-1,-1):
        b=(v>>k)&1; e.bit(p,base+m,b); m=(m<<1)|b
def bt_dec(d,p,base,nbits):
    m=1
    for _ in range(nbits): m=(m<<1)|d.bit(p,base+m)
    return m-(1<<nbits)
def rbt_enc(e,p,base,nbits,v):
    m=1
    for k in range(nbits):
        b=(v>>k)&1; e.bit(p,base+m,b); m=(m<<1)|b
def rbt_dec(d,p,base,nbits):
    m=1; r=0
    for k in range(nbits):
        b=d.bit(p,base+m); m=(m<<1)|b; r|=b<<k
    return r
# 8-entry trees need index up to 15 -> size 16
def lenmodel(): return {'c':[HALF,HALF],'low':[HALF]*16,'mid':[HALF]*16,'high':[HALF]*512}
def len_enc(e,L,v):  # v = len-2
    if v<8: e.bit(L['c'],0,0); bt_enc(e,L['low'],0,3,v)
    elif v<16: e.bit(L['c'],0,1); e.bit(L['c'],1,0); bt_enc(e,L['mid'],0,3,v-8)
    else: e.bit(L['c'],0,1); e.bit(L['c'],1,1); bt_enc(e,L['high'],0,8,v-16)
def len_dec(d,L):
    if d.bit(L['c'],0)==0: return bt_dec(d,L['low'],0,3)
    if d.bit(L['c'],1)==0: return 8+bt_dec(d,L['mid'],0,3)
    return 16+bt_dec(d,L['high'],0,8)
MINM=2; MAXM=273
def slot_of(dist):
    if dist<4: return dist
    n=dist.bit_length()-1
    return (n<<1)|((dist>>(n-1))&1)
def dist_enc(e,m,dist,ln):
    ls=min(ln-2,3); s=slot_of(dist); bt_enc(e,m['slot'],ls*64,6,s)
    if s>=4:
        fb=(s>>1)-1; base=(2|(s&1))<<fb; red=dist-base
        if s<14: rbt_enc(e,m['spec'],base-s,fb,red)   # LZMA SpecPos base: base - slot - 1 + 1(m starts at1)
        else: e.direct(red>>4,fb-4); rbt_enc(e,m['align'],0,4,red&15)
def dist_dec(d,m,ln):
    ls=min(ln-2,3); s=bt_dec(d,m['slot'],ls*64,6)
    if s<4: return s
    fb=(s>>1)-1; base=(2|(s&1))<<fb
    if s<14: return base+rbt_dec(d,m['spec'],base-s,fb)
    return base+(d.direct(fb-4)<<4)+rbt_dec(d,m['align'],0,4)
def st_lit(s): return 0 if s<4 else (s-3 if s<10 else s-6)
def st_match(s): return 7 if s<7 else 10
def st_rep(s): return 8 if s<7 else 11
def st_short(s): return 9 if s<7 else 11
def lit_enc(e,m,prev,s,b,mb):
    base=0x300*(prev>>(8-LC))
    p=m['lit']; sym=1
    if s>=7:
        for k in range(7,-1,-1):
            mbit=(mb>>k)&1; bit=(b>>k)&1
            e.bit(p,base+((1+mbit)<<8)+sym,bit); sym=(sym<<1)|bit
            if mbit!=bit: break
        else:
            return
        k-=1
        while k>=0:
            bit=(b>>k)&1; e.bit(p,base+sym,bit); sym=(sym<<1)|bit; k-=1
        return
    for k in range(7,-1,-1):
        bit=(b>>k)&1; e.bit(p,base+sym,bit); sym=(sym<<1)|bit
def lit_dec(d,m,prev,s,mb):
    base=0x300*(prev>>(8-LC)); p=m['lit']; sym=1
    if s>=7:
        k=7
        while k>=0:
            mbit=(mb>>k)&1; bit=d.bit(p,base+((1+mbit)<<8)+sym); sym=(sym<<1)|bit; k-=1
            if mbit!=bit: break
        while sym<0x100: sym=(sym<<1)|d.bit(p,base+sym)
        return sym-0x100
    while sym<0x100: sym=(sym<<1)|d.bit(p,base+sym)
    return sym-0x100
def compress(data, depth=48, nice=64):
    return compress_with(b'', data, depth, nice)
def compress_with(hist, data, depth=48, nice=64):
    # encode `data` against the window `hist`: matches may reach into it; only `data` is emitted
    hl=len(hist); m_in=len(data); data=bytes(hist)+bytes(data)
    e=Enc(); m=model(); L=lenmodel(); R=lenmodel()
    n=len(data); HB=16; head3=[-1]*(1<<HB); chain=[-1]*n
    cd=0
    while (1<<cd)<n+1: cd+=1
    H4=min(22,max(16,cd)); head=[-1]*(1<<H4)
    def h(i): return ((data[i]<<16|data[i+1]<<8|data[i+2])*2654435761>>(32-HB))&((1<<HB)-1)
    def h4(i): return ((int.from_bytes(data[i:i+4],'little')*2654435761)>>(32-H4))&((1<<H4)-1)
    def ins(i):
        if i+3<n: k=h4(i); chain[i]=head[k]; head[k]=i
        if i+2<n: head3[h(i)]=i
    def mlen(a,b,lim):
        l=0
        while l<lim and data[a+l]==data[b+l]: l+=1
        return l
    def find(i):
        # the latest position with the same 3-byte hash gives the short matches; the 4-byte chain, the long ones
        best=0; bd=0
        if i+2>=n: return 0,0
        lim=min(MAXM,n-i)
        j=head3[h(i)]
        if j>=0:
            l=mlen(j,i,lim)
            if l>=2: best=l; bd=i-j-1
        if i+3>=n or best>=nice: return best,bd
        j=head[h4(i)]; c=depth
        while j>=0 and c>0:
            if (data[j+best] if j+best<n else 0)==(data[i+best] if i+best<n else 0):  # an out-of-range read is 0, as in LexLean
                l=mlen(j,i,lim)
                if l>best: best=l; bd=i-j-1
                if l>=nice: break
            j=chain[j]; c-=1
        return best,bd
    for j in range(hl): ins(j)
    s=0; reps=[0,0,0,0]; i=hl
    while i<n:
        lim=min(MAXM,n-i)
        # reps
        rl=0; ri=0
        for k in range(4):
            src=i-reps[k]-1
            if src>=0:
                l=mlen(src,i,lim)
                if l>rl: rl=l; ri=k
        ml,md=find(i)
        if ml==2 and md>=128: ml=0
        ins(i)
        if ml>=2 and ml>rl+1 and i+1<n:
            ml2,md2=find(i+1)
            if ml2>ml+1 or (ml2==ml+1 and md2<md*4): ml=0
        prev=data[i-1] if i>0 else 0
        if rl>=2 and rl+1>=ml:
            e.bit(m['isMatch'],s,1); e.bit(m['isRep'],s,1)
            if ri==0:
                e.bit(m['isRepG0'],s,0); e.bit(m['isRep0Long'],s,1)
            else:
                e.bit(m['isRepG0'],s,1)
                if ri==1: e.bit(m['isRepG1'],s,0)
                else:
                    e.bit(m['isRepG1'],s,1); e.bit(m['isRepG2'],s,ri-2)
                d=reps[ri]; del reps[ri]; reps.insert(0,d)
            len_enc(e,R,rl-2); s=st_rep(s)
            for k in range(1,rl): ins(i+k)
            i+=rl; continue
        if ml>=2:
            e.bit(m['isMatch'],s,1); e.bit(m['isRep'],s,0)
            len_enc(e,L,ml-2); dist_enc(e,m,md,ml)
            reps=[md]+reps[:3]; s=st_match(s)
            for k in range(1,ml): ins(i+k)
            i+=ml; continue
        e.bit(m['isMatch'],s,0)
        mb=data[i-reps[0]-1] if i-reps[0]-1>=0 else 0
        lit_enc(e,m,prev,s,data[i],mb); s=st_lit(s); i+=1
    return m_in.to_bytes(8,'little')+e.finish()
def decompress(blob):
    n=int.from_bytes(blob[:8],'little'); d=Dec(blob[8:]); m=model(); L=lenmodel(); R=lenmodel()
    out=bytearray(); s=0; reps=[0,0,0,0]
    while len(out)<n:
        prev=out[-1] if out else 0
        if d.bit(m['isMatch'],s)==0:
            mb=out[-reps[0]-1] if len(out)-reps[0]-1>=0 else 0
            out.append(lit_dec(d,m,prev,s,mb)); s=st_lit(s); continue
        if d.bit(m['isRep'],s):
            if d.bit(m['isRepG0'],s)==0:
                if d.bit(m['isRep0Long'],s)==0:
                    out.append(out[-reps[0]-1]); s=st_short(s); continue
            else:
                if d.bit(m['isRepG1'],s)==0: ri=1
                else: ri=2+d.bit(m['isRepG2'],s)
                dd=reps[ri]; del reps[ri]; reps.insert(0,dd)
            ln=len_dec(d,R)+2; s=st_rep(s)
        else:
            ln=len_dec(d,L)+2; dd=dist_dec(d,m,ln); reps=[dd]+reps[:3]; s=st_match(s)
        src=len(out)-reps[0]-1
        for k in range(ln): out.append(out[src+k])
    return bytes(out)
if __name__=='__main__' and sys.argv[1] not in ('c','d'):
    import time,zlib,lzma
    for f in sys.argv[1:]:
        data=open(f,'rb').read(); t=time.time(); c=compress(data); t1=time.time(); dd=decompress(c); t2=time.time()
        assert dd==data, f
        print(f"{f}: {len(data)} -> ulz {len(c)}  gzip9 {len(zlib.compress(data,9))}  xz9e {len(lzma.compress(data,preset=9|lzma.PRESET_EXTREME))}  enc {t1-t:.1f}s dec {t2-t1:.1f}s")
def _cli():
    import sys
    if sys.argv[1]=='c': open(sys.argv[3],'wb').write(compress(open(sys.argv[2],'rb').read()))
    elif sys.argv[1]=='d': open(sys.argv[3],'wb').write(decompress(open(sys.argv[2],'rb').read()))
if __name__=='__main__' and sys.argv[1] in ('c','d'): _cli()

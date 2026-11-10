import sys,re
def fix(s):
    while True:
        i=s.find(' * ')
        if i<0: return s
        # find enclosing '('
        d=0; j=i
        while j>=0:
            c=s[j]
            if c==')': d+=1
            elif c=='(':
                if d==0: break
                d-=1
            j-=1
        # find matching ')'
        d=0; k=j
        while True:
            c=s[k]
            if c=='(': d+=1
            elif c==')':
                d-=1
                if d==0: break
            k+=1
        inner=s[j+1:k]
        # split top-level ' * '
        d=0; parts=[]; last=0; m=0
        while m<len(inner):
            c=inner[m]
            if c in '([{': d+=1
            elif c in ')]}': d-=1
            elif d==0 and inner.startswith(' * ',m): parts.append(inner[last:m]); last=m+3; m+=3; continue
            m+=1
        parts.append(inner[last:])
        # Nat.mul recurses on its second argument, so a certificate's kernel unfolds `x * 16777216` once per unit.
        # A literal goes first, where the recursion meets the symbolic side and stops.
        lits=[q for q in parts if q.strip().isdigit()]; rest=[q for q in parts if not q.strip().isdigit()]
        parts=lits+rest
        expr=parts[0]
        for p in parts[1:]: expr=f"(prim multiply {expr} {p} : Nat)"
        s=s[:j]+expr+s[k+1:]
p=sys.argv[1]; t=fix(open(p).read().replace('(Nat * Dec)','PAIRTY').replace('(Dec * Bool)','DBTY').replace('(Nat * Nat)','NNTY').replace('(Enc * Bool)','EBTY').replace('(Nat * Bytes)','NBYTY').replace('(Bytes * Bytes)','BBTY')).replace('PAIRTY','(Nat * Dec)').replace('DBTY','(Dec * Bool)').replace('NNTY','(Nat * Nat)').replace('EBTY','(Enc * Bool)').replace('NBYTY','(Nat * Bytes)').replace('BBTY','(Bytes * Bytes)'); open(p,'w').write(t)

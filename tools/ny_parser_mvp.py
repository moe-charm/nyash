#!/usr/bin/env python3
"""
Ny parser MVP (Stage 1): Ny -> JSON v0

Grammar (subset):
  program  := [return] expr EOF
  expr     := term (('+'|'-') term)*
  term     := factor (('*'|'/') factor)*
  factor   := INT | STRING | '(' expr ')'

Outputs JSON v0 compatible with --ny-parser-pipe.
"""
import sys, re, json

class Tok:
    def __init__(self, kind, val, pos):
        self.kind, self.val, self.pos = kind, val, pos

def lex(s: str):
    i=n=0; n=len(s); out=[]
    while i<n:
        c=s[i]
        if c.isspace():
            i+=1; continue
        if c in '+-*/()':
            out.append(Tok(c,c,i)); i+=1; continue
        if c.isdigit():
            j=i
            while j<n and s[j].isdigit():
                j+=1
            out.append(Tok('INT', int(s[i:j]), i)); i=j; continue
        if c=='"':
            j=i+1; buf=[]
            while j<n:
                if s[j]=='\\' and j+1<n:
                    buf.append(s[j+1]); j+=2; continue
                if s[j]=='"':
                    j+=1; break
                buf.append(s[j]); j+=1
            out.append(Tok('STR',''.join(buf), i)); i=j; continue
        if s.startswith('return', i):
            out.append(Tok('RETURN','return', i)); i+=6; continue
        raise SyntaxError(f"lex: unexpected '{c}' at {i}")
    out.append(Tok('EOF','',n))
    return out

class P:
    def __init__(self,toks): self.t=toks; self.i=0
    def cur(self): return self.t[self.i]
    def eat(self,k):
        if self.cur().kind==k: self.i+=1; return True
        return False
    def expect(self,k):
        if not self.eat(k): raise SyntaxError(f"expect {k} at {self.cur().pos}")
    def factor(self):
        tok=self.cur()
        if self.eat('INT'): return {"type":"Int","value":tok.val}
        if self.eat('STR'): return {"type":"Str","value":tok.val}
        if self.eat('('):
            e=self.expr(); self.expect(')'); return e
        raise SyntaxError(f"factor at {tok.pos}")
    def term(self):
        lhs=self.factor()
        while self.cur().kind in ('*','/'):
            op=self.cur().kind; self.i+=1
            rhs=self.factor()
            lhs={"type":"Binary","op":op,"lhs":lhs,"rhs":rhs}
        return lhs
    def expr(self):
        lhs=self.term()
        while self.cur().kind in ('+','-'):
            op=self.cur().kind; self.i+=1
            rhs=self.term()
            lhs={"type":"Binary","op":op,"lhs":lhs,"rhs":rhs}
        return lhs
    def program(self):
        if self.eat('RETURN'):
            e=self.expr()
        else:
            e=self.expr()
        self.expect('EOF')
        return {"version":0, "kind":"Program", "body":[{"type":"Return","expr":e}]}

def main():
    if len(sys.argv)<2:
        print("usage: ny_parser_mvp.py <file.nyash>", file=sys.stderr); sys.exit(1)
    with open(sys.argv[1],'r',encoding='utf-8') as f:
        src=f.read()
    toks=lex(src)
    prog=P(toks).program()
    print(json.dumps(prog, ensure_ascii=False))

if __name__=='__main__':
    main()

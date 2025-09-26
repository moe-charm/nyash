#!/usr/bin/env python3
"""
Ny parser MVP (Stage 2): Ny -> JSON v0

Grammar (subset):
  program  := stmt* EOF
  stmt     := 'return' expr
            | 'local' IDENT '=' expr
            | 'if' expr block ('else' block)?
            | 'loop' '(' expr ')' block
            | expr                         # expression statement
  block    := '{' stmt* '}'

  expr     := logic
  logic    := compare (('&&'|'||') compare)*
  compare  := sum (('=='|'!='|'<'|'>'|'<='|'>=') sum)?
  sum      := term (('+'|'-') term)*
  term     := unary (('*'|'/') unary)*
  unary    := '-' unary | factor
  factor   := INT | STRING | IDENT call_tail* | '(' expr ')' | 'new' IDENT '(' args? ')'
            | '{' map_entries? '}'               # map literal (string keys only)
  call_tail:= '.' IDENT '(' args? ')'   # method
            | '(' args? ')'             # function call
  args     := expr (',' expr)*

Outputs JSON v0 compatible with --ny-parser-pipe.
"""
import sys, re, json

class Tok:
    def __init__(self, kind, val, pos):
        self.kind, self.val, self.pos = kind, val, pos

KEYWORDS = {
    'return':'RETURN', 'local':'LOCAL', 'if':'IF', 'else':'ELSE', 'loop':'LOOP', 'new':'NEW'
}

def lex(s: str):
    i=0; n=len(s); out=[]
    def peek():
        return s[i] if i<n else ''
    while i<n:
        c = s[i]
        if c.isspace():
            i+=1; continue
        # two-char ops
        if s.startswith('==', i) or s.startswith('!=', i) or s.startswith('<=', i) or s.startswith('>=', i) or s.startswith('&&', i) or s.startswith('||', i):
            out.append(Tok('OP2', s[i:i+2], i)); i+=2; continue
        if c in '+-*/(){}.,<>=[]:':
            out.append(Tok(c, c, i)); i+=1; continue
        if c=='"':
            j=i+1; buf=[]
            while j<n:
                if s[j]=='\\' and j+1<n:
                    buf.append(s[j+1]); j+=2; continue
                if s[j]=='"': j+=1; break
                buf.append(s[j]); j+=1
            out.append(Tok('STR',''.join(buf), i)); i=j; continue
        if c.isdigit():
            j=i
            while j<n and s[j].isdigit(): j+=1
            out.append(Tok('INT', int(s[i:j]), i)); i=j; continue
        if c.isalpha() or c=='_':
            j=i
            while j<n and (s[j].isalnum() or s[j]=='_'): j+=1
            ident = s[i:j]
            if ident in KEYWORDS:
                out.append(Tok(KEYWORDS[ident], ident, i))
            else:
                out.append(Tok('IDENT', ident, i))
            i=j; continue
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
    def program(self):
        body=[]
        while self.cur().kind!='EOF':
            body.append(self.stmt())
        return {"version":0, "kind":"Program", "body":body}
    def stmt(self):
        if self.eat('RETURN'):
            e=self.expr(); return {"type":"Return","expr":e}
        if self.eat('LOCAL'):
            tok=self.cur(); self.expect('IDENT'); name=tok.val
            self.expect('='); e=self.expr(); return {"type":"Local","name":name,"expr":e}
        if self.eat('IF'):
            cond=self.expr(); then=self.block(); els=None
            if self.eat('ELSE'):
                els=self.block()
            return {"type":"If","cond":cond,"then":then,"else":els}
        if self.eat('LOOP'):
            self.expect('('); cond=self.expr(); self.expect(')'); body=self.block()
            return {"type":"Loop","cond":cond,"body":body}
        # expression statement
        e=self.expr(); return {"type":"Expr","expr":e}
    def block(self):
        self.expect('{'); out=[]
        while self.cur().kind!='}': out.append(self.stmt())
        self.expect('}'); return out
    def expr(self): return self.logic()
    def logic(self):
        lhs=self.compare()
        while (self.cur().kind=='OP2' and self.cur().val in ('&&','||')):
            op=self.cur().val; self.i+=1
            rhs=self.compare(); lhs={"type":"Logical","op":op,"lhs":lhs,"rhs":rhs}
        return lhs
    def compare(self):
        lhs=self.sum()
        k=self.cur().kind; v=getattr(self.cur(),'val',None)
        if (k=='OP2' and v in ('==','!=','<=','>=')) or k in ('<','>'):
            op = v if k=='OP2' else self.cur().kind
            self.i+=1
            rhs=self.sum(); return {"type":"Compare","op":op,"lhs":lhs,"rhs":rhs}
        return lhs
    def sum(self):
        lhs=self.term()
        while self.cur().kind in ('+','-'):
            op=self.cur().kind; self.i+=1
            rhs=self.term(); lhs={"type":"Binary","op":op,"lhs":lhs,"rhs":rhs}
        return lhs
    def term(self):
        lhs=self.unary()
        while self.cur().kind in ('*','/'):
            op=self.cur().kind; self.i+=1
            rhs=self.unary(); lhs={"type":"Binary","op":op,"lhs":lhs,"rhs":rhs}
        return lhs
    def unary(self):
        if self.cur().kind=='-':
            self.i+=1
            rhs=self.unary()
            return {"type":"Binary","op":"-","lhs":{"type":"Int","value":0},"rhs":rhs}
        return self.factor()
    def factor(self):
        tok=self.cur()
        if self.eat('INT'): return {"type":"Int","value":tok.val}
        if self.eat('STR'): return {"type":"Str","value":tok.val}
        if self.eat('('):
            e=self.expr(); self.expect(')'); return e
        # Array literal: [e1, e2, ...] → Call{name:"array.of", args:[...]}
        if self.eat('['):
            args=[]
            if self.cur().kind != ']':
                args.append(self.expr())
                while self.eat(','):
                    args.append(self.expr())
            self.expect(']')
            return {"type":"Call","name":"array.of","args":args}
        # Map literal: {"k": v, ...} (string keys only) → Call{name:"map.of", args:[Str(k1), v1, ...]}
        if self.eat('{'):
            args=[]
            if self.cur().kind != '}':
                # first entry
                k=self.cur(); self.expect('STR');
                self.expect(':'); v=self.expr();
                args.append({"type":"Str","value":k.val}); args.append(v)
                while self.eat(','):
                    if self.cur().kind == '}':
                        break
                    k=self.cur(); self.expect('STR');
                    self.expect(':'); v=self.expr();
                    args.append({"type":"Str","value":k.val}); args.append(v)
            self.expect('}')
            return {"type":"Call","name":"map.of","args":args}
        if self.eat('NEW'):
            t=self.cur(); self.expect('IDENT'); self.expect('(')
            args=self.args_opt(); self.expect(')')
            return {"type":"New","class":t.val,"args":args}
        if self.eat('IDENT'):
            node={"type":"Var","name":tok.val}
            # call/methtail
            while True:
                if self.eat('('):
                    args=self.args_opt(); self.expect(')')
                    node={"type":"Call","name":tok.val,"args":args}
                elif self.eat('.'):
                    m=self.cur(); self.expect('IDENT'); self.expect('(')
                    args=self.args_opt(); self.expect(')')
                    node={"type":"Method","recv":node,"method":m.val,"args":args}
                else:
                    break
            return node
        raise SyntaxError(f"factor at {tok.pos}")
    def args_opt(self):
        args=[]
        if self.cur().kind in (')',):
            return args
        args.append(self.expr())
        while self.eat(','):
            args.append(self.expr())
        return args

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

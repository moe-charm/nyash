use super::ast::{ExprV0, ProgramV0, StmtV0};

#[derive(Clone, Debug)]
enum Tok {
    Int(i64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Return,
    Eof,
}

fn lex(input: &str) -> Result<Vec<Tok>, String> {
    let mut chars = input.chars().peekable();
    let mut toks = Vec::new();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\n' | '\t' | '\r' => {
                chars.next();
            }
            '+' => {
                chars.next();
                toks.push(Tok::Plus);
            }
            '-' => {
                chars.next();
                toks.push(Tok::Minus);
            }
            '*' => {
                chars.next();
                toks.push(Tok::Star);
            }
            '/' => {
                chars.next();
                toks.push(Tok::Slash);
            }
            '(' => {
                chars.next();
                toks.push(Tok::LParen);
            }
            ')' => {
                chars.next();
                toks.push(Tok::RParen);
            }
            '0'..='9' => {
                let mut n = 0i64;
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        n = n * 10 + (d as i64 - '0' as i64);
                        chars.next();
                    } else {
                        break;
                    }
                }
                toks.push(Tok::Int(n));
            }
            'r' => {
                let kw = "return";
                let mut it = kw.chars();
                let mut ok = true;
                for _ in 0..kw.len() {
                    if Some(chars.next().unwrap_or('\0')) != it.next() {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    toks.push(Tok::Return);
                } else {
                    return Err("unexpected 'r'".into());
                }
            }
            _ => return Err(format!("unexpected char '{}'", c)),
        }
    }
    toks.push(Tok::Eof);
    Ok(toks)
}

struct P {
    toks: Vec<Tok>,
    pos: usize,
}
impl P {
    fn new(toks: Vec<Tok>) -> Self {
        Self { toks, pos: 0 }
    }
    fn peek(&self) -> &Tok {
        self.toks.get(self.pos).unwrap()
    }
    fn next(&mut self) -> Tok {
        let t = self.toks.get(self.pos).unwrap().clone();
        self.pos += 1;
        t
    }
    fn expect_return(&mut self) -> Result<(), String> {
        match self.next() {
            Tok::Return => Ok(()),
            _ => Err("expected 'return'".into()),
        }
    }
    fn parse_program(&mut self) -> Result<ExprV0, String> {
        self.expect_return()?;
        self.parse_expr()
    }
    fn parse_expr(&mut self) -> Result<ExprV0, String> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Tok::Plus => {
                    self.next();
                    let r = self.parse_term()?;
                    left = ExprV0::Binary {
                        op: "+".into(),
                        lhs: Box::new(left),
                        rhs: Box::new(r),
                    };
                }
                Tok::Minus => {
                    self.next();
                    let r = self.parse_term()?;
                    left = ExprV0::Binary {
                        op: "-".into(),
                        lhs: Box::new(left),
                        rhs: Box::new(r),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }
    fn parse_term(&mut self) -> Result<ExprV0, String> {
        let mut left = self.parse_factor()?;
        loop {
            match self.peek() {
                Tok::Star => {
                    self.next();
                    let r = self.parse_factor()?;
                    left = ExprV0::Binary {
                        op: "*".into(),
                        lhs: Box::new(left),
                        rhs: Box::new(r),
                    };
                }
                Tok::Slash => {
                    self.next();
                    let r = self.parse_factor()?;
                    left = ExprV0::Binary {
                        op: "/".into(),
                        lhs: Box::new(left),
                        rhs: Box::new(r),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }
    fn parse_factor(&mut self) -> Result<ExprV0, String> {
        match self.next() {
            Tok::Int(v) => Ok(ExprV0::Int {
                value: serde_json::Value::from(v),
            }),
            Tok::LParen => {
                let e = self.parse_expr()?;
                match self.next() {
                    Tok::RParen => Ok(e),
                    _ => Err(") expected".into()),
                }
            }
            _ => Err("factor expected".into()),
        }
    }
}

pub(super) fn parse_source_v0_to_json(input: &str) -> Result<String, String> {
    let toks = lex(input)?;
    let mut p = P::new(toks);
    let expr = p.parse_program()?;
    let prog = ProgramV0 {
        version: 0,
        kind: "Program".into(),
        body: vec![StmtV0::Return { expr }],
    };
    serde_json::to_string(&prog).map_err(|e| e.to_string())
}

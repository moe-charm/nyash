# ANCP Transcoder v1 トークン仕様（EBNF＋運用ルール）

Author: ChatGPT5  
Date: 2025-09-03  
Version: 1.0  

> まずは「P=Pretty（正典）→C=Compact（ANCP ASCII/Unicode）」の**トークン変換**が安全に往復できる最小コア。

## 1) レクサの前提

* 入力は UTF-8。**ASCIIモード**と**Unicodeモード**を切替（既定=ASCII）。
* 変換は**トークン列→トークン列**。**文字列/コメント/正規表現**内部は**絶対に変換しない**。
* 空白/改行/コメントは**隣接トークン間のメタに付与**して保持（ソースマップ2.0）。

### 1.1 トークンクラス（共通）

```ebnf
Identifier   = IDStart IDContinue* ;
IDStart      = Letter | "_" ;
IDContinue   = IDStart | Digit ;
Digit        = "0"…"9" ;
Letter       = "A"…"Z" | "a"…"z" | NonAsciiLetter ;

IntLiteral   = Digit+ ;
FloatLiteral = Digit+ "." Digit+ (ExponentPart)? | "." Digit+ (ExponentPart)? ;
ExponentPart = ("e"|"E") ("+"|"-")? Digit+ ;

StringLit    = '"' ( Escape | ~["\r\n] )* '"' 
             | "'" ( Escape | ~['\r\n] )* "'" ;
Escape       = "\\" . ;

RegexLit     = "/" ( Escape | ~[/\r\n] )+ "/" [a-z]* ;   // Pのみ（Cでは素通し）

CommentLine  = "//" ~[\r\n]* ;
CommentBlock = "/*" .*? "*/" ;              // ネスト不可（Phase1）

WS           = Space | Tab ;
NL           = "\r"? "\n" ;
```

## 2) 予約語と記号マップ（P→C）

**衝突しないASCII記号**を採用。Unicodeモードは `→` の右側を `uni` 欄で置換。
**識別子と区別**するため、`~x` 形は**先頭に `~`**を付ける（通常のIDに現れにくい）。

| 機能 | Pretty(P) | Compact(C ascii) | Compact(C uni) |
|------|-----------|------------------|----------------|
| Box定義 | `box` | `$` | `＄` |
| 新規生成 | `new` | `~n` | `ⁿ` |
| 自参照 | `me` | `m` | `ｍ` |
| 局所変数 | `local` | `~l` | `ˡ` |
| 戻り | `return` | `~r` | `↩` |
| 継承/委譲 | `from` | `@` | `＠` |
| 初期化 | `init` | `#` | `＃` |
| コンストラクタ | `birth` | `b` | `ᵇ` |
| 静的 | `static` | `S` | `Ｓ` |
| 条件 | `if` | `?` | `？` |
| else | `else` | `:` | `：` |
| ループ | `loop` | `~L` | `ᴸ` |
| 継続 | `continue` | `~c` | `↻` |
| 分岐peek | `peek` | `~p` | `ᵖ` |

> 予約域：`~[A-Za-z]` は**将来予約**で識別子に使えないことを仕様化。

## 3) 演算子・糖衣（P↔C 等価）

* パイプ |>: `a |> f(x)` → **そのまま**（記号は等価、空白最小化のみ）
* セーフアクセス ?.: `o?.f` → **そのまま**
* ディレクティブ /:: `/: name` → **そのまま**（意味を壊さず最小化）

## 4) セパレータ・自動挿入規約

* **C出力**時、**記号トークンの左右に英数IDが隣接**する場合は**1スペース**を強制挿入（`m$X` の誤読防止）。
* セミコロンは P 側の規約準拠。C では**危険箇所のみ挿入**（§6の「ASI判定」参照）。

## 5) 変換アルゴリズム（疑似コード）

```text
encode(P → C):
  lex P → tokens[]
  for t in tokens:
    if t in (StringLit, Comment*, RegexLit): emit t (verbatim); continue
    if t is Keyword and t.lexeme in MAP: emit MAP[t.lexeme] as SymbolToken
    else emit t (with WS-minify rules)
  apply ASI (only-when-necessary)
  attach inter-token trivia to sourcemap

decode(C → P):
  lex C → tokens[]
  for t in tokens:
    if t is SymbolToken and t.lexeme in INV_MAP: emit INV_MAP[t.lexeme] as Keyword
    else emit t
  restore WS/comments by sourcemap if available
```

## 6) ASI（セミコロン自動挿入）判定（最小）

**挿入する**条件（どれか）：

1. 次トークンが `}` / EOF
2. 現トークンが `return (~r) / continue (~c) / break` 等で、**直後が行末**
3. 構文上、次トークンが**先頭に来るべき**（例えば次が `box/$` 定義）

**挿入しない**：

* 行末でも、次トークンが `(` `[` `{` `.` `?.` `/:` のとき

## 7) EBNF（P→C 変換で必要なサブセット）

**目的**：可逆のための**字句と一部構文の境界**を定義。完全文法ではなく、トークン接合規則に必要な核のみ。

```ebnf
Program      = WS_NL* (Stmt WS_NL*)* ;

Stmt         = BoxDecl
             | LocalDecl
             | ReturnStmt
             | ExprStmt
             ;

BoxDecl      = "box" Identifier BoxBody ;
BoxBody      = "{" (MemberDecl WS_NL*)* "}" ;

MemberDecl   = ( FieldDecl | MethodDecl | StaticDecl ) ;

FieldDecl    = ( "init" | "#" ) Identifier ( "=" Expr )? ";"? ;
MethodDecl   = Identifier ParamList Block ;
StaticDecl   = ( "static" | "S" ) MethodDecl ;

LocalDecl    = ( "local" | "~l" ) Identifier ( "=" Expr )? ";"? ;

ReturnStmt   = ( "return" | "~r" ) Expr? ";"? ;

ExprStmt     = Expr ";"? ;

Expr         = AssignExpr ;
AssignExpr   = OrExpr ( AssignOp OrExpr )? ;
AssignOp     = "=" | "+=" | "-=" | "*=" | "/=" ;

OrExpr       = AndExpr ( "||" AndExpr )* ;
AndExpr      = PipeExpr ( "&&" PipeExpr )* ;

PipeExpr     = TernaryExpr ( "|>" CallLike )* ;

TernaryExpr  = NullsafeExpr ( "?" Expr ":" Expr )? ;

NullsafeExpr = MemberExpr | MemberExpr "?." Identifier | MemberExpr "/:" Identifier ;

MemberExpr   = Primary ( ("." | "[") ... )? ;   // 省略（可逆に影響しない部分）

CallLike     = Identifier | Call ;

Call         = Identifier "(" ArgList? ")" ;
ArgList      = Expr ("," Expr)* ;

Primary      = Identifier
             | Literal
             | "(" Expr ")"
             ;

Literal      = IntLiteral | FloatLiteral | StringLit | RegexLit ;

Identifier   = see §1.1 ;
```

> **ポイント**
> * `FieldDecl` は `init` と `#` を等価扱い（Cでは `#` に寄せる）
> * `StaticDecl` は `static` と `S` を等価
> * `LocalDecl` は `local` と `~l` を等価
> * `ReturnStmt` は `return` と `~r` を等価
> * `box` は `$` と等価（`BoxDecl`）

## 8) ソースマップ2.0（トークン粒度）

* **単一フォーマット（JSON Lines 推奨）**：各出力トークンに**元トークン範囲**と**トリビア**を付与。

```json
{"out_i":42,"out_span":[l1,c5,l1,c6],"in_file":"foo.ny","in_span":[l10,c1,l10,c3],"trivia":{"lead":" ","trail":""}}
```

* 例外/ログは**BoxID + トークン範囲**で P へ逆引き。

## 9) 衝突回避ルール（最重要）

* **ASCIIモード**：`~[A-Za-z]` は**保留記号**。Identifier と**絶対に一致しない**。
* **記号の周囲**：`$ m` のように**必要時1スペース**（前後が英数IDの場合）。
* **文字列/コメント/Regex**：**一切変換せず** verbatim。

## 10) 例（往復保証）

**P (Pretty)**

```nyash
box NyashCompiler {
  compile(source) {
    local ast = me.parse(source)
    local mir = me.lower(ast)
    return me.codegen(mir)
  }
}
```

**C (Compact ASCII)**

```
$ NyashCompiler{
  compile(src){
    ~l ast=m.parse(src)
    ~l mir=m.lower(ast)
    ~r m.codegen(mir)
  }
}
```

**decode(C) → P** は上記Pと**等価**（空白/改行はソースマップで復元）。

---

## 実装メモ（すぐ書ける骨組み）

* レクサは **状態機械**：`DEFAULT / STRING / REGEX / COMMENT`
* 置換は**辞書マッチ → 最長一致**（`box`→`$` を `Identifier` と衝突させない）
* 出力時に**区切り挿入規則**を適用：`need_space(prev, next)`
* ASI は §6 の規則のみ実装（Phase1）。曖昧時は**セミコロン挿入を選ぶ**。

---

これで **Phase 12.7-A（Week1）** の「P↔C 可逆・安全」まで一気に行けるにゃ。

次にやるなら：この仕様をそのまま基に**トークナイザのテストケース**（OK/NG 30本）を並べよう。
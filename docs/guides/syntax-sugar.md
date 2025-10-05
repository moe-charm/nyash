# Syntax Sugar Guide (Phase 12.7+)

## Overview
- Default: ON（dev/prod 共通）。全体切替: `NYASH_SYNTAX_SUGAR_LEVEL={off|basic|full}`（未設定=ON）。
- Scope: すべて「正規化（desugar）」のみ。意味論は変わらず Core 形へ降下してから実行されます。

## PreLex（前正規化・共通）
- ランナー共通のトークナイズ前処理（ON時）。VM/LLVM/PyVM で統一。
- 正規化するもの:
  - Raw strings: `r"…"`, `r#"…"#`, `r##"…"##` → 通常文字列トークン（内容はそのまま）
  - Numeric separators: 数値リテラルの `_` を除去（例: `1_000_000` → `1000000`）
  - 行頭dev糖衣: `@name[:Type] = expr` → `local name[:Type] = expr`
- 備考: PreLex はトークン安定化のみ（意味論に影響なし）。

## Levels
- off: すべての糖衣を無効化
- basic: `|>` / `??` / `?.` / `..` / 配列/Map リテラル / 末尾カンマ / 数値セパレータ（Mapの識別子キーは不可）
- full: basic + パイプ受信者糖 `.m(...)` + パイプ `_` プレースホルダ + Raw Strings + Mapの識別子キー許可

## Pipeline `|>`（basic/full）
Examples
```nyash
x |> f(a,b)        # → f(x, a, b)
x |> obj.m(a)      # → obj.m(x, a)
x |> .m(a)         # → x.m(a)  （受信者糖）
x |> f(_, k)       # → f(x, k) （_ を x で1箇所だけ置換）
```
Rules
- `_` は 0 か 1 回のみ。2 回以上は Fail‑Fast（構文エラー）。
- `_` が無い場合、`x` は RHS 呼び出しの第1引数に注入。
- 優先順位: 関数適用/ドットが `|>` より強く結合（`x |> f(a).g()` は `g(f(x,a))`）。

## Optional chaining `?.` / Coalesce `??`（basic）
```nyash
a?.b        # a が null の時は null、それ以外は a.b
x ?? y       # x が null の時は y、それ以外は x
```
Lowering
- `a?.b` → `match a { null => null, _ => a.b }`
- `x ?? y` → `match x { null => y, _ => x }`

## Raw Strings（full）
```nyash
r"C:\\path\\file.txt"      # バックスラッシュ等をそのまま
r#"{"key": "value"}"#     # # でクオートをネスト
r##""" triple "quotes" """##
```
Notes
- 閉じは 開きと同数の `#` + `"`。内容はエスケープしない（PreLex で通常文字列へ）。

## Trailing Commas（basic）
- 配列/Map/引数で末尾カンマを許可: `f(a,b,)`, `{k:1,}`, `[1,2,]`。

## Numeric Separators（basic）
- 整数/浮動小数で `_` を許可: `1_000_000`, `3.141_592`（PreLex で削除）。

## Env/CLI（運用）
- `NYASH_SYNTAX_SUGAR_LEVEL={off|basic|full}`（未設定=ON）。同義として `on|1|true` も受理。
- 参考: PreLex 実装は `src/runner/modes/common_util/prelex.rs`。ランナー入口で共通適用。
- 備考: パーサ側にもフォールバック（raw文字列/数値セパレータなど）を用意しており、PreLex未適用の単体パースでも安定動作します。

## 実装メモ
- 構文ガードは Parser 側（Expressions）で実装し、誤用は Fail‑Fast にします（例: `_` 多重など）。
- バックエンド（VM/LLVM/PyVM）は正規化後の Core 形のみを扱うため、糖衣が ON/OFF でも意味論は一致します。


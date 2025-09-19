# LoopForm ガイド — ループ正規化（ユーザーマクロ）

目的
- while/for/foreach を Nyash のユーザーマクロで“キャリア（タプル）”に正規化し、MIR/LLVMにとって最適化しやすい形へ落とし込む。

使い方（予定）
- マクロ登録（例）:
```
export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS=apps/macros/examples/loop_normalize_macro.nyash
```
- 自己ホスト前展開（auto）を利用して、parse直後にLoopForm展開を有効化（PyVM環境）。

JSON生成ユーティリティ（JsonBuilder）
- ループ正規化では AST JSON v0 の断片を安全に構成する必要があります。
- 最小ユーティリティとして `apps/lib/json_builder.nyash` を提供しています（includeで読み込み、文字列でJSON断片を生成）。
- 例:
```
local JB = include "apps/lib/json_builder.nyash"
local v_i   = JB.variable("i")
local v_sum = JB.variable("sum")
local lit_0 = JB.literal_int(0)
local assign = JB.assignment(v_i, JB.binary("+", v_i, JB.literal_int(1)))
```

正規化の考え方
- ループで更新される変数群をタプルに束ね、ヘッダに“1個のφ”を置く。
- break/continue は“次キャリア”または“現キャリア”で遷移し、一貫した合流点を保つ。

対応状況（MVP→順次拡張）
- Week1: while（break/continue無し）
- Week2: break/continue/ネスト最小対応、キャリア自動抽出
- Week3: for/foreach（限定）

制約（MVP）
- try/finally/throwとの相互作用は未対応。例外は後続フェーズで設計を明記。

検証
- macro‑golden（展開後ASTのゴールデン）
- LLVM PHI健全性スモーク（空PHI無し、先頭グループ化）

参考
- docs/development/roadmap/phases/phase-17-loopform-selfhost/

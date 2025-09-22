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

キャリア正規化（MVP-2）の具体例
- 前提: break/continue なし、更新変数は最大2個（例: i と sum）。

例1: 基本的な while（i を 0..n-1 で加算）

Nyash（入力・素朴形）
```
local i = 0
local sum = 0
while (i < n) {
  sum = sum + i
  i = i + 1
}
```

正規化の狙い（概念）
- ループ本体の末尾に更新（Assignment）をそろえる（既に末尾ならそのまま）。
- ループヘッダの合流で i と sum を同一グループの PHI でまとめやすくする。

AST JSON v0 のスケッチ（JsonBuilder を用いた生成例）
```
local JB   = include "apps/lib/json_builder.nyash"
local v_i  = JB.variable("i")
local v_s  = JB.variable("sum")
local v_n  = JB.variable("n")

// 先頭（ローカル導入）
local i0   = JB.local_decl("i", JB.literal_int(0))
local s0   = JB.local_decl("sum", JB.literal_int(0))

// 条件と本体（更新は末尾に揃える）
local cond = JB.binary("<", v_i, v_n)
local body_nonassign = JB.block([
  JB.assignment(v_s, JB.binary("+", v_s, v_i))
])
local body_updates   = JB.block([
  JB.assignment(v_i, JB.binary("+", v_i, JB.literal_int(1)))
])

// ループノード（MVP: while→Loop; キャリアは概念上）
local loop_node = JB.loop(cond, JB.concat_blocks(body_nonassign, body_updates))

JB.program([ i0, s0, loop_node ])
```

備考
- MVP-2 では“新しいキャリア用ノード”は導入せず、既存の Local/If/Loop/Assignment で表現する。
- 「非代入→代入」の順を崩すと意味が変わる可能性があるため、再配置は安全にできる場合のみ行う（既に末尾に更新がある等）。

例2: 2変数更新の順序混在（安全な並べ替え）
```
while (i < n) {
  print(i)
  sum = sum + i
  i = i + 1
}
```
- 正規化: 非代入（print）→ 代入(sum) → 代入(i) の末尾整列。
- 非代入が末尾に来るケースは再配置しない（意味が変わりうるため、スキップ）。

今後の拡張（MVP-3 概要）
- continue: 「次キャリア」へ（更新後にヘッダへ戻す）。
- break: 「現キャリア」を exit へ（ヘッダ合流と衝突しないよう保持）。
- いずれも 1 段ネストまでの最小対応から開始。

MVP-3（実装済み・最小対応）
- 本体を break/continue でセグメント分割し、各セグメント内のみ安全に「非代入→代入」に整列。
- ガード:
  - 代入先は変数のみ（フィールド等は対象外）
  - 全体の更新変数は最大2種（MVP-2 制約を継承）
  - セグメント内で「代入の後に非代入」があれば整列しない（順序保持）
- スモーク:
  - `tools/test/smoke/macro/loopform_continue_break_output_smoke.sh`

for / foreach の糖衣と正規化（概要）
- for: `for(fn(){ init }, cond, fn(){ step }, fn(){ body })` を `init; loop(cond){ body; step }` へ正規化。
  - init/step は `Assignment`/`Local` 単体でも可。
- foreach: `foreach(arr, "x", fn(){ body })` を `__ny_i` で走査する Loop へ正規化し、`x` を `arr.get(__ny_i)` に置換。
- スモーク: `tools/test/smoke/macro/for_foreach_output_smoke.sh`

対応状況（MVP→順次拡張）
- Week1: while（break/continue無し）
- Week2: break/continue/ネスト最小対応、キャリア自動抽出
- Week3: for/foreach（限定）

制約（MVP）
- try/finally/throwとの相互作用は未対応。例外は後続フェーズで設計を明記。

検証
- macro‑golden（展開後ASTのゴールデン）
- LLVM PHI健全性スモーク（空PHI無し、先頭グループ化）
 - 出力一致スモーク（two‑vars の実行出力が同一であること）

手元での確認
- ゴールデン（キー順無視の比較）
  - `tools/test/golden/macro/loop_simple_user_macro_golden.sh`
  - `tools/test/golden/macro/loop_two_vars_user_macro_golden.sh`
 - 出力一致スモーク（VM）
   - `tools/test/smoke/macro/loop_two_vars_output_smoke.sh`
 - 自己ホスト前展開（PyVM 経由）
  - `NYASH_VM_USE_PY=1 NYASH_USE_NY_COMPILER=1 NYASH_MACRO_ENABLE=1 NYASH_MACRO_PATHS=apps/macros/examples/loop_normalize_macro.nyash ./target/release/nyash --macro-preexpand --backend vm apps/tests/macro/loopform/simple.nyash`

Selfhost compiler prepass（恒等→最小正規化）
- Runner が `NYASH_LOOPFORM_NORMALIZE=1` を `--loopform` にマップして子に渡し、`apps/lib/loopform_normalize.nyash` の前処理を適用（現状は恒等）。
- 既定OFF。将来、キー順正規化→簡易キャリア整列を段階的に追加する。

実装メモ（内蔵変換ルート / Rust）
- 既定のマクロ実行は internal‑child（Rust内蔵）です。LoopNormalize は以下の保守的なガードで正規化します。
  - トップレベル本体に Break/Continue がないこと
  - 代入対象は最大2変数、かつ単純な変数（フィールド代入などは除外）
  - 代入の後ろに非代入が現れない（安全に末尾整列できる）
- 条件を満たす場合のみ「非代入→代入」の順でボディを再構成します（意味は不変）。

参考
- docs/development/roadmap/phases/phase-17-loopform-selfhost/

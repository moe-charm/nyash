# If/Match 正規化ガイド（Control-Flow Normalize）

目的
- If と Match を“合流点が明確な制御フロー”へ正規化し、MIR/LLVM が一貫した PHI 生成を行えるようにする。
- ルールをシンプルに保ち、LoopForm（キャリア）と相性良く動作させる。

推奨ビルダー（コンパイル時メタ）
- ControlFlowBuilder: If/Match の正規化（join 変数・If 連鎖生成）。
- PatternBuilder: パターン条件（==/OR/AND/型チェック/デフォルト）を構築。

適用タイミング
- マクロ前展開パスの中で、If/Match 正規化 →（必要に応じて）LoopForm 正規化の順で適用するのが基本。
- ループ本体に If/Match が含まれていても、If/Match 正規化は局所的に完結する（LoopForm と独立に安全）。

正規化ポリシー（If）
- 条件式を一度だけ評価し、`cond` ローカル（gensym）へ束ねる。
- then/else の両方で結果が必要な場合（式コンテキスト）は、`res` ローカル（gensym）を導入して各枝で代入し、合流ブロックの先頭で PHI に集約される形を誘導する。
- 代入や副作用の順は保存する（順序変更はしない）。

例（式 If の正規化）
```
// 入力
local x = if (a < b) { 10 } else { 20 }

// 概念的な正規化（AST JSON v0 を生成するマクロ）
local cond = (a < b)
local res
if (cond) {
  res = 10
} else {
  res = 20
}
local x = res
```

正規化ポリシー（Match）
- スクルーティニー（`match <expr>` の `<expr>`）を一度だけ評価し、`scrut` ローカル（gensym）へ束ねる。
- 各アームのパターンは If 連鎖へ合成する：
  - リテラル: `scrut == lit1 || scrut == lit2 || ...`
  - 型パターン: `type_check(scrut, T)`（必要なら `as/cast` を then 側で実行）
  - ガード: パターン条件と `&&` で合成（短絡規約に従う）
- デフォルト（`_`）は最後に配置。式コンテキストでは `res` ローカルを用いて各アームで代入し、合流で 1 個の PHI に収束させる。

例（ガード付き Match の正規化概念）
```
// 入力
local msg = match x {
  0 | 1 if small => "small",
  IntegerBox(n)  => n.toString(),
  _              => "other",
}

// 概念的な正規化
local scrut = x
local res
if ((scrut == 0 || scrut == 1) && small) {
  res = "small"
} else if (type_check(scrut, IntegerBox)) {
  // 必要なら as/cast を then 内で行う
  res = toString(as_int(scrut))
} else {
  res = "other"
}
local msg = res
```

PHI と合流の不変条件
- PHI は合流ブロックの“先頭”にのみ現れる構造を誘導する。
- 式 If/Match は必ず `res` ローカルへ各枝で代入してから合流させる（空 PHI が出ない）。
- 条件・スクルーティニーは 1 回評価（gensym ローカル）。

実装メモ（MVP）
- 本ガイドはユーザーマクロ（Nyash）で AST JSON v0 を編集する前提（既定で有効化）。
- 既存の Match→If 連鎖はパーサ段階でも行っているが、正規化マクロは“式コンテキストの合流（res ローカル）”や“型チェック/ガード合成の一貫性”を担保する。
- 先に If/Match 正規化、その後に LoopForm（キャリア整形）を行うと、LLVM 側の PHI が安定する。

テストと検証
- ゴールデン: 入力→展開後 AST JSON を正規化比較（キー順非依存）
- PHI スモーク: If/Match 正規化後のプログラムで「空 PHI の不在、ブロック先頭配置」を確認

関連ドキュメント
- docs/guides/loopform.md
- docs/reference/ir/ast-json-v0.md

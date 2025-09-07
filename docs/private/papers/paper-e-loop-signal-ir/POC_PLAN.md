# LoopForm IR / LifeBox Model — PoC計画（安全な縦スライス）

## コア要件（不変条件）
- ループ正規形: すべての制御は `loop.begin → loop.iter → loop.branch → loop.end` の列で現れ、dispatchブロックで合流（PHIはdispatch直後のみ）。
- Signal整合性: `LoopSignal<T> = Next/Break/Yield/Return` は i8タグ＋単一SSAペイロード。4タグは厳密相互排他。
- 単一継続点: Loopごとに1出口（break/return合流点）。Loop1は必ず即時畳み込み（inline）でゼロコスト化。
- 可逆性: LoopForm → Core‑13 への逆Loweringは常に可能（フラグでON/OFF）。

## IR最小仕様（再掲）
- Signal: 表層 `LoopSignal<T>` / 実体 `{ i8 tag, T payload }`（tag=0/1/2/3）。
- 命令4種（中間正規形）:
  - `loop.begin %lp`（メタ: ソース位置/ID）
  - `loop.iter   %sig, %lp, %state` — LoopBox 1反復を実行して Signal を返す
  - `loop.branch %sig { onNext: L1, onBreak: L2, onYield?: L3, onReturn?: L4 }`
  - `loop.end    %lp`
- LoopBox契約: どのBoxも `init → step(state) → fini`。通常Boxは `step` が1回で `Break(result)`（=Loop1）。

## 逆Lowering規則（抜粋）
- scope: Loop1（iter→即Break→end）→ 直列化
- while: iter→Nextで継続、Breakで終了（dispatch合流）→ br/jump へ
- if: Loop1（条件→ next/break(path)）→ branch+phi へ
- return: `Break(Return v)` に統一→最上位dispatch合流で `Return v` へ

## PoC範囲
- 命令セット: 4命令 + Signal型のみ。対象は while / scope / if（Loop1）に限定。
- 逆Lowering: Core‑13へ完全可逆（VM/Interpreter 経路の実行一致）。
- 最適化: Loop1インライン畳み込み（Yieldなしのstate省略、dispatch一点合流）を先行実装。

## 受入基準
- 構造: dispatch合流点のみでPHI（Printer/Verifierで検出可能）。
- 結果一致: 5スモーク（const/return, add, branch+phi, while, nested-branch）が現行MIRと一致。
- トレース: covered/unsupported と decisions(allow,fallback) をLoopForm/逆Lowering双方で採取できる。

## 拡張の切り口（後続）
- 関数/returnの完全Loop化: ReturnをSignalに残すか、Loop外のReturn命令に落とすかをflag化。
- generator/async: YieldをSignalに昇格。state＝継続スロットを LoopBox側で保持（状態箱を1st-classに）。
- 効果モデル: effect lattice（Pure/RO/Side）をSignalに乗せ、dispatchでbarrier位置を決めやすく。
- AOT/JSON v0: mir_version/extensionsでLoopFormは“中間正規形”として明示。

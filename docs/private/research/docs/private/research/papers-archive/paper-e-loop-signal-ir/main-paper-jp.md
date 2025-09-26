# LifeBox Model と LoopForm IR: Box指向実行系における制御の値化と統一

> 統一直観（融合案）: 「Everything is Box（空間）」×「Everything is Loop（時間）」
>
> すべての箱は“ループ1回（Loop1）の箱”として捉えられ、制御は LoopSignal（Next/Break/Yield[/Return]）という値で運ばれる。

## 概要

本稿は、Nyashの「Everything is Box」を拡張する概念 LifeBox Model（LBM: Box=Loop1）と、その思想をIR上で実現する LoopForm IR（別名: LoopSignal IR）を提案する。分岐・関数・スコープ・反復・ジェネレータ・async を「Loop=反復の箱」に正規化し、制御結果を値（Signal）として扱う。IRレベルでは `loop.begin / loop.iter / loop.branch / loop.end` による標準ディスパッチ形を導入する。これにより、Front-endのLoweringが一様化し、CFGの合流点が定型化（dispatchブロックへ集約）され、PHIの整理・最適化（DCE/LICM/インライン）が素直になる。Loop1 は完全インライン化によりオーバーヘッドを排除可能である。

なお、LoopFormは中間正規形であり、最終的にはCore‑13 IR（固定13命令）に再Loweringできることを前提とする（後方互換を常時確保）。

本稿はコンセプト草稿であり、論文A（MIR13/IR設計）の後、短論文/ワークショップ稿として展開する。

### Executive Summary（1ページ要約）
- 目的: if/while/for/scope/function/generator/async を LoopBox+LoopSignal に正規化し、Lowering/最適化/拡張を単純化する。
- 中核: Signal（Next/Break/Yield[/Return]）をタグ付き値で表し、`loop.begin/iter/branch/end` の4命令で統一ディスパッチ。
- 融合: 「普通の箱」も Loop1 として扱い（init/step/fini）、スコープ=Loop1 を基本単位にする。
- 実利: 合流点の定型化により PHI/支配木が単純化。Loop1はインライン畳み込みでオーバーヘッドゼロ。generator/asyncは Signal 拡張のみで実装可能。

## 1. 背景と動機

- 現行MIR13は、BoxCall統一によりデータ操作を簡素化した。一方で制御構造（if/loop/call/return/scope）は表現が分散し、Loweringと最適化で個別処理が残る。
- 制御を「値化（Signal）」し、Loopの反復単位で標準ディスパッチすることで、表現・Lowering・最適化・拡張（generator/async/effect）のすべてを一様化できる。

## 2. 提案: LoopSignal IR の最小仕様

- Signal 型: `LoopSignal<T> = Next(T) | Break(T) | Yield(T) [| Return(T)]`
- 擬似命令（IR注釈）:
  - `loop.begin %id`
  - `loop.iter %sig, %loop, %state`  // 反復1回実行（LoopBoxの一回分）
  - `loop.branch %sig { onNext: L1, onBreak: L2, onYield: L3 }`
  - `loop.end %id`
- LoopBox: 反復1回の本体。入力 `state` を受け `LoopSignal<...>` を返す小さな「箱」。

### 2.1 型の実体化（LLVM想定）
```llvm
%LoopSignal_T = type { i8 /*tag:0=Next,1=Break,2=Yield,3=Return*/, T /*payload*/ }
; switch %tag で L_next/L_break/L_yield/L_return へ分岐
```

### 2.2 Box=Loop1 の取り決め
- どの Box も `init -> step -> fini` を持つとみなせる。
- 「通常の箱」は `step()` が 1 回で `Break(result)` を返す（= Loop1）。
- RAII/using/defer は `init/fini` に収容され、`loop.begin/end` に対応付ける。

## 3. 正規化（Lowering）規則（例）

### 3.0 記法: 二層構造（表記と内部）

表記（従来構文）と内部（LoopForm）を併存させ、内部では常にLoopFormへ正規化する。

```
// 従来: 複雑な分岐
if (x) { a() } else { b() }
while (y) { c() }
func() { d() }

// ループ統一: シンプル（内部正規形のイメージ）
loop { x ? break(a()) : next }
loop { y ? next : break }
loop { break(d()) }

// 「複雑に見えるのは慣れの問題」→ 表記は従来も提供、内部はLoopで統一
```

### 3.1 スコープ `{ ... }` → Loop1
```
{
  x = 1
  y = 2
}
```
↓
```mir
%lp = loop.begin
%s0 = Const(void)
%sg = loop.iter %lp, %s0
loop.branch %sg { onBreak: Lb, onNext: Ln }
Ln: jump Ld
Lb: loop.end %lp; jump Ld
Ld: ; 続き
```

解釈: Blockを「1回だけ回る Loop」と見なし、`iter` は即時 `Break` を返す。

### 3.2 while ループ
```mir
%lp = loop.begin
%s  = Const(init)
Lh:
  %sg = loop.iter %lp, %s
  loop.branch %sg { onNext: Lh, onBreak: Lb }
Lb:
  loop.end %lp
```

### 3.3 for-in（LoopBox化の雛形）
```mir
func LoopBox_for(state: ForState, env: Env) -> LoopSignal<ForState> {
  %x, %st' = iter_next(state)
  br_if %x==None -> return Break(state)
  call body(%x, env)
  return Next(%st')
}

%lp = loop.begin
%sig = loop.iter %lp, %init
Ldisp:
  loop.branch %sig { onNext: Lnext, onBreak: Lbr }
Lnext:
  %st = extract_next(%sig)
  %sig = loop.iter %lp, %st
  jmp Ldisp
Lbr:
  %fin = extract_break(%sig)
  loop.end %lp
  ret finalize(%fin)
```

### 3.4 関数呼び出し/return → Loop1
`return v` ≒ `Break(Return v)` として統一可能（実装ではReturnをSignalに含めるか分離を選択）。

### 3.5 generator/async
`Yield` をSignalに含め、`onYield` 分岐でハンドリング。状態は `state` に保持。

## 4. 実装方針（安全な導入順序）

1) LoopSignal 型と `loop.*` ノードをMIRに追加（オプトイン）

2) while/for/スコープのみ LoopForm にLowering（if/関数は従来）

3) LoopForm → 従来MIRへの逆Loweringパス実装（常時オフに戻せる）

4) 最適化: Loop1 の完全インライン化、Yieldなしの状態保存省略、支配関連の簡略解析

5) 拡張: generator/async → effect（必要なら）

6) フォールバック: 互換性のため LoopForm→従来MIR への逆Loweringを維持（フラグで切替）。

## 4.1 コア要件（不変条件 / Core Invariants）

- ループ正規形: すべての制御は `loop.begin → loop.iter → loop.branch → loop.end` の列で現れ、dispatchブロックで合流（PHIはdispatch直後のみ）。
- Signal整合性: `LoopSignal<T> = Next/Break/Yield/Return` は i8タグ＋1ペイロード（SSA値）。4タグは相互排他。
- 単一継続点: 各Loopは1つの出口（break/return合流点）を持つ。Loop1は必ず畳み込み可能（inlineでゼロコスト）。
- 可逆性: LoopForm → Core‑13（現行MIR）への逆Loweringが常に可能（フラグでON/OFF安全導入）。

## 5. 評価計画

- 表現の簡潔さ: ブロック数・分岐数・PHI数の変化
- コンパイラ時間: Lowering/最適化の時間（±）
- 実行性能: VM/JIT/AOTで従来MIRとの差（同等〜微差で良い）。主眼は実装簡易性。
- 拡張容易性: generator/asyncの実装コード量・変更影響の定量化

### 5.1 最初のPoC課題（段階的）
1. `while(true){break}` を Loop1 に変換 → 命令数/PHI/実行時間を計測
2. `for-in` をステートマシン化 → Next/Break 型で VM/AOT の一致を検証
3. `yield` 付き最小ジェネレータ → 状態保持と再開の健全性（再現テスト）

### 5.2 受入基準（Acceptance）
- MIR構造: dispatch合流点のみでPHI（Printerで自明にわかる）
- 差分実行: 5スモーク（const/return, add, branch+phi, while, nested-branch）が現行MIRとVM結果一致
- 観測: covered/unsupported と decisions(allow,fallback) をLoopForm/逆Lowering双方で採取

## 6. 関連研究と差分

- CPS/継続: 制御を関数化するが、LoopSignalは「値としての制御」をMIRで直接扱う点が異なる（CPS変換を前提にしない）。
- 代数的効果/ハンドラ: 近いが、最小Signal集合＋loop.*に限定し実用性を優先。
- コルーチン/ジェネレータ: 専用機構に閉じず、関数/スコープまでLoop1として統一。
- Smalltalk/Lispの統一思想: 本稿は制御フローの大統一をMIRで具体化。

補足: MLIR/Swift SIL/Rust MIR 等のSSA系IRと比較すると、本稿は「制御（時間）側の正規化」をMinimal命令で達成し、最適化・実装コストの低減を主眼に置く。

## 7. 限界と今後

- 説明コスト: 初学者負荷。導入は while/for のみから段階的に。
- オーバーヘッド: Loop1は必ず即時畳み込み（インライン化）で無害化。
- デバッグ: loop.begin にソース範囲・ID埋め込み、iterで行情報維持。

---

付録：名前候補（査読者向け）
- Signal Loop IR（SLIR）/ LoopSignal Form（LSF）/ Boxed Loop Semantics（BLS）/ LoopBox IR

## 8. 外部レビュー統合と確定方針（Claude/Gemini 要点）

本案に対する外部AIレビュー（claude_output.md / gemini_output.md）の要点を統合し、当面の確定方針を示す。

- 型表現（LoopSignal<T>）: Next/Break/Yield/Return を採用（Geminiの Continue 提案は命名差。既存説明と整合する Next を採用）。タグは i8 固定、ペイロードはSSA値。
- 返却の扱い: Return を Signal に含める「統一モデル」を採用（Claude/Gemini一致）。IR直交性と最適化容易性を優先。
- MIR命令の厳密化: loop.begin / loop.iter / loop.branch / loop.end の4命令でLoopFormを定義。PHI配置点は loop.begin（合流標準形）に確定。
- LLVM/ABI: まずは { i8 tag, payload } の素直なstruct表現で実装し、ホットパス（Next）を既存最適化に委ねる。必要なら将来特殊化を導入。
- 逆Lowering（互換性）: LoopForm→従来MIRへの逆変換パスをフラグでON/OFF可能に（--no-loop-signal-ir）。段階的導入で安全性を担保。
- 最適化パス: 1) Loop1完全インライン化 2) Yieldなし状態省略 3) 分岐合流点正規化（dispatch集中）を優先実装。既存DCE/LICM/Inlineとの整合を確認。
- 適用範囲の段階化: while/for/scope から導入（関数/ifは後追い）。例外・effects（代数的効果/ハンドラ）は将来拡張として外出し。
- ドキュメント方針: LoopFormは「中間正規形」であり、最終的にCore-13に再Loweringできることを明記（実装も常時オン/オフ可能）。

実装ロードマップ（最小到達順）
1) LoopSignal型・loop.*命令（IRノード）追加（ビルドフラグでオプトイン）
2) while/for/scope のLoweringをLoopFormへ移行（if/関数は現状）
3) 逆Lowering（LoopForm→従来MIR）を完成、デフォルトONで安全運用可に
4) 最適化: Loop1 inline / Yieldなし省略 / dispatch統合の3本
5) 評価: 表現簡潔さ（PHI/ブロック数）、コンパイラ時間、実行性能（同等〜微差で良い）
6) 適用拡大: 関数/if、generator/async（Return/YieldのSignal化）


付録：用語の二元性（覚え方）
- Box（空間）= データ/資源の単位。Instance化で1反復を生む。
- Loop（時間）= 制御/継続の単位。Next/Break/Yield で時間を進める/止める。

# Phase 17: 二本立てIR実行系とモジュール分割（AST共有 → MIR13 / Loop MIR 分岐）

## 目的（Why）
- 文法・予約語・ASTまでは完全共有し、その後のIR層で分岐（MIR13/Core‑13 と LoopForm/Loop MIR）する設計に再編する。
- 解釈器（interp）を各IRに対して同型で用意し、変換を挟まずに意味・性能をA/B比較できるようにする。
- VM/JIT/AOTは共通の実行インタフェース（ExecEngine）で差し替え可能にし、分散開発を容易にする。

## スコープ（Scope）
- 共有フロントエンド: Lexer/Parser/AST/Resolver は現行仕様（文法・予約語を変えない）。
- 二系統IR: Core‑13 MIR（= MIR13）と Loop MIR（= LoopForm IR）。
- 同型解釈層: core13→core13(interp)、loop→loop(interp)。変換は後段。
- 安定IR ABI: 各IRをシリアライズ/デシリアライズ可能にし、CLI/ツールと疎結合化。
- CLI統合: `--engine` と `ir-emit/ir-run/trace` の導線。

## 全体アーキテクチャ（High‑level）
- frontend
  - grammar（文法）/ lexer / parser / AST builder
  - semantic resolver（必要なら）
- lowering
  - `lower_core13(ast) -> Core13Module`
  - `lower_loop(ast)   -> LoopModule`
- ir
  - `nyash-ir-core13`: 型・検証（verify）・正規化（normalize）・serde（json/bin）
  - `nyash-ir-loop`: 型・検証（verify）・serde（json/bin）
- exec (共通トレイト)
  - `nyash-exec-traits`: `ExecEngine`, `TraceSink`, `Value`, `EffectMask`, `Event`
  - `nyash-interp-core13`: Core‑13インタプリタ
  - `nyash-interp-loop`: Loop MIRインタプリタ
  - `nyash-vm` / `nyash-jit` / `nyash-aot`（将来/既存統合）
- runtime
  - `nyash-rt`: BoxCall/ExternCall/GC/FFI等の実体
- cli
  - `nyash`: `run`, `dump-ir`, `exec --engine`, `trace`, `bench`

## AST共有 → IR分岐（設計原則）
- 文法・予約語・AST構造は単一実装で共有（差分なし）。
- AST→MIRはインタフェース分岐のみ：
  - `LoweringFacade` が `LowerCore13` と `LowerLoop` を注入可能な形で生成
  - ASTノード毎に `emit_*` を両実装で提供（if/while/for/return/break/continue/try 等）
- エラー/位置情報: AST→IRの間で `dbg.origin` と `scope_id` を保持し、IR間比較・デバッグに使う。

## ExecEngine インタフェース（安定境界）
- 型
  - `Value`: int/float/bool/ptr/box/none
  - `EffectMask`: R/W/IO/GC/FFI などのビット
  - `Event`: Enter/Exit/Block/PhiMerge/LoopIter/Branch/ExternCall/Safepoint/Barrier/GC/Jit
- トレイト
  - `load_module(ir) -> ModuleHandle`
  - `get_func(m, name) -> FuncHandle`
  - `call(ctx, f, args) -> Result<Value>`
  - `set_tracer(TraceSink)` / `features() -> FeatureMask`
- 切替
  - `nyash run --engine=interp-core13|interp-loop|vm|jit`（同一AST/IRに対し差し替え）

## IR ABI（シリアライズ）
- 目的: ブランチや外部プロセス間でIRを受け渡しし、分散開発・ツール連携を容易に。
- 形式: `json`（可読） + `bin`（MessagePack/bincode 等）。
- バージョン: `schema_version`, `features` を明示。後方互換は“追加のみ”。
- CLI:
  - `nyash ir-emit --ir=core13|loop --format=json|bin -o out`
  - `nyash ir-run  --engine=... < in.ir`
  - `nyash trace --engine=...`（イベント列のダンプ/比較）

## Core‑13 MIR（MIR13）概要
- 命令: Const, BinOp, Compare, Jump, Branch, Return, Phi, Call, BoxCall, ExternCall, TypeOp, Safepoint, Barrier（固定13）
- verify: `phi`配置、効果マスクと`safepoint/barrier`の規則、レガシー命令の禁止
- normalize: クリティカルエッジ分割、可換演算正規化、不要ジャンプ除去、`phi`順序安定化

## Loop MIR（LoopForm IR）概要
- プリミティブ: `loop.begin`, `loop.iter`, `loop.branch`, `loop.end`, `loop.signal(Next|Break|Return|Yield?)`
- 状態: ループキャリア値は `state tuple` として管理、合流は `loop.branch` に集約
- verify: 単一エントリ/単一点帰還、`signal`の終端性、state↔phi対応、例外/非局所脱出はSignal表現

## 2つの解釈器（変換なしで比較）
- `interp-core13`: 基本ブロック/PC/SSA環境、`phi`合流、Box/Extern/Type/Safepoint/Barrier 実装
- `interp-loop`: LoopFrame(state, pc, hdr), `loop.iter/branch/signal` を直接実行
- 共通計測: `steps, blocks, phi_merges, loop_iter, branch_taken, extern_calls, box_allocs, safepoints`
- トレース一致: 同一プログラムで I/O/ExternCall列/Effect可視イベント列が一致することを自動検証

## モジュールとファイル案（例）
- `crates/nyash-ir-core13/`（schema, verify, normalize, serde）
- `crates/nyash-ir-loop/`（schema, verify, serde）
- `crates/nyash-exec-traits/`（Value, EffectMask, Event, ExecEngine, TraceSink）
- `crates/nyash-interp-core13/`（ExecEngine実装）
- `crates/nyash-interp-loop/`（ExecEngine実装）
- `crates/nyash-rt/`（ランタイム）
- `crates/nyash-front/`（lexer/parser/AST/resolver/lowering-facade）
- `apps/nyash-cli/`（サブコマンド: run, dump-ir, ir-run, trace, bench）

## ブランチ運用
- `feature/mir-core13-interp-refactor`: 既存Core‑13実行をExecEngineでラップ、計測/トレース導入
- `experiment/loopform-interp-poc`: Loop IR定義+verify+loop→loop解釈器
- `infra/exec-switch-cli`: CLIに `--engine` と IR入出力/トレース差分
- （後段）`feature/loopform-lowering`: 変換器（Core‑13⇄Loop）— デフォルトOFF

## マイルストーン
1. ExecEngineトレイト雛形を追加（ビルド通る最小）
2. Core‑13解釈器をトレイト実装化（既存コード最小改修でアダプタ挟み）
3. Core‑13 IRのserde/verify/normalize + `ir-emit/ir-run`
4. Loop IRのschema/verify/serde + `interp-loop` 最小実装
5. A/Bトレース一致（代表: if/while/break/early return/副作用混在）
6. ベンチ3種（算術核/ループ核/副作用核）で `--engine` 差し替え比較
7. CI: 双経路の意味同値・形式同値（α同値は後段）テストを追加

## 受入れ基準（Definition of Done）
- 同一入力で `interp-core13` と `interp-loop` の I/O と外部呼び出しトレースが一致
- 代表3ベンチでイベント内訳が収集可能、差分が説明可能
- IRのシリアライズ/デシリアライズ→verify が双系統とも成功
- CLIの `--engine` / `ir-emit` / `ir-run` / `trace` が動作

## リスクと対策
- 変換せず比較するため構文上の差異は見えにくい → AST→IRで `dbg.origin/scope_id` を保持し比較用に活用
- ExecEngineの型進化 → 予約タグ/ビット・schema_versionで後方互換を担保
- デバッグ難度 → `trace-diff` と `ir-dump` に高レベル再構成表示（while/loop切替）を用意

## タスクリスト（最小）
- [ ] `nyash-exec-traits` 作成（Value/Effect/Event/ExecEngine/TraceSink）
- [ ] Core‑13実行器のトレイトラップ（最小アダプタ）
- [ ] `nyash-ir-core13` serde/verify/normalize + CLI `ir-emit/ir-run`
- [ ] `nyash-ir-loop` schema/verify/serde
- [ ] `nyash-interp-loop` 最小実装（begin/iter/branch/signal）
- [ ] CLI `--engine` 切替 + `trace` サブコマンド
- [ ] ベンチ3種 + 共通計測 + CSV出力
- [ ] CI: A/Bトレース一致テスト

## 備考
- 文法・予約語・ASTは完全共有（このフェーズでは変更しない）。
- 変換（Core‑13⇄Loop）は次フェーズ以降。まずは“同型解釈”で比較と観測に集中する。

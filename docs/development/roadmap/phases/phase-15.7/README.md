# Phase 15.7: セルフホスティング実現への道筋 - Hakoruneコンパイラ完成計画

Branch Note (selfhost)
- このブランチでは CLI バイナリ名は `hako` だよ。本文中の `hakorune`/`nyash` は `hako` に読み替えて実行してね。
- 環境変数は `HAKO_*`/`HAKU_*`/`HRN_*` は `NYASH_*` と相互エイリアス（自動マップ）なので、そのまま使ってOK。

## 🎯 **Phase 15.7の真の目的**

**「Hakorune で Hakorune をコンパイルする」完全なセルフホスティングの実現**


## 🛣️ 実行ルート（道）— 小さく進めて確実に緑にする

この順で小粒に進めると、常に quick 緑を維持しながらセルフホスティングへ到達できるよ。

1) パーサ仕上げ（prelude安全・静的化）
- ObjectParseBox を prelude 安全に（_dq/_bs ヘルパで直文字回避）
- expr/stmt 側も utils を静的呼び出しに統一（new排除）
- 受け入れ: json_native 簡易 roundtrip 緑

2) Stage‑1 JSON の早期経路固定（quiet/--min-json）
- quiet/--min-json では AST プレリュードマージをスキップ容認（エラーにしない）
- main(args) に CLI script args を渡す（-- の後を JSON 経由で注入）
- 受け入れ: selfhost_min_json_header_vm PASS（緑）

3) MIR ローアの段階実装（最小→分岐/PHI）
- Return/Const → BinOp → Compare → Branch/Jump → PHI の順に有効化
- JSON v0 Bridge の到達不能 pred 除外を統一（return/throw）
- 受け入れ: selfhost_mir_m2_*（eq_true/eq_false/compare）を順に UN‑SKIP→緑

4) Mini‑VM 箱化ラインの安定化
- InstructionScannerBox/OpHandlersBox に一本化（無限ループ対策・観測の集約）
- Arithmetic/Compare の委譲（小さな共通箱へ移譲）
- 受け入れ: m2/m3 代表（branch/jump/phi）緑

5) using/[modules] のE2E（AST OFFでも登録維持）
- modules 登録は quiet でも常時維持（AST マージはスキップ可）
- 受け入れ: using_modules_alias_vm / using_modules_rune_host_vm PASS

6) LLVM 連結ラインの最小安定化（AOT/ハーネス）
- hako_kernel に最小シンボルを追加（string.concat_{ss,si,is} / console.* / readline）
- 最小セマンティクスはフラグ NYASH_HAKO_MIN_SEM=1 でON（既定OFF）
- 受け入れ: 代表ベンチのリンク成功・終了コード一致（ハーネス or AOT）

7) スモーク整備（quick → integration）
- selfhost_min_json_header_vm / selfhost_mir_m2_* / pipeline_v2_* / json_native roundtrip を quick に集約
- 受け入れ: quick 全緑、integration 代表緑

8) 性能・回収（ホット箇所のみ）
- Mini‑VM の compare/binop ホットパス最小化（ログ削減・必要なら inlining）
- LLVM の concat/console 経路の cpool 最適化
- 受け入れ: ベンチ中央値で説明可能な改善（仕様不変）

### よく使うコマンド（抜粋）
- ビルド: `cargo build --release`
- quiet ヘッダ確認: `NYASH_USING=1 NYASH_USING_AST=0 NYASH_JSON_ONLY=1 ./target/release/hakorune --backend vm apps/selfhost-compiler/compiler.hako -- --min-json`
- LLVM ハーネス: `NYASH_LLVM_USE_HARNESS=1 NYASH_NY_LLVM_COMPILER=target/release/ny-llvmc NYASH_EMIT_EXE_NYRT=target/release ./target/release/hakorune --backend llvm apps/APP/main.hako`
- AOT最小セマンティクス: `NYASH_HAKO_MIN_SEM=1 NYASH_NYRT_SILENT_RESULT=1 ./app`
### 📊 **現状分析（2025-09-30）**

#### ✅ **既に実装済み（堅固な基盤）**
- **Rustコンパイラ**: 完全実装・安定動作（Phase 1-14）
  - Parser（完全実装✅）
  - AST生成（完全✅）
  - MIRビルダー（完全✅）
  - 3バックエンド実装✅
    - Rust VM（712行、開発・デバッグ用）
    - Python LLVM/llvmlite（1,456行、本番・最適化用）
    - PyVM（1,074行、JSON v0ブリッジ・using処理専用）
  - プラグインシステム（完全✅）

#### 🔄 **実装中（Hakoruneコンパイラ）**
- **場所**: `apps/selfhost-compiler/`
- **現状**:
  - パーサーMVP ✅（Stage-2/3サポート）
  - MIR生成基本 ✅（const/binop/compare/branch/jump/ret）
  - JSON v0出力 ✅（最小動作確認済み）
  - **Pipeline V2 🔄（Box-First emit-only architecture）**
    - 📋 **[詳細設計](../../selfhosting/pipeline_v2.md)** - 全体像・Boxes・制約
    - 📦 **[実装](../../../../apps/selfhost-compiler/pipeline_v2/)** - ExecutionPipelineBox/BackendBox/MirBuilderBox
    - 🔧 **[契約](../../../../apps/selfhost-compiler/INTERFACES.md)** - インターフェース仕様
    - 🧪 **[スモーク](../../selfhosting/pipeline_v2.md#smokes-quick)** - 受け入れテスト

#### ❌ **未完成（Phase 15.7の目標）**
1. **branch/jump最小生成** ✅（完了）
2. **LocalSSA.ensure_cond** ✅（最終パスに統合）
3. **全構文サポート** 📝（match式、property、lambda等）
4. **最適化パス** 📝（デッドコード削除、インライン化等）
5. **完全なブートストラップ** 🎯（c0→c1→c1'）

### 🤔 **VM層も一緒に作った方が楽？** → **YES！絶対YES！**

#### 💡 **理由1: 相互検証が可能**
```
Hakoruneコンパイラ（apps/selfhost-compiler/compiler.hako｜互換: .nyash）
    ↓ MIR生成
Hakorune-VM（apps/hakorune/vm/boxes/hakorune_vm_min.hako｜互換: apps/selfhost/vm/boxes/mir_vm_min.hako）
    ↓ 実行
Rust VM（src/backend/mir_interpreter/）
    ↓ 比較検証
差分があれば即座に発見！
```

#### 💡 **理由2: デバッグが容易**
- **Hakoruneコンパイラのバグ**: Mini-VMで実行 → エラー出る → MIRを見る → Rust VMと比較
- **Mini-VMのバグ**: Rustコンパイラ生成MIRで実行 → Rust VMと比較 → 差分発見（hakorune-vmへ改名・箱構造を強化）

#### 💡 **理由3: 完全な理解（教材として最高）**
```
Hakoruneでコンパイラ書く
    +
Hakoruneで実行器書く
    =
完全な理解（世界一美しい自己参照システム）
```

### 🎯 **Phase 15.7の正しい優先順位**

#### **P0: Rust VM層の安定化（既存バグの点修正・回帰防止）**
- 受け手推定・RouterPolicy・LocalSSA/材化・VarMapGuard 等の補強を優先
- quick/integration 常緑維持（既定の品質基準）
- **理由**: Rust VMは比較検証の基準点として絶対的に安定している必要がある

#### **P1: Hakorune‑VM 仕上げ（完了✅）**
- M2/M3 の代表＋エッジスモークを quick に追加
- 単一パス＋厳密セグメントで緑維持
- **成果**: `apps/hakorune/vm/boxes/hakorune_vm_min.hako` 安定動作（旧パスとの互換ルート維持）

【2025-10-01 追記】
- Mini‑VM に call/boxcall/newbox の最小意味論（i64 引数の総和）を追加。代表スモーク（exec）を quick に追加:
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_call_exec_vm.sh`
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_method_exec_vm.sh`
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_newbox_exec_vm.sh`
- Stage‑1 抽出器を負数/空白に寛容化。Emit 側は配列/文字列の両方から引数を正規化材化。
- Pipeline V2 に `LocalSSA.ensure_calls(...)` を導入（call/method/new の材化ポリシー集約）。
- v1 `mir_call` の shape スモーク（VM-only）を追加:
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_call_v1_shape_vm.sh`
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_method_v1_shape_vm.sh`
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_newbox_v1_shape_vm.sh`
- LLVM ハーネス compile-only の PHI 形状スモークを追加（STRICT=1）:
  - `tools/smokes/v2/profiles/quick/llvm/phi_if_merge_compile_ok.sh`
  - `tools/smokes/v2/profiles/quick/llvm/phi_loop_compile_ok.sh`
- 返り値→終了コードの統一（VM/WASM/AOT）: Rust VM はプログラムの戻り値をプロセス終了コードへ反映（0..255）。

【2025-10-02 追記】
- FlowEntryBox / FlowRunner（箱化・薄導線）
  - 追加: `apps/selfhost-compiler/pipeline_v2/flow_entry.hako`（emit-only 入口）
  - 追加: `apps/selfhost/vm/flow_runner.hako`（Mini‑VM 実行薄箱）
  - 役割分離: emit は selfhost-compiler 配下、実行は selfhost/vm 配下（箱境界）
- LocalSSA 材化ポリシーの統一
  - `ensure_calls`（call/method/new）、`ensure_cond`（branch cond）ともに「PHI直後に copy 挿入」に統一
  - JSONテキスト整形で挙動不変・Fail‑Fast（未対応形は無変更）
- MirCall v1（統一呼出し）
  - 薄箱: `apps/selfhost-compiler/pipeline_v2/mir_call_box.hako` を追加（emit-only）
  - ハーネス時の v1→v0 ダウングレード (`NYASH_LLVM_DOWNGRADE_V1=1`) を前提に shape/compile を安定化
  - 未解決 Global は v0 extern へ降格（compile-only）、VM/AOT は未解決エラー（Fail‑Fast）
- CLI: `--emit-mir-json` をグローバル早期ゲートに（バックエンド非依存）
  - どの backend 指定でも、パース→MIR→JSON 書き出し→即終了
  - ベンチ／WASM パイプラインの自動化に利用
- 工具: WASM 一括スクリプトを追加
  - `tools/build_and_run_wasm.sh`（.nyash → MIR(JSON) → WASM → 実行/exit code）
  - 依存: python3+llvmlite, node（wasm_runner.js）
- LLVM ハーネス（PHI）
  - Φ生成=PhiHandler、配線=finalize の不変を明記
  - 関数境界で `phi_wired`/`block_phi_incomings` をクリア（リーク防止）
  - ハーネス compile 前に IR をサニタイズ: 空PHI除去＋ブロック先頭へのPHIグループ化（検証を安定化）

#### **P2: Hakoruneコンパイラ MVP（次の主作業）**
- **既存**: `apps/selfhost-compiler/compiler.hako` を軸に実装（.nyash は後方受理）
- **目標**: Stage‑2/3 入力から JSON v0 を安定排出
- **直近TODO**:
  1. branch/jump 最小生成（完了✅）
  2. LocalSSA.ensure_cond 材化コピー（完了✅）
  3. Mini‑VM 代表追加（If/Compare 代表、Loop カウンタ 代表 追加済み✅）
  4. **🔥 UsingResolverBox実装（最優先・未着手）** - 詳細は下記「using解決の2つの側面」参照

#### **🔍 using解決の2つの側面（重要な洞察）**

**結論**: usingなどの解決は**コンパイラー側のUsingResolverBox実装**が核心。Mini-VM側は既に設計完了！

##### **A. Mini-VM側 = 既に解決済み！✅**
- MIR JSONには**解決済みのCallee::ModuleFunction**が入る
- VM側は`BoxCall`/`ExternCall`を機械的に実行するだけ
- using解決は**不要**（名前解決は事前完了）
- **現状**: M2/M3で完全動作中、追加実装不要

##### **B. コンパイラー側 = これが残課題！🔥**
```hako
// これを解析してMIR生成する機能が未実装
using "apps/lib/timer" as Timer
using "apps/lib/array_ops" as Arr

flow main() {
    local t = Timer.now_ms()  // ← シンボル解決が必要
    local a = Arr.map(...)    // ← モジュール関数解決が必要
}
```

**必要な実装（3段階）**:

1. **UsingResolverBox**（新規箱、優先度P2-A、見積もり1週間）
   ```hako
   static box UsingResolverBox {
       modules: MapBox  // "Timer" -> "/abs/path/to/timer.hako"
       symbols: MapBox  // "Timer.now_ms" -> Callee::ModuleFunction

       resolve_using(path, alias) {
           // using文を解析してmodules/symbolsに登録
       }

       get_module_path(alias) {
           // "Timer" -> "/abs/path/to/timer.hako"
       }
   }
   ```

2. **NamespaceBox**（新規箱、優先度P2-B、見積もり5日）
   ```hako
   static box NamespaceBox {
       resolve_call(namespace, method) {
           // "Timer.now_ms" -> Callee::ModuleFunction
           // UsingResolverBoxと連携
       }

       resolve_static_access(namespace, field) {
           // "Config.VERSION" -> 静的フィールドアクセス
       }
   }
   ```

3. **Pipeline V2統合**（優先度P2-C、見積もり3日）
   - CompareExtractBoxの前段階でusing解決
   - 既存のLocalSSABoxと連携
   - MIR生成時に`Callee::ModuleFunction`に変換

**関心の分離の完璧さ（設計の優秀さ）** ⭐
```
┌─────────────────┬──────────────────┐
│ コンパイラー側  │ Mini-VM側        │
├─────────────────┼──────────────────┤
│ using解決       │ 解決済みCallee   │
│ 名前空間管理    │ 機械的実行       │
│ シンボル解決    │ 型情報不要       │
│ → 複雑          │ → シンプル       │
└─────────────────┴──────────────────┘
```
**これがPhase 15.7設計の真の優秀さ！**

Quick smokes from using/new依存削減の真の意味：
- Mini-VM自体は**using不要で動く**（静的メソッド＋extern_call）
- using解決は**コンパイラー側の責務**
- **関心の分離**が完璧に実現されている！

#### **P3: Known/Rewrite 統合 Stage‑1 の仕上げ（dev観測）**
- 仕様は不変のまま、観測（resolve.try/choose / ssa.phi）と関数化の一貫性を高める
- **理由**: 開発者体験の向上（デバッグ情報の充実）

#### **P4: NYABI Kernel 下地の維持（未配線・既定OFF）**
- 将来の拡張性のための下地準備（Phase 16以降で本格化）

【2025-10-03 追記 — Core Kernel: TimerBox (P1)】
- ねらい: ベンチ/待機/パリティ検証のための「最小の時刻API」をコアBoxで提供する。
- Extern 仕様（最小）:
  - nyrt.time.now_ms → i64（単調時刻; ms）
  - LLVM: src/llvm_py/instructions/externcall.py にバインドを追加
  - VM(Rust): extern_call("nyrt","time.now_ms") を実装（std::time::Instant ベース）
  - WASM: JS の Date.now() で一時バインド（将来 Monotonic を検討）
- Box 仕様（最小）:
  - TimerBox.now_ms(): i64（上記 extern の薄ラップ）
  - modules 登録: selfhost.core.timer = "apps/core/timer/TimerBox.hako"（配置は後追い）
- 受け入れ（quick 最小スモーク）:
  - tools/smokes/v2/profiles/quick/core/timer_now_ms_vm.sh（2回の now_ms の差 ≥ 0）
  - tools/smokes/v2/profiles/quick/llvm/timer_now_ms_harness.sh（ハーネス時のみ; 無ければ SKIP）
- 注意:
  - まずは now_ms のみ（sleep_ms/async は別フェーズ）。
  - 壁時計と混在しないよう「単調時刻」を明記（壁時計は ClockBox として将来導入）。

【2025-10-03 実装完了メモ】
- 実装:
  - VM/LLVM/WASM へ `nyrt.time.now_ms` を配線。TimerBox は薄い導線（`apps/core/timer/TimerBox.hako`）。
  - フロント（Builder）で `TimerBox.now_ms()`/`new TimerBox().now_ms()` を `ExternCall("nyrt.time","now_ms")` に正規化。
- テスト:
  - quick: `core/timer_now_ms_vm.sh` で Result 行を検証。
  - quick/llvm: `llvm/timer_now_ms_harness.sh` はハーネス環境が無い場合 SKIP（Fail‑Fast設計）。
- 付記:
  - quick プロファイルでは LLVM/AOT の一部ケースを環境依存にしないため、ハーネス未検出・静的プラグイン未構成時は SKIP 方針に統一（詳細は各スモークスクリプト参照）。

【Core Kernel 候補（検討メモ）】
- ConsoleBox（既存）: log/print の最小API。現状維持。
- TimerBox（本件 P1）: now_ms のみ。sleep_ms は P2 以降（協調スケジューラ設計後）。
- RandomBox（候補）: seed(u64)/next_u64() のみ（テスト再現性目的）。導入は CI/シード方針が固まってから。
- EnvBox（候補）: get(name)/set(name,value) の最小。既定OFF; 影響範囲が広いので Box 境界で隔離。
- FSBox（候補）: read_file(path)/write_file(path,data) の最小。Runner/サンドボックス方針の下で将来。

## 📊 **進捗評価と残り作業（客観的評価・2025-10-03）**

### **現在位置: 80%完成、残り20%（見積もり3-4週間）**

```
Phase 15.7進捗: ████████░░ 80%完成

完了（80%）：
✅ Mini-VM M3基盤（6命令動作実証）
  - const, binop, compare, branch, jump, ret
✅ Pipeline V2基礎実装
  - CompareExtractBox, EmitCompareBox
  - LocalSSABox導入済み
  - MirJsonBuilderMin（JSON v0生成）
✅ FlowRunner/JsonProgramBox
  - FlowEntryBox（emit-only入口）
  - Mini-VM実行薄箱
✅ Quick smokes 172/172 PASS
  - コア常在ルール達成
  - プラグイン無効化対応
✅ TimerBox実装完了
  - nyrt.time.now_ms実装（VM/LLVM/WASM）
✅ EffectResolver一元化導線

残り20%（3-4週間）：
🔲 UsingResolverBox実装（1週間）
🔲 NamespaceBox実装（5日）
🔲 Pipeline V2統合（using）（3日）
🔲 Mini-VM命令拡張（2週間）
  - newbox（2日・最重要）
  - boxcall（2日・最重要）
  - phi（2日）
  - load/store（2日）
  - externcall（1日）
  - unaryop/typeop（2日）
🔲 セルフホストループE2E（1週間）
```

### **作業分解（週別見積もり）**

#### **Week 1: UsingResolver + Namespace実装**
- Day 1-3: UsingResolverBox実装・テスト
- Day 4-5: NamespaceBox実装・テスト
- Day 6-7: Pipeline V2統合（using解決）

#### **Week 2: Mini-VM命令拡張（最重要）**
- Day 1-2: newbox実装（Box生成）
- Day 3-4: boxcall実装（メソッド呼び出し）
- Day 5-6: phi実装（SSA合流）
- Day 7: load/store実装開始

#### **Week 3: Mini-VM命令拡張（完了）**
- Day 1-2: load/store実装完了
- Day 3: externcall実装（print等）
- Day 4-5: unaryop/typeop実装
- Day 6-7: 統合テスト・スモークテスト整備

#### **Week 4: セルフホストループE2E**
- Day 1-2: .hakoソース→MIR JSON生成（コンパイラー）
- Day 3-4: MIR JSON→実行（Mini-VM）
- Day 5-6: パリティテスト（Rust VM vs Mini-VM）
- Day 7: ブートストラップ達成！🎉

### **達成基準（明確な終了条件）**

✅ **Phase 15.7完了 = 以下すべて満たす**:
1. UsingResolverBox/NamespaceBox動作
2. Mini-VM 14命令すべて実装
3. .hakoソース→MIR JSON→Mini-VM実行成功
4. c0（Rustコンパイラ）→c1（Hakoruneコンパイラ）動作
5. c1→c1'（自己コンパイル）成功
6. Quick smokes 全PASS維持

## 🚀 **セルフホスティング実現への道筋**

### 🔄 **セルフホストループの具体的4ステップ（詳細化）**

```
┌──────────────────────────────────────┐
│ Step 1: .hako ソース解析             │
│    ↓                                  │
│ Step 2: MIR JSON生成（コンパイラ）   │
│    ↓                                  │
│ Step 3: MIR JSON実行（Mini-VM）      │
│    ↓                                  │
│ Step 4: 出力検証（パリティテスト）    │
└──────────────────────────────────────┘
```

#### **Step 1: .hakoソース解析（Rust VMで実行）**
- **入力**: `apps/selfhost-compiler/compiler.hako`
- **処理**:
  1. using文解析（UsingResolverBox）
  2. 名前空間解決（NamespaceBox）
  3. AST構築
  4. シンボルテーブル構築
- **出力**: 解決済みAST
- **検証**: `HAKO_CLI_VERBOSE=1`で解決状況確認

#### **Step 2: MIR JSON生成（コンパイラ）**
- **入力**: 解決済みAST
- **処理**:
  1. Pipeline V2実行
  2. CompareExtractBox/EmitCompareBox適用
  3. LocalSSABox適用（材化コピー）
  4. MirJsonBuilderMin実行
- **出力**: `output.json`（JSON v0形式）
- **検証**:
  ```bash
  ./target/release/hakorune --emit-mir-json output.json input.hkr
  cat output.json | jq .  # 整形表示
  ```

#### **Step 3: MIR JSON実行（Mini-VM）**
- **入力**: `output.json`
- **処理**:
  1. FlowEntryBox初期化
  2. JsonProgramBox読み込み
  3. FlowRunner実行
  4. 命令順次実行（const/binop/compare/branch/jump/ret/newbox/boxcall/phi/load/store/externcall）
- **出力**: 実行結果（標準出力/終了コード）
- **検証**:
  ```bash
  HAKO_CLI_VERBOSE=1 ./target/release/hakorune \
    apps/selfhost/vm/flow_runner.hako -- output.json
  echo $?  # 終了コード確認
  ```

#### **Step 4: 出力検証（パリティテスト）**
- **比較対象**:
  1. Rust VM実行結果
  2. Mini-VM実行結果
  3. 期待値（テストケース）
- **検証項目**:
  - 標準出力一致
  - 終了コード一致
  - 実行トレース一致（`HAKO_VM_DUMP_MIR=1`）
- **自動化**:
  ```bash
  # パリティテストスクリプト
  tools/smokes/v2/run.sh --profile quick --filter "selfhost_parity_*"
  ```

### 📅 **推奨実装順序（並行開発戦略）**

#### **Week 1-2: Hakoruneコンパイラ MVP完成（P2優先）**
- **Day 1-2**: branch/jump最小生成 ✅
- **Day 3**: LocalSSA.ensure_cond最終化 ✅
- **Day 4-7**: UsingResolverBox実装 🔥
- **Day 8-10**: NamespaceBox実装 🔥
- **Day 11-14**: Pipeline V2統合（using解決） 🔥

#### **Week 3: 統合＋検証**
- JSON v0完全出力
- Rust VM で実行確認
- スモークテスト整備
- **目標**: `tools/smokes/v2/profiles/quick/` 全緑維持

#### **Week 4: ブートストラップ達成**
- Hakoruneコンパイラ + Mini-VM統合
- セルフホスティング初期検証
- 基本プログラム実行成功
- **成果**: c0→c1→c1' 完全動作

### 📊 **実装優先度マトリックス（2025-10-03更新）**

| 項目                   | 優先度   | ステータス | 理由       | 実装時間 | 担当領域 |
|----------------------|-------|-------|----------|------|------|
| branch/jump生成        | 🔴 P2 | ✅完了 | 制御フロー必須  | 2日   | コンパイラ |
| LocalSSA.ensure_cond | 🔴 P2 | ✅完了 | 条件分岐安定化  | 1日   | コンパイラ |
| **UsingResolverBox実装** | 🔴 P2-A | 🔥未着手 | **using解決の核心** | 1週間 | コンパイラ |
| **NamespaceBox実装** | 🔴 P2-B | 🔥未着手 | 名前空間解決 | 5日 | コンパイラ |
| **Pipeline V2統合（using）** | 🔴 P2-C | 🔥未着手 | using→MIR変換 | 3日 | コンパイラ |
| **Mini-VM newbox実装** | 🟡 P1-A | 🔥未着手 | **Box生成（最重要！）** | 2日 | Mini-VM |
| **Mini-VM boxcall実装** | 🟡 P1-B | 🔥未着手 | **メソッド呼び出し** | 2日 | Mini-VM |
| Mini-VM phi実装 | 🟡 P1-C | 📝計画 | SSA合流 | 2日 | Mini-VM |
| Mini-VM load/store実装 | 🟡 P1-D | 📝計画 | メモリアクセス | 2日 | Mini-VM |
| Mini-VM externcall実装 | 🟡 P1-E | 📝計画 | print等外部呼び出し | 1日 | Mini-VM |
| match式完全対応 | 🟡 P1-F | 📝計画 | 頻繁に使用 | 2日 | コンパイラ |
| Mini-VM unaryop/typeop | 🟢 P3-A | 📝計画 | 単項演算・型操作 | 2日 | Mini-VM |
| 最適化パス | 🟢 P3-B | 📝計画 | 性能向上 | 1週間 | コンパイラ |
| エラーハンドリング | 🟢 P3-C | 📝計画 | UX向上 | 3日 | コンパイラ |

**凡例**:
- 🔴 P2: 最優先（セルフホスティング必須）
- 🟡 P1: 高優先度（基本機能実装）
- 🟢 P3: 中優先度（改善・UX向上）
- ✅完了 / 🔥未着手 / 📝計画 / 🔄実装中

**重要な優先順位**:
1. **P2-A/B/C（using解決）**: これがないとモジュールシステムが動かない
2. **P1-A/B（newbox/boxcall）**: これだけでほとんどの.hakoが動く
3. **P1-C/D/E（phi/load/store/externcall）**: 完全動作に必要

### 🎯 **具体的な実装提案**

#### **Option A: 並行開発（推奨✨）**

**トラック1（Hakoruneコンパイラ）**:
- 担当: ChatGPT + Claude
- 期間: 3週間
- 成果: 完全なHakoruneコンパイラ
- ファイル: `apps/selfhost-compiler/`

**トラック2（Mini-VM拡張）**:
- 担当: ChatGPT + Claude
- 期間: 2週間
- 成果: 完全な Hakorune-VM（M4-M7）
- ファイル: `apps/selfhost/vm/boxes/mir_vm_min.nyash`

**統合**:
- 期間: 1週間
- 成果: セルフホスティング達成

**総期間**: 4週間

#### **Option B: 順次開発**

1. コンパイラ完成（3週間）
2. Mini-VM完成（2週間）
3. 統合（1週間）

**総期間**: 6週間

### 💎 **Mini-VM実装のメリット**

#### **1. 教材として最高**
```hakorune
// Hakorune で Hakorune VM を書く
box MirVmMin {
    registers: MapBox
    blocks: MapBox

    execute(mir_json) {
        // MIR実行ロジック
        // これ自体が教材になる！
    }
}
```

#### **2. デバッグの容易さ**
- **Rust VM**: コンパイル必要、デバッグ困難
- **Mini-VM**: 即座に修正、即座に実行

#### **3. 完全な制御**
- **Rust VM**: 複雑、変更リスク大
- **Mini-VM**: シンプル、実験容易

#### **4. セルフホスティングへの道**
```
Hakoruneコンパイラ（Hakorune実装）
    ↓ MIR生成
Mini-VM（Hakorune実装）
    ↓ 実行
完全なセルフホスティング達成！🎉
```

### 🎯 **受け入れ条件（Acceptance Criteria）**

#### **Phase 15.7完了基準**

1. **quick プロファイル**: 全緑維持（96/96 PASS）
   - Mini-VM（M2/M3）代表スモーク緑
   - const/binop/compare/branch/jump/ret 動作確認
   - call/boxcall/newbox（最小意味論）実行スモーク緑
   - v1 `mir_call` 形状スモーク（VM-only）緑
   - LLVM/PHI compile-only スモーク緑（if-merge / loop）

2. **integration プロファイル**: 代表パリティ緑（llvmlite/ハーネス）
   - VM↔LLVM↔Ny のパリティ一致

### 🔧 ENV クイックリファレンス（関連）
- `NYASH_PIPELINE_V2=1` — Selfhost Pipeline V2 を有効化
- `NYASH_LLVM_USE_HARNESS=1` — LLVM llvmlite ハーネス経路
- `NYASH_LLVM_PHI_STRICT=1` — PHI: create-only（PhiHandler）/ wiring（finalize）
- `NYASH_JSON_SCHEMA_V1=1` — JSON v1（mir_call）を有効化（shape 検証用）
- `NYASH_LLVM_DOWNGRADE_V1=1` — ハーネス出力時に v1→v0 ダウングレード（compile-only 安定化）
- `NYASH_VM_USE_PY=1` — PyVM 経路（開発/比較用）


3. **Builder観測**: resolve.try/choose と ssa.phi が dev‑only で取得可能
   - 環境変数: `HAKO_DEBUG_*`

4. **表示API統一**: QuickRef/ガイドが `str()` に統一
   - 実行挙動は従前と同じ（互換性維持）

5. **Selfhost Compiler（dev限定）**:
   ```bash
   HAKO_JSON_ONLY=1 ./target/release/hakorune \
     apps/selfhost-compiler/compiler.hako -- --stage3   # 互換: compiler.nyash も受理
   ```
   → JSON ヘッダ（`{"version":…, "kind":…}`）を出力（非空）

6. **ブートストラップ成功**:
   ```bash
   # c0（Rustコンパイラ）→ c1（Hakoruneコンパイラ）
   ./target/release/hakorune apps/selfhost-compiler/compiler.hako \
     -- input.hkr > output.json

   # c1 → c1'（自己コンパイル）
   ./target/release/hakorune apps/selfhost/vm/boxes/mir_vm_min.nyash \
     -- output.json
   ```

補足（Branding/Flags の整理）
- 設定ファイルは `hako.toml` を優先（互換: `nyash.toml`/`hakorune.toml`）。
- プラグイン仕様は `hako_box.toml` を優先（互換: `nyash_box.toml`）。
- 環境変数の公式前置詞は `HAKO_*`（互換: `HAKU_*`/`HRN_*`/`NYASH_*`）。

## 📋 **実装タスクリスト（小粒・段階的）**

### **Phase 1: Hakoruneコンパイラ基本強化（Week 1-2）**

1. **branch/jump最小生成実装**（2日）
   - ファイル: `apps/selfhost-compiler/boxes/mir_emitter_box.nyash`
   - 目標: if/loop の制御フローを JSON v0 で正しく出力
   - 検証: Rust VM で実行して期待値一致

2. **LocalSSA.ensure_cond最終化**（1日）
   - ファイル: `apps/selfhost-compiler/builder/ssa/local.nyash`
   - 目標: 条件分岐前の材化コピー完全動作
   - 検証: compare/branch の組み合わせテスト

3. **基本構文完全対応**（4日）
   - if/else（完了✅）
   - loop（実装中🔄）
   - call/method（実装中🔄）
   - new/me（計画📝）

### **Phase 2: Mini-VM拡張（Week 2-3、並行可能）**

4. **M4: ループサポート**（3日）
   - ファイル: `apps/selfhost/vm/boxes/mir_vm_min.nyash`
   - 目標: loop命令の実行
   - 検証: 累積計算テスト

5. **M5: Box操作サポート**（2日）
   - new/field access/method call
   - 検証: StringBox/ArrayBox基本操作

6. **M6-M7: プラグインBox対応**（2日）
   - FileBox/PathBox統合
   - 検証: パリティテスト全PASS

### **Phase 3: 統合＋ブートストラップ（Week 4）**

7. **統合テスト**（3日）
   - Hakoruneコンパイラ + Mini-VM連携
   - JSON v0 完全出力確認
   - スモークテスト整備

8. **ブートストラップ達成**（4日）
   - c0→c1コンパイル成功
   - c1→c1'自己コンパイル成功
   - パリティテスト合格

## 🔧 **開発環境・ツール**

### **スモークテスト実行**
```bash
# quick プロファイル（全体）
tools/smokes/v2/run.sh --profile quick

# セルフホスティング関連のみ
tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"

# Mini-VM テスト
tools/smokes/v2/run.sh --profile quick --filter "selfhost_mir_m*"
```

### **手動テスト**
```bash
# Hakoruneコンパイラ実行
./target/release/hakorune apps/selfhost-compiler/compiler.hako \
  -- --stage3 sample.hkr > output.json

# Mini-VM実行
./target/release/hakorune apps/selfhost/vm/boxes/mir_vm_min.nyash \
  -- output.json

# Rust VM比較
./target/release/hakorune --backend vm sample.hkr
```

### **デバッグ用環境変数**
```bash
# 詳細診断
HAKO_CLI_VERBOSE=1

# MIR出力
HAKO_VM_DUMP_MIR=1
./target/release/hakorune --dump-mir program.hkr

# JSON IR出力
./target/release/hakorune --emit-mir-json debug.json program.hkr

# セルフホスト専用
HAKO_JSON_ONLY=1      # JSON のみ出力
HAKO_COMPILER_TRACK=1 # コンパイラトラック有効化
```

## 🎊 **Phase 15.7完了の意義**

### **技術的成果**
1. ✅ **完全なセルフホスティング達成**
   - Hakoruneで Hakorune をコンパイル
   - 外部コンパイラ依存からの完全解放

2. ✅ **教材として最高の価値**
   - コンパイラ実装: `apps/selfhost-compiler/` 全体
   - VM実装: `apps/selfhost/vm/boxes/mir_vm_min.nyash`
   - 完全な理解が可能な規模

3. ✅ **保守性の革命**
   - Hakorune でコンパイラを書く → 誰でも改造可能
   - MIR 13命令 → 究極のシンプルさ
   - Everything is Box哲学の完成

### **次のマイルストーン（Phase 16）**
- 最適化パス追加（デッドコード削除、インライン化）
- エラーメッセージ改善
- LLVM バックエンド完全統合
- ネイティブバイナリ生成（EXE化）

## 📚 **関連ドキュメント**

### **Phase 15シリーズ**
- [Phase 15: セルフホスティング全体計画](../phase-15/README.md)
- [Phase 15.5: Core Box統一](../phase-15.5/README.md)
- [Phase 15.6: MIR Call革新](../phase-15.6/README.md)

### **実装ガイド**
- [セルフホスティング戦略 2025年9月版](../phase-15/implementation/self-hosting-strategy-2025-09.md)
- [LLVM EXE生成戦略](../phase-15/implementation/llvm-exe-strategy.md)

### **言語リファレンス**
- [Quick Reference](../../../../reference/language/quick-reference.md)
- [完全言語仕様](../../../../reference/language/LANGUAGE_REFERENCE_2025.md)
- [MIR命令セット](../../../../reference/mir/INSTRUCTION_SET.md)

## 🌟 **結論**

**「VM層も一緒に作った方が楽」 = 絶対YES！✨**

理由:
- ✅ 相互検証で品質向上
- ✅ デバッグ容易
- ✅ 完全な理解
- ✅ 教材として最高
- ✅ セルフホスティング直結

**推奨**: コンパイラとMini-VMを並行開発！4週間でセルフホスティング達成！

---

背景（技術詳細）
- Instance→Function 正規化の方針は既定ON。Known 経路は関数化し、VM側は単純化する。
- resolve.try/choose（Builder）と ssa.phi（Builder）の観測は dev‑only で導入済み（既定OFF）。
- Mini‑VM は M2/M3 の代表ケースを安定化（パス/境界厳密化）。
- VM Kernel の Ny 化は後段（観測・ポリシーから段階導入、既定OFF）。

優先順（2025‑09‑29 リバランス / 2025‑10‑04 反映）
- P0: Rust VM 層の安定化（既存バグの点修正・回帰防止）
  - 受け手推定・RouterPolicy・LocalSSA/材化・VarMapGuard 等の補強を優先（quick/integration 常緑）。
- P1: Mini‑VM 仕上げ（完了）
  - M2/M3 の代表＋エッジスモークを quick に追加し、単一パス＋厳密セグメントで緑維持。
- P2: Nyash コンパイラ MVP（Phase 15.6）の前進（次の主作業）
- 既存 `apps/selfhost-compiler/compiler.hako` を軸に、Stage‑2/3 入力から JSON v0 を安定排出（.nyash は後方受理）。
  - 受け入れ（dev限定）: `NYASH_JSON_ONLY=1` で `version/kind` を含む JSON ヘッダが非空であること。
  - 既定挙動は不変。コンパイラは別アプリ（apps/）として進め、VM/LLVM 本線は影響最小。
  - 直近 TODO: branch/jump 最小生成＋LocalSSA.ensure_cond の材化コピー最終化、Mini‑VM 代表追加1件。
- P3: Known/Rewrite 統合 Stage‑1 の仕上げ（dev観測）
  - 仕様は不変のまま、観測（resolve.try/choose / ssa.phi）と関数化の一貫性を高める。
- P4: NYABI Kernel 下地の維持（未配線・既定OFF）

Compiler Track（大規模変更の部分解禁 — apps/selfhost-compiler/ 限定）
- 目的: Selfhost Compiler を段階的に実用化。Core（src/）は引き続き安定運用。
- ガード:
  - 既定OFFのフラグ/引数（例: `NYASH_COMPILER_TRACK=1`, `--min-json`, `--emit-mir`）。
  - quick/integration 常緑を維持。影響は Selfhost 実行時に限定。
- 受け入れ（dev）:
  - `NYASH_JSON_ONLY=1 ... --min-json` で JSON ヘッダ非空。
  - `--emit-mir` で最小 MIR(JSON v0)（const→ret）を生成可能。

Unified Call（開発既定ON）
- 呼び出しの統一判定は、環境変数 `NYASH_MIR_UNIFIED_CALL` が `0|false|off` でない限り有効（既定ON）。
- メソッド解決/関数化を `emit_unified_call` に集約し、以下の順序で決定:
  1) 早期 toString/stringify→str
  2) equals/1（Known 優先→一意候補; ユーザーBox限定）
  3) Known→関数化（`obj.m → Class.m(me,…)`）／一意候補フォールバック（決定性確保）
- レガシー側の関数化は dev ガードで抑止可能: `NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1`（移行期間の重複回避）

スコープ（やること）
1) Builder: Known 化 + Rewrite 統合（Stage‑1）
   - P0: me 注入・Known 化（origin 付与/維持）— 軽量PHI補強（単一/一致時）
   - P1: Known 経路 100% 関数化（obj.m → Class.m(me,…)）。special は `toString→str（互換:stringify）/equals` を統合
   - 観測: resolve.try/choose / ssa.phi を dev‑only で JSONL 出力（既定OFF）。`resolve.choose` に `certainty` を付加し、KPI（Known率）を任意出力（`NYASH_DEBUG_KPI_KNOWN=1`, `NYASH_DEBUG_SAMPLE_EVERY=N`）。

2) 表示APIの統一（挙動不変）
   - 規範: `str()` / `x.str()`（同義）。`toString()` は早期に `str()` へ正規化
   - 互換: `stringify()` は当面エイリアスとして許容
   - QuickRef/ガイドの更新（plus混在の誘導も `str()` に統一）

3) Mini‑VM（MirVmMin）安定化（devのみ）
   - 厳密セグメントによる単一パス化、M2/M3 代表スモーク常緑（const/binop/compare/branch/jump/ret）
   - パリティ: VM↔LLVM↔Ny のミニ・パリティ 2〜3件

4) NYABI（VM Kernel Bridge）下地（未配線・既定OFF）
   - docs/abi/vm-kernel.md（関数: caps()/policy.*()/resolve_method_batch()）
   - スケルトン: apps/selfhost/vm/boxes/vm_kernel_box.nyash（policy スタブ）
 - 既定OFFトグル予約: NYASH_VM_NY_KERNEL, *_TIMEOUT_MS, *_TRACE

非スコープ（やらない）
- 既定挙動の変更（Rust VM/LLVMが主軸のまま）
- PHI/SSAの一般化（Phase 16 で扱う）
- VM Kernel の本配線（観測・ポリシーは dev‑only/未配線）

リスクと軽減策
- 性能: 境界越えは後Phaseに限る（本Phaseは未配線）。Mini‑VMは開発補助で性能要件なし。
- 複雑性: 設計は最小APIに限定。拡張は追加のみ（後方互換維持）。
- 安全: すべて既定OFF。Fail‑Fast方針。再入禁止/タイムアウトを仕様に明記。

受け入れ条件（Acceptance）
- quick: Mini‑VM（M2/M3）代表スモーク緑（const/binop/compare/branch/jump/ret）
- integration: 代表パリティ緑（llvmlite/ハーネス）
- Builder: resolve.try/choose と ssa.phi が dev‑only で取得可能（NYASH_DEBUG_*）
- 表示API: QuickRef/ガイドが `str()` に統一（実行挙動は従前と同じ）
- Unified Call は開発既定ONだが、`NYASH_MIR_UNIFIED_CALL=0|false|off` で即時オプトアウト可能（段階移行）。
- Selfhost Compiler（dev限定・任意ゲート）:
- `NYASH_JSON_ONLY=1 ./target/release/nyash apps/selfhost-compiler/compiler.hako -- --stage3` が JSON ヘッダ（`{"version":…, "kind":…}`）を出力（非空）。

実装タスク（小粒）
1. origin/observe/rewrite の分割方針を CURRENT_TASK に反映（ガイド/README付き）
2. Known fast‑path の一本化（rewrite::try_known_rewrite）＋ special の集約
3. 表示APIの統一（toString→str、互換:stringify）— VM ルータ特例の整合・ドキュメント更新
4. MirVmMin: 単一パス化・境界厳密化（M2/M3）・代表スモーク緑
5. docs/abi/vm-kernel.md（下書き維持）・スケルトン Box（未配線）

トグル/ENV（予約、既定OFF）
- NYASH_VM_NY_KERNEL=0|1
- NYASH_VM_NY_KERNEL_TIMEOUT_MS=200
- NYASH_VM_NY_KERNEL_TRACE=0|1

ロールバック方針
- Mini‑VMの変更は apps/selfhost/ 配下に限定（本線コードは未配線）。
- NYABIは docs/ と スケルトンBoxのみ（実行経路から未参照）。
- Unified Call は env で即時OFF可能。問題時は `NYASH_MIR_UNIFIED_CALL=0` を宣言してレガシーへ退避し、修正後に既定へ復帰。

補足（レイヤー・ガード）
- builder 層は origin→observe→rewrite の一方向依存を維持する。違反検出スクリプト: `tools/dev/check_builder_layers.sh`

関連（参照）
- Phase 15（セルフホスティング）: ../phase-15/README.md
- Phase 15.5（基盤整理）: ../phase-15.5/README.md
- Known/Rewrite 観測: src/mir/builder/{method_call_handlers.rs,builder_calls.rs}, src/debug/hub.rs
- QuickRef（表示API）: docs/reference/language/quick-reference.md
- Mini‑VM: apps/selfhost/vm/boxes/mir_vm_min.nyash
- スモーク: tools/smokes/v2/profiles/quick/core/

更新履歴
- 2025‑09‑28 v2（本書）: Known 化＋Rewrite 統合（dev観測）、表示API `str()` 統一、Mini‑VM 安定化へ焦点を再定義
- 2025‑09‑28 初版: Mini‑VM M3 + NYABI下地の計画

## ステータス（2025‑09‑28 仕上げメモ）
- M3（compare/branch/jump）: Mini‑VM（MirVmMin）が厳密セグメントの単一パスで動作。代表 JSON 断片で compare(Eq)→ret、branch、jump を評価。
- 統合スモーク: integration プロファイル（LLVM/llvmlite）は PASS 17/17（全緑）。
- ルータ／順序ガード（仕様不変）:
  - Router: 受信者クラスが Unknown のメソッド呼び出しは常にレガシー BoxCall にフォールバック（安定性優先・常時ON）。
  - Router（補足）: `InstanceBox × {length,len,substring,indexOf,lastIndexOf}` は Unified に固定し、`StringBox` 正規化へ導く（VM救済に依存しない）。
  - BlockSchedule: φ→Copy(materialize)→本体(Call) の順序を dev‑only で検証（`NYASH_BLOCK_SCHEDULE_VERIFY=1`）。
  - LocalSSA: 受信者・引数・条件・フィールド基底を emit 直前で「現在のブロック内」に必ず定義。
- VM 寛容フラグの方針:
  - `NYASH_VM_TOLERATE_VOID`: dev 時の救済専用（quick テストからは除去）。
  - Router の Unknown→BoxCall は常時ON（仕様不変・安定化目的）。

## 次のTODO（短期）
- Rust VM 安定化（点補修の仕上げ）
  - 既知箇所の観測を最小ONで確認（必要時のみ）。
- json_query_vm（VM）: LocalSSA/順序の取りこぼし補強（救済OFFで緑）。
- ループ PHI 搬送: header/合流の VarMapGuard 観測（break/continue を安定）。
- Mini‑VM M2/M3: 追加エッジ（複数compare/ret先頭/ゼロ除算/no‑retフォールバック）を quick で常緑（完了済）。
- Selfhost Compiler（dev）: JSONヘッダ非空スモーク（任意ゲート）を準備。

## Builder 小箱（Box 化）方針（仕様不変・段階導入）
- S-tier（導入）:
  - MetadataPropagationBox（型/起源伝播）: `metadata/propagate.rs`
  - ConstantEmissionBox（Const発行）: `emission/constant.rs`
  - TypeAnnotationBox（最小型注釈）: `types/annotation.rs`
  - RouterPolicyBox（Unified vs BoxCall ルート）: `router/policy.rs`
  - EmitGuardBox（emit直前の最終関所）: `emit_guard/mod.rs`
  - NameConstBox（関数名Const生成）: `name_const.rs`
- A/B-tier（計画）:
  - Compare/BranchEmissionBox、PhiWiringBox、EffectMask/TypeInferenceBox（Phase16以降）

採用順（小さく安全に）
1) Const → metadata → 最小注釈の順に薄く差し替え（代表箇所→全体）
2) RouterPolicyBox を統一Call経路に導入（utils側は後段で移行）
3) EmitGuardBox で Call 周辺の finalize/verify を集約（Branch/Compare は後段）
4) NameConstBox を rewrite/special/known に段階適用

ドキュメント
- 詳細は `docs/development/builder/BOXES.md` を参照。

## Unskip Plan（段階復帰）
- P0: json_query_vm → 期待出力一致、寛容フラグ不要。
- P1: loops（break/continue/loop_statement）→ PHI 搬送安定。
- P2: Mini‑VM（M2/M3）→ 代表4件 PASS、coarse 撤去・単一パス維持。


【2025-10-05 更新】Hakorune‑VM と箱の適用／WASM toolchain ゲート

- Hakorune‑VM の配置と alias（段階移行）
  - 追加: `apps/hakorune/vm/boxes/hakorune_vm_min.hako`
  - `hako.toml` に `[modules] hakorune.vm.mir_min` を追加（旧 `selfhost.vm.mir_min` は1リリース alias 維持）
  - FlowRunner/dev サンプルの using を hakorune alias へ寄せ（印字は `run`、実行は `run_min`）

- 箱（Box‑First）の導入と責務分解（薄い境界）
  - InstrDecoderBox: JSON v0 のオブジェクト走査（`InstructionScannerBox.next` 委譲）
  - ProgramStateBox: `regs/bb/prev_bb/steps` の状態管理（write 全面／read は段階導入中）
  - CfgNavigatorBox: ブロック head/tail と `index_of_from` を集約し、VM/Scanner から委譲
  - RetResolverBox: `ret` 値決定を一本化（last_cmp 優先→regs→Fail‑Fast）
  - PhiWiringBox: Φ 適用の薄いラッパ（既存 apply に委譲）
  - DiagnosticsBox: 既定静音、`{"__dev__":1}` マーカーで `debug()` を有効化（将来 CLI→Box 橋渡し）

- 実装ポイント（HakoruneVmMin）
  - `run_min` を標準入口に統一（`run` は印字付きアダプタ）
  - `ProgramStateBox` を regs/steps/bb/prev に導入（write 全面、read を段階拡大）
  - `CfgNavigatorBox` の head/tail、`index_of_from` へ呼び替え（重複排除）
  - `RetResolverBox.resolve` へ ret 判定を委譲（診断は Diagnostics に一元化）

- スモーク（代表最小・増やしすぎ回避）
  - 追加: `hakorune_vm_m2_eq_true_vm.sh`
  - 追加: `hakorune_vm_m3_branch_true_vm.sh` / `..._branch_false_vm.sh` / `..._jump_vm.sh` / `..._jump_chain_vm.sh` / `..._phi_diamond_vm.sh`
  - Builder auto‑birth のクロスモジュール確認: `builder_autobirth_cross_module_{vm,alias}_vm.sh`（常時ONに寄せ）

- using/[modules] の扱い
  - `hakorune.vm.mir_min` への alias 寄せを段階実施（selfhost.* は1リリース alias 維持）
  - E2E は代表1本に限定して確認（増やしすぎ回避）

- WASM: llvmlite 制限の回避（オプトイン・フォールバック有）
  - `NYASH_LLVM_WASM_TOOLCHAIN=1` で `llc + wasm-ld` 経路を使用（既定OFF）
  - 追加エクスポートは `NYASH_WASM_EXPORTS="Main.main,main"`（既定で `ny_main`）
  - 失敗時は llvmlite `emit_object` にフォールバック（既定挙動は不変）
  - ドキュメント追記: `docs/guides/execution-modes-guide.md` にツールチェイン手順を追加

- 次の小粒 TODO（48h）
  - ProgramStateBox の read を残差2箇所で get 化（bb/prev_bb 一貫性）
  - CfgNavigatorBox への `index_of_from` 呼びを段階置換（VM 側の重複を削減）
  - CLI→DiagnosticsBox の DEV 橋渡し（起動時に dev マーカー注入）
  - hakorune_* の代表を1本だけ追加（compare→branch→phi entry の複合）


【2025-10-05 追記 2】DEV ブリッジと箱化の仕上げ（FlowRunner/CLI／スキャン統一／プラグイン検証）

- DEV ブリッジ（CLI 主導）
  - CLI（Rust）で `NYASH_DEV_JSON_MARKER=1` の時、MIR JSON 出力に `{"__dev__":1}` を付与（`src/runner/mir_json_emit.rs`）。
  - FlowRunner は注入を既定OFFにし、将来用の薄い導線 `_maybe_inject_dev_marker(j, ast_json)` を追加。
    - AST JSON 側に `{"__cli_dev__":1}` が含まれる時のみ `{"__dev__":1}` に正規化（既定は未使用）。
  - HakoruneVmMin は `{"__dev__":1}` を検出して DiagnosticsBox.debug を有効化（既定静音）。

- ProgramStateBox の read を debug/trace でも統一
  - `bb/prev_bb` は ProgramStateBox から取得（write/read の一貫化）。
  - 情報系ログは DiagnosticsBox.debug（DEVのみ）に寄せ、[ERROR]のみ従来出力。

- CfgNavigatorBox 委譲の仕上げ（ホットパス外から）
  - InstructionScannerBox に加え、MiniVmScan（selfhost/common）側でも `index_of_from` を委譲。
  - JSON スキャン系の重複ロジックを段階的に削減（性能影響は最小）。

- プラグイン birth 仕様の固定（E2E）
  - 既存: `plugin_birth_e2e_vm.sh`（明示 birth の冪等）、`plugin_no_birth_nop_vm.sh`（no‑birth は no‑op）。
  - 追加: `plugin_autobirth_e2e_vm.sh`（new→method の auto‑birth）。
  - 追加（opt‑in）: `userbox_birth_idempotent_vm.sh`（二回目 birth が no‑op、パーサ制約のためゲート付与）。

—

この先（セルフホスティングに向けた大まかな計画）

1) コンパイラ最小ルートの完成（Ny→JSON v0→MIR(JSON v0)）
   - Stage‑1: AST→JSON v0 の出力見直し（UsingResolverBox の導入、prelude 安全）。
   - Stage‑2: MIR(JSON v0) の最小命令（const/binop/compare/branch/jump/ret/phi）を安定排出。
   - 代表ケース（const→ret／compare→ret／compare→branch→phi）で Hakorune‑VM 実行＝緑。

2) VM/箱の仕上げ（薄い境界を維持）
   - ProgramStateBox: read 残差の完全移行（log/trace/観測点まで get 化）。
   - CfgNavigatorBox: 依存箇所の委譲をもう一段だけ拡大（重複ロジックの排除）。
   - RetResolverBox/DiagnosticsBox: 失敗系と観測の一元化（Fail‑Fast＋DEV静音）。

3) DEV ブリッジの一本化
   - 既定は CLI→MIR JSON emit で `{"__dev__":1}` を付与。
   - FlowRunner の `_maybe_inject_dev_marker` は将来の AST 側からの橋渡し用（既定OFFのまま）。

4) スモーク/テスト（増やしすぎず代表性重視）
   - hakorune_*: m2/m3 の代表を1本ずつ（既存 eq_true / branch / jump / phi + 複合1本）。
   - プラグイン系: auto/明示/no‑birth の3本で固定。userbox 冪等は opt‑in で補助。

5) ドキュメントとエコシステム
   - Phase‑15.7 の更新点を継続追記（箱の責務・導線・ゲート説明）。
   - CI は quick 緑維持、重い/環境依存は opt‑in（ゲート付き運用）。


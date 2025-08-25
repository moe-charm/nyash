# 🎯 CURRENT TASK - 2025年8月26日（状況整理＆再起動ショートカット）

## ⏱️ 再開ショートカット（今日のフォーカス）
- フォーカス: VM比較経路の安定化後片付け + 1000行分解の下準備
- 目標: 一時ログ抑制・Phi正規化・基本ボックス統一（String/Bool）・VM分割の導線作成
- 参照: `docs/development/roadmap/phases/phase-9/phase_9_78h_mir_pipeline_stabilization.md`

### 直近の実行タスク（9.78h）
1) 一時デバッグログの抑制（`NYASH_VM_DEBUG_*`のみ）
   - 進捗: Runnerのバナー/プラグイン初期化ログは `NYASH_CLI_VERBOSE`/`NYASH_DEBUG_PLUGIN` のみで出力。
           VMの逐次ログは `NYASH_VM_DEBUG[_EXEC|_CMP|_ANDOR|_PHI]` に限定。
2) Phi正規化（LoopExecutorの借用衝突解消 → 正しい選択へ復帰）
   - 進捗: VM側の選択を `previous_block` 基準に復帰（fallback: 先頭）。`NYASH_VM_DEBUG_PHI=1` でログ。
   - 設計: docs/development/current/PHI_NORMALIZATION_PLAN.md を参照（段階プラン/次アクション）。
3) 基本ボックス統一（StringBox/BoolBoxもre-export化）
4) VM分割の導線（control_flow/dispatch/frameへ分離設計）
   - 進捗: `src/backend/{control_flow.rs,dispatch.rs,frame.rs}` を追加（骨組み）。ビルド通過。
5) 代表スナップショット追加（compare/loop/typeop_mixed）

### すぐ試せるコマンド
```bash
cargo build --release -j32
nyash --dump-mir --mir-verbose local_tests/typeop_is_as_func_poc.nyash | sed -n '1,160p'
NYASH_OPT_DIAG_FAIL=1 nyash --dump-mir --mir-verbose local_tests/typeop_diag_fail.nyash || echo DIAG BLOCKED
tools/ci_check_golden.sh  # 代表ケースのMIR含有チェック

# 比較・論理のスモーク（VM）
nyash --backend vm local_tests/compare_box_vm.nyash    # 期待: true
nyash --backend vm local_tests/and_or_vm.nyash         # 期待: false\ntrue
nyash --backend vm local_tests/and_or_truthy_vm.nyash  # 期待: false,true,false,true,false
```

### 重要リンク（唯一参照/ゲート）
- 命令セット（唯一出典・26命令）: `docs/reference/mir/INSTRUCTION_SET.md`
- 9.78h（本フェーズ詳細）: `docs/development/roadmap/phases/phase-9/phase_9_78h_mir_pipeline_stabilization.md`
- 9.79（P2P本体 前提: 9.78h完了）: `docs/development/roadmap/phases/phase-9/phase_9_79_p2pbox_rebuild.md`
- Phase 10（Cranelift JIT主経路）: `docs/development/roadmap/phases/phase-10/phase_10_cranelift_jit_backend.md`

### 🧩 今日の再開ポイント（2025-08-26）
- 完了（適度な分解）: runnerを`modes/`へ分離（mir/vm/llvm/bench/wasm/aot/common）、objectsを`objects/{ops,methods,fields}.rs`へ、VMに`frame.rs`/`control_flow::record_transition`/`dispatch::execute_instruction`導入。ログ抑制（NYASH_VM_DEBUG_* / NYASH_CLI_VERBOSE / NYASH_DEBUG_PLUGIN）、Phi正規化Step1（previous_block基準）。
- 次アクション（小さく前進）:
  1) MIR26命令「総数一致」チェック（コード≡ドキュメント）
  2) Loop SSA Step2（seal/pred更新のスケルトン＋Verifierのphi系文言強化）
  3) 代表スナップショット再確認（TypeOp/extern_call/loop/await/boxcall）
- クイックコマンド: `cargo build --release -j32` → `nyash --backend vm local_tests/compare_box_vm.nyash` → `./tools/ci_check_golden.sh`


## 🚨 現在の状況（2025-08-25）

### ✅ 完了したタスク

1. **✅ MIRビルダーリファクタリング完了🎉**（2025-08-25）
   - mir/builder.rs: 1547行の大規模モジュール → **モジュール分割完了**
   - 新構造: `src/mir/builder/` ディレクトリ
     - `mod.rs`: 公開API定義
     - `core.rs`: MirBuilder本体 + コア機能 (205行)
     - `expressions.rs`: 式変換処理 (621行) - 18関数実装済み
     - `statements.rs`: 文変換処理 (165行) - 6関数実装済み 
     - `control_flow.rs`: 制御フロー構築 (194行) - 4関数実装済み
     - `box_handlers.rs`: Box関連処理 (73行) - 2関数実装済み
   - **ビルド確認**: 新構造でコンパイル正常完了 ✅
   - **総移動関数数**: 30関数（ヘルパー関数含む）
   - **行数削減**: 元の1547行から分割により読みやすさ大幅向上
   - **コミット完了**: 
     - Phase 3-5: cc2a5c2 (expressions.rs)
     - Phase 6-8: 29af5e1 (statements/control_flow/box_handlers)

2. **✅ VMモジュールのリファクタリング完了**（2025-08-25）
   - execute_instruction関数を29個のハンドラーに分割
   - vm_instructions.rsモジュール作成（487行）
   - execute_instruction_old削除（691行削減）
   - vm.rs: 2075行→1382行（33%削減）
   
### 🆕 進捗（2025-08-25 夜）
- VM and/or 対応の実装を追加（短絡評価は `as_bool()` 強制で統一、`BoxRef(IntegerBox)` の四則演算にも対応）。
- ゴールデン（TypeOp代表2ケース）は緑のまま、Optimizer診断（未lowering検知）も動作確認済み。
- ただし、VM実行時に `And` が `execute_binop` の短絡ルートに入らず、`execute_binary_op` の汎用経路で `Type error: Unsupported binary operation: And ...` が発生。
  - 兆候: `NYASH_VM_DEBUG_ANDOR` のデバッグ出力が出ない＝`execute_binop` に入っていない、または別実装ルートを通過している可能性。
  - 仮説: 古いVM経路の残存/リンク切替、もしくは実行時に別ビルド/別profileが使用されている。

### 🆕 進捗（2025-08-26 早朝）
- and/or 真理値強制の最終化: `VMValue::as_bool()` を拡張（`BoxRef(IntegerBox)`→非0でtrue、`Void`→false）。
- テスト追加: `local_tests/and_or_truthy_vm.nyash`（期待: `false,true,false,true,false`）。緑を確認。
- 基本ボックス統一（第一弾）: `IntegerBox` を正典に一本化。
  - `src/boxes/integer_box.rs` は `pub use crate::box_trait::IntegerBox;` に置換し、実体の二重化を排除。
- 既知の未解決: ループ比較で `BoxRef(IntegerBox) < BoxRef(IntegerBox)` が TypeError。
  - 対応中: 比較前に i64 へ正規化するフォールバックをVMに実装（downcast→toString→parse）。
  - 80/20ポリシー: 数値にパース可能なら比較継続、失敗時のみTypeError。

### 🆕 進捗（2025-08-26 午前）
- TypeError（`And/Or` 経路）再発なしを確認（3スモーク緑: compare/and_or/and_or_truthy）。
- ログ抑制の徹底: Runner/VMのデバッグ出力を既定で静音、環境変数でのみ有効化。
- Phi正規化 Step1: `previous_block` によるPhi入力選択をVMに実装（`NYASH_VM_DEBUG_PHI=1`）。
- VM分割の骨組み: `control_flow.rs`/`dispatch.rs`/`frame.rs` 追加（今後段階移動）。
- レガシー削除: `src/mir/builder_old.rs`, `src/mir/builder.rs.backup`, `src/parser.rs.backup`, `src/instance.rs.backup`, `src/box_trait.rs.backup` を削除。
- objects.rs 分解 Step1: `execute_new` をヘルパ（三分割）へ抽出しスリム化（等価挙動）。
   
### 🎯 次の優先タスク（更新）

1. **copilot_issues.txtの確認** 
   - Phase 8.4: AST→MIR Lowering完全実装（最優先）
   - Phase 8.5: MIRダイエット（35命令→20命令）
   - Phase 8.6: VM性能改善（0.9倍 → 2倍以上）

1.5 **VM実行経路の特定と修正（最優先）**
   - `runner.rs → VM::execute_module → execute_instruction → execute_binop` の各段でデバッグログを追加し、`BinOp(And/Or)` がどこで分岐しているか特定。
   - バイナリ一致確認: `strings` によるシグネチャ（デバッグ文字列）含有の照合で実行バイナリを同定。
   - 代替経路の洗い出し: `src/` 全体で `execute_binop`/`And`/`Unsupported binary operation` を再走査し、影響箇所を一掃。
   - 修正後、`local_tests/and_or_vm.nyash` で `false/true` の出力を確認。
   - ルート確定: Compare経路はBuilder側のCast導線で安定。VM側は保険フォールバックを維持しつつ一時ログを抑制へ。
1.6 **objects.rs 分解 Step2（安全にファイル分割）**
   - `objects_impl.rs` を導入し、抽出済みヘルパを移動。本体は薄いラッパに。
   - 以降: `objects/{fields.rs,methods.rs,ops.rs}` への段階分解。

1.7 **runner.rs 分離**
   - `init_bid_plugins` を `runner/plugin_init.rs` へ抽出。各モードを `runner/modes/*.rs` に。

1.8 **VM分割の段階移動**
   - ブロック遷移を `control_flow.rs`、フレーム状態を `frame.rs` に移し、`dispatch.rs` の導線を準備。

2. **MIR26命令対応**
   - TypeOp/WeakRef/Barrierのプリンタ拡張
   - スナップショット整備（extern_call/loop/boxcall/typeop_mixed 追加済）
   - vm-stats差分確認

3. **Builder適用拡大**
   - 言語 `is/as` 導線の実装
   - 弱参照フィールドのWeakLoad/WeakNew対応
   - 関数スタイル `isType/asType` の早期lowering強化

## 🗺️ 計画の粒度（近々/中期/長期）

### 近々（1–2週間）
- Loop SSA復帰: `loop_api` 小APIを活用し、Phi挿入・seal・predecessor更新を段階適用。簡易loweringを正しいSSAに置換。
- Builder移行完了: `builder.rs`の機能を`builder_modularized/*`へ移し切り、両者の差分（命令フィールド名・効果）を完全一致。
- TypeOp網羅: `is/as`/`isType/asType`の早期loweringを再点検、Optimizer診断（未lowering検出）を有効化し回帰を防止。
- 軽量スナップショット: MIRダンプ（verbose）に対する含有チェックを代表ケース（TypeOp/extern_call/loop/await/boxcall）へ拡張。

### 中期（3–4週間）
- WeakRef/Barrier統合: lowering（WeakNew/WeakLoad/Barrier）導線を整理、統合命令フラグON/OFFでMIR差を固定化。Verifierに整合チェック追加。
- MIR26整合化: Printer/Verifier/Optimizerの26命令前提をそろえ、効果の表記・検証を強化。
- CLI分離テスト: ワークスペース分割 or lib/binaryテスト分離で、`cargo test -p core` のみで回る導線を確立（CLI構成変更で止まらない）。
- ResultBox移行: `box_trait::ResultBox` 参照を `boxes::ResultBox` へ一掃し、レガシー実装を段階的に削除。

### 長期（Phase 8.4–8.6連動）
- AST→MIR Lowering完全化（8.4）: すべての言語要素を新MIRへ安定lower。未対応分岐をゼロに。
- MIRダイエット（8.5）: 命令の統一・簡素化（TypeOp/WeakRef/Barrierの統合効果）を活用し最小集合へ。
- VM性能改善（8.6）: Hot-path（BoxCall/Array/Map）高速化、And/OrのVM未実装の解消、BoxRef演算の抜けを補完。

### Phase 10 着手ゲート（Cranelift前提条件）
- [ ] MIR26整合化完了（Printer/Verifier/Optimizerが26命令で一致・効果表記統一）
- [ ] Loop SSA復帰（Phi/Seal/Pred更新がVerifierで合格）
- [ ] TypeOp網羅（is/as/isType/asTypeの早期lowering＋Optimizer診断ONで回帰ゼロ）
- [ ] 軽量スナップショット緑（TypeOp/extern_call/loop/await/boxcall 代表ケース）
- [ ] P2PBox再設計（Phase 9.79）完了・E2Eパス
- [ ] CLI分離テスト（`cargo test -p core`）が安定実行

### ⚠️ トラブルノート（VM and/or 追跡用）
- 症状: `--backend vm` で `And` が `execute_binop` の短絡パスを経由せず、`execute_binary_op` 汎用パスで `Type error`。
- 状況: `MIR --dump-mir` では `bb0: %X = %A And %B` を出力（BuilderはOK）。
- 差分検証: `execute_instruction`/`execute_binop` に挿入したデバッグ出力が未出力→実行経路の相違が濃厚。
- 対応: 実行バイナリの署名チェックとコードパスの網羅的ログ追加でルート確定→修正。

### 🔧 再開ガイド（最短手順）
- ログ抑制: 一時ログはそのままでもOK。静かにしたい場合は `NYASH_VM_DEBUG_CMP=0 NYASH_VM_DEBUG_ANDOR=0` で実行。
- 検証: 上記スモーク3本（compare/and_or/and_or_truthy）→`tools/ci_check_golden.sh`。
- 進める順: Phi正規化 → String/Bool re-export → VM分割設計 → スナップショット追加。

### ✅ 小タスク完了（2025-08-25 深夜）
- Verifier: Barrierの軽い文脈診断を追加（`NYASH_VERIFY_BARRIER_STRICT=1`で有効）。
- ResultBox移行TODOを追加（`docs/development/current/RESULTBOX_MIGRATION_TODO.md`）。
- VM: 旧`box_trait::ResultBox`扱いのデプリケーション警告を最小抑制（完全移行までの暫定）。

### ✅ 小タスク完了（2025-08-26 早朝）
- VM: `as_bool()` の拡張（IntegerBox/ Void を真理値化）。
- テスト: `and_or_truthy_vm.nyash` 追加。
- 基本ボックス統一（第一弾）: `IntegerBox` 実体の一本化（re-export）。

### ▶ リファクタリング開始（控えめ）
- 方針: 肥大化防止のため`src/backend/vm.rs`を2分割のみ実施。
  - `vm_values.rs`: `execute_binary_op`/`execute_unary_op`/`execute_compare_op` を移動。
  - `vm_boxcall.rs`: `call_box_method` を移動（`call_unified_method`は現状のまま）。
- 影響最小・挙動非変更で、可読性と責務分離を先行する。

## 🧭 リファクタリングゴール（AI一目見て分かる導線）

### 目的/原則
- 明確責務: 各モジュールが1行で説明できる目的・責務を持つ
- 入口統一: VM/MIRの読む順番が固定され、迷わない
- ファイル肥大防止: 1000行超は段階分割（まずVMホットパス）
- ドキュメント導線: quick-referenceで最短経路を提示
- 振る舞い不変: すべてのゴールデン/CIが緑

### マイルストーン（受入基準）
- M1: VM導線完成（達成）
  - vm.rs → vm_instructions.rs → vm_values.rs / vm_boxcall.rs / vm_stats.rs / vm_phi.rs
  - code-map（docs/quick-reference/code-map.md）で入口と責務を明記
  - 受入: build OK / golden 緑 / 実行変化なし
- M2: モジュールヘッダ（着手）
  - Purpose/Responsibilities/Key APIs/Typical Callers を各VMモジュール先頭に追記
  - 受入: 各モジュール先頭4行程度の要約とcode-mapの整合
- M3: 1000行級の導線整備（設計メモ）
  - interpreter/plugin_loader.rs: v2主導の導線明文化（削除は後）
  - interpreter/objects.rs: 可視性/フィールド/ライフサイクルの分割指針をTODO化
- M4: レガシー移行（土台）
  - RESULTBOX_MIGRATION_TODOの維持、VMは新旧両対応のまま

### 実行順（小さく確実に）
1) M2ヘッダ追加（VM系）
2) M3 plugin_loader 導線の明文化（コメント/TODO/コードマップ）
3) M3 objects.rs 分割設計メモ
4) 用語の統一・code-map追従
5) golden + 重要E2Eを確認

### ⚠️ MIRビルダー引き継ぎポイント（ChatGPT5さんへ）
- **状況**: MIRビルダーのモジュール化完了（Phase 1-8コミット済み）
- **問題**: MIR命令構造の変更により、expressions.rsでエラー発生
  - `Call`命令: `function`→`func`, `arguments`→`args`
  - `ArrayAccess`, `ArrayLiteral`, `Await`ノードが削除？
  - effectsフィールドの有無が命令により異なる
  - TypeOpKindのインポートパスエラー
  - loop_builder.rsでのプライベートフィールドアクセス問題
- **現在の対応**: 
  - builder_modularizedディレクトリに一時退避
  - 元のbuilder.rsでフルビルド可能な状態に復帰
  - ChatGPT5さんのMIR命令変更に合わせた調整が必要

2. **VMの既知の問題**
   - 論理演算子（and, or）がBinOpとして未実装
   - エラー: `Type error: Unsupported binary operation: And on Bool(true) and Bool(false)`
   - インタープリターでは動作するがVMで動作しない
   - **新発見**: BoxRef(IntegerBox) + BoxRef(IntegerBox)のような演算も未対応
     - execute_binary_opにBoxRef同士の演算ケースが不足

## ✅ 直近の完了
1. VMモジュールのリファクタリング完了（2025-08-25）
   - execute_instruction関数を29個のハンドラーに分割
   - vm_instructions.rsモジュール作成（487行）
   - execute_instruction_old削除（691行削減）
   - vm.rs: 2075行→1382行（33%削減）
2. ドキュメント再編成の完了（構造刷新）
2. VM×プラグインのE2E整備（FileBox/Net）
   - FileBox: open/write/read, copyFrom(handle)（VM）
   - Net: GET/POST（VM）、404/500（Ok(Response)）、unreachable（Err(ErrorBox)）
3. VM命令カウンタ＋時間計測のCLI化（`--vm-stats`, `--vm-stats-json`）とJSON出力対応
   - サンプル/スクリプト整備（tools/run_vm_stats.sh、local_tests/vm_stats_*.nyash）
4. MIR if-merge 修正（retがphi dstを返す）＋ Verifier強化（mergeでのphi未使用検知、支配関係チェック導入）
5. VMの健全化（分岐・比較・Result）
   - Compare: Void/BoolのEq/Ne定義（順序比較はTypeError）
   - Branch条件: `BoxRef(BoolBox)→bool`／`BoxRef(VoidBox)→false`／`Integer≠0→true`
   - ResultBox: 新旧両実装への動的ディスパッチ統一（isOk/getValue/getError）
6. VMビルトイン強化（Array/Map/Socket）
   - ArrayBox/MapBox: 代表メソッドをVM統合ディスパッチで実装（push/get/set/size等）
   - SocketBox: `acceptTimeout(ms)`（void）/ `recvTimeout(ms)`（空文字）を追加
   - E2E追加: `socket_timeout_server.nyash` / `socket_timeout_client.nyash`
7. E2E拡張（Net/Socket）
   - HTTP: 大ボディ取得クライアント `local_tests/http_big_body_client.nyash`
   - Socket: 反復タイムアウト検証 `local_tests/socket_repeated_timeouts.nyash`
   - インタープリタ: SocketBoxの `acceptTimeout/recvTimeout` を結線
8. VM/MIRの健全化（Builder/VM）
   - Compare拡張: Float/Int-Float混在をサポート（Eq/Ne/Lt/Le/Gt/Ge）
   - TypeOp(Check)最小意味論実装（Integer/Float/Bool/String/Void/Box名）
   - ArrayGet/ArraySet（VM）本実装（ArrayBox.get/setへ橋渡し）
   - Array/Mapをidentity扱い（clone_or_shareがshareを選択）
   - BoxCallにArrayBox fast-path（BoxRefからget/set直呼び）
   - me参照の安定化（fallback時に一度だけConstを発行しvariable_mapに保持）
   - デバッグ: `NYASH_VM_DEBUG_BOXCALL=1` でBoxCallの受け手/引数/経路/結果型を標準エラーに出力
9. ドキュメント追加・更新
   - MIR→VMマッピング（分岐条件の動的変換、Void/Bool比較）
   - VM README（SocketBoxタイムアウト/E2E導線・HTTP Result整理）
   - 26命令ダイエット: PoCフラグと進捗追記（TypeOp/WeakRef/Barrier）
10. CI: plugins E2E ジョブ（Linux）を追加

### 📊 大規模ファイルのリファクタリング候補
1. src/interpreter/objects.rs (1,272行) - オブジェクト処理の分割
2. src/interpreter/plugin_loader.rs (1,217行) - v2があるので削除候補？
3. src/interpreter/expressions/calls.rs (1,016行) - 関数呼び出し処理の分割

## 🚧 次にやること（再開方針）

1) MIR26 前進（短期）
   - プリンタ拡張: `TypeOp/WeakRef/Barrier` を `--mir-verbose` に明示表示
   - スナップショット整備: 代表ケースで flag ON/OFF のMIR差分固定化
   - vm-stats差分: `weak_field_poc.nyash` 等で JSON 取得・比較（キー: TypeOp/WeakRef/Barrier）
   - 旗: `mir_typeop_poc`（TypeCheck/Cast→TypeOp）、`mir_refbarrier_unify_poc`（Weak*/Barrier→統合）

2) Builder適用拡大（短期〜中期）
   - 言語 `is/as` 導線（最小でも擬似ノード）→ `emit_type_check/emit_cast` へ配線
   - 弱参照: 既存の `RefGet/RefSet` パスは弱フィールドで `WeakLoad/WeakNew`＋Barrier（flag ONで統合命令）
   - 関数スタイル `isType/asType` の早期loweringを強化（`Literal("T")` と `new StringBox("T")` を確実に検出）
   - `print(isType(...))` の未定義SSA回避（print直前で必ず `TypeOp` のdstを生成）

補足: 近々/中期で並行対応する項目
- Loop SSA復帰（Phi挿入/Seal/Pred更新の段階適用、簡易loweringの置換）
- Builder移行完了（`builder.rs` → `builder_modularized/*`、命令フィールド名・効果の完全一致）
- CLI分離テスト導線（`cargo test -p core` 単独で回る構成）
- ResultBox移行（`box_trait::ResultBox` → `boxes::ResultBox`、レガシー段階削除）

3) VM/Verifierの補強（中期）
   - `TypeOp(Cast)` の数値キャスト（Int/Float）安全化、誤用時TypeError整備
   - Verifierに26命令整合（Barrier位置、WeakRef整合、支配関係）チェックを追加

4) VM×プラグインE2Eの維持（短期）
   - HTTP/Socketの回帰確認（Void防御・遅延サーバ軽量化は済）
   - 必要に応じて `VM_README.md` にTips追記

5) BoxCall高速化（性能段階）
   - `--vm-stats` ホットパス特定後、Fast-path/キャッシュ適用

## 🐛 既知の問題（要フォロー）
- 関数スタイル `isType(value, "Integer")` が一部ケースで `TypeOp` にloweringされず、`print %X` が未定義参照になる事象
  - 現状: `asType` は `typeop cast` に変換されるが、`isType` が欠落するケースあり
  - 仮対処: Interpreterに `is/isType/as/asType` のフォールバックを実装済（実行エラー回避）
  - 恒久対処（次回対応）:
    - Builderの早期loweringをFunctionCall分岐で強化（`Literal`/`StringBox`両対応、`print(...)` 内でも確実にdst生成）
    - Optimizerの安全ネット（BoxCall/Call→TypeOp）を `isType` パターンでも確実に発火させる（テーブル駆動の判定）

- 命令の二重化（Legacy vs Unified）
  - 状況: `MirInstruction` に旧系（`TypeCheck/Cast/WeakNew/WeakLoad/BarrierRead/BarrierWrite`）と統合系（`TypeOp/WeakRef/Barrier`）が併存。
  - リスク: ドキュメントは26命令（統合系）で凍結のため、実装との二重化がバグ温床に。
  - 当面のガード: 26命令同期テストを追加（`src/mir/instruction_introspection.rs`）。ドキュメント≡実装名がズレたら赤。
  - 次アクション:
    1) Optimizer診断に「旧命令検出」フラグを追加（`NYASH_OPT_DIAG_FORBID_LEGACY=1`で旧命令が存在したら失敗）。
    2) Printer/Verifier/Optimizerの表記・効果を統合側に寄せる（表示も統一）。
    3) 段階的に旧命令をcfgで囲む or 内部で統合命令へアダプト（最終的に旧命令経路を削除）。

## ⏸️ セッション再開メモ（次にやること）
- [ ] Builder: `extract_string_literal` の `StringBox`対応は導入済 → `FunctionCall` 早期loweringの再検証（`print(isType(...))` 直下）
- [ ] Optimizer: `Call` 形式（関数呼び出し）でも `isType/asType` を検出して `TypeOp(Check/Cast)` に置換する安全ネットの強化とテスト
- [ ] MIRダンプの確認：`local_tests/typeop_is_as_func_poc.nyash` に `typeop check/cast` が両方出ることを確認
- [ ] スナップショット化：`typeop_is_as_*_poc.nyash` のダンプを固定し回帰検出

## ▶ 補助コマンド（検証系）
```bash
# リビルド
cargo build --release -j32

# 関数スタイルのMIRダンプ確認（isType/asType）
./target/release/nyash --dump-mir --mir-verbose local_tests/typeop_is_as_func_poc.nyash | sed -n '1,200p'

# メソッドスタイルのMIRダンプ確認（is/as）
./target/release/nyash --dump-mir --mir-verbose local_tests/typeop_is_as_poc.nyash | sed -n '1,200p'
```

### 🆕 開発時の可視化・診断（最小追加）
- `--mir-verbose-effects`: MIRダンプ行末に効果カテゴリを表示（`pure|readonly|side`）
  - 例: `nyash --dump-mir --mir-verbose --mir-verbose-effects local_tests/typeop_is_as_func_poc.nyash`
- `NYASH_OPT_DIAG_FAIL=1`: Optimizer診断で未lowering（`is|isType|as|asType`）検知時にエラー終了（CI向け）
  - 例: `NYASH_OPT_DIAG_FAIL=1 nyash --dump-mir --mir-verbose local_tests/typeop_diag_fail.nyash`
- Builder生MIRスナップショット: `tools/snapshot_mir.sh <input.nyash> [output.txt]`
  - 例: `tools/snapshot_mir.sh local_tests/typeop_is_as_func_poc.nyash docs/status/golden/typeop_is_as_func_poc.mir.txt`
- ゴールデン比較（ローカル/CI）: `tools/ci_check_golden.sh`（代表2ケースを比較）
  - 例: `./tools/ci_check_golden.sh`（差分があれば非ゼロ終了）
  
補足: ASTの形状確認は `--dump-ast` を使用。

#### 代表スナップショット対象（拡張）
- TypeOp系: `typeop_is_as_poc.nyash`, `typeop_is_as_func_poc.nyash`
- extern_call: `http_get_*.nyash`, `filebox_copy_from_handle.nyash`
- loop/await: `loop_phi_seal_poc.nyash`, `await_http_timeout_poc.nyash`
- boxcall: `array_map_fastpath_poc.nyash`, `boxcall_identity_share_poc.nyash`

## ▶ 実行コマンド例

計測実行:
```bash
tools/run_vm_stats.sh local_tests/vm_stats_http_ok.nyash vm_stats_ok.json
tools/run_vm_stats.sh local_tests/vm_stats_http_err.nyash vm_stats_err.json
tools/run_vm_stats.sh local_tests/vm_stats_http_404.nyash vm_stats_404.json
tools/run_vm_stats.sh local_tests/vm_stats_http_500.nyash vm_stats_500.json
```

VM×プラグインE2E:
```bash
cargo test -q --features plugins e2e_interpreter_plugin_filebox_close_void
cargo test -q --features plugins e2e_vm_plugin_filebox_close_void
```

MIR26 PoC（弱参照・Barrier統合）:
```bash
# 弱フィールドPoC（flag OFF: WeakNew/WeakLoad/BarrierRead/Write）
NYASH_VM_STATS=1 NYASH_VM_STATS_JSON=1 ./target/release/nyash --backend vm --vm-stats --vm-stats-json local_tests/weak_field_poc.nyash > vm_stats_weak_default.json

# flag ON: WeakRef/Barrier 統合
cargo build --release --features mir_refbarrier_unify_poc -q
NYASH_VM_STATS=1 NYASH_VM_STATS_JSON=1 ./target/release/nyash --backend vm --vm-stats --vm-stats-json local_tests/weak_field_poc.nyash > vm_stats_weak_unified.json
```

MIRダンプ（プリンタ拡張後の確認）:
```bash
./target/release/nyash --dump-mir --mir-verbose local_tests/weak_field_poc.nyash | sed -n '1,200p'
```

MIRダンプ/検証:
```bash
nyash --dump-mir --mir-verbose examples/plugin_box_sample.nyash
nyash --verify examples/plugin_box_sample.nyash
```

## 🔭 26命令ターゲット（合意ドラフト）
- コア: Const / Copy / Load / Store / BinOp / UnaryOp / Compare / Jump / Branch / Phi / Return / Call / BoxCall / NewBox / ArrayGet / ArraySet / RefNew / RefGet / RefSet / Await / Print / ExternCall(最小) / TypeOp(=TypeCheck/Cast統合) / WeakRef(=WeakNew/WeakLoad統合) / Barrier(=Read/Write統合)
- メタ降格: Debug / Nop / Safepoint（ビルドモードで制御）

---
最終更新: 2025年8月25日（MIR26整合確認・VM and/or実装・Loop補強・CLI分離テスト導線/ResultBox移行TODO追加／スナップショット最小運用）

## 🔁 再起動後の再開手順（ショート）
```bash
# 1) ビルド
cargo build --release -j32

# 2) plugins E2E（Linux）
cargo test --features plugins -q -- --nocapture

# 3) VM Stats 代表値の再取得（任意）
tools/run_vm_stats.sh local_tests/vm_stats_http_ok.nyash vm_stats_ok.json
tools/run_vm_stats.sh local_tests/vm_stats_http_err.nyash vm_stats_err.json

# 4) SocketBox タイムアウト確認（任意）
./target/release/nyash local_tests/socket_timeout_server.nyash
./target/release/nyash local_tests/socket_timeout_client.nyash

# 5) 反復タイムアウト確認（任意）
./target/release/nyash local_tests/socket_repeated_timeouts.nyash

# 6) HTTP 大ボディ確認（任意）
./target/release/nyash local_tests/http_big_body_client.nyash

# 7) VM BoxCall デバッグ（任意）
NYASH_VM_DEBUG_BOXCALL=1 ./target/release/nyash --backend vm local_tests/test_vm_array_getset.nyash
```
- MIR26 整合（候補1）確認:
  - Printer: `--mir-verbose-effects` の `pure|readonly|side` 表記と TypeOp/WeakRef/Barrier/ExternCall の表示が整合。
  - Verifier: SSA/支配/CFG/merge-phi に加え WeakRef/Barrier の最小検証＋Strict Barrier診断を実装（環境変数でON）。
  - Optimizer: 未lowering検知（is/as/isType/asType）をBoxCall/Call両経路で検出、`NYASH_OPT_DIAG_FAIL=1` と連携。
  - 代表スナップショット: extern_call/loop/boxcall/typeop_mixed をCIに追加、全件緑。
  - 注: WeakRef/Barrier の“統合”はPoCフラグで切替可能（レガシー命令も支援）—MIR26はドキュメントの正典、実装は互換を維持。


## ✅ 本日の成果（9.78h）
- MIR26命令のコード≡ドキュメント同期テストを追加（tests/mir_instruction_set_sync.rs）→ 緑
- スナップショット安定化（tools/snapshot_mir.sh に静音フィルタ、golden全緑）
- Phi選択の委譲スケルトンをVMへ実装（LoopExecutor::execute_phi）
- ResultBox移行スイープ（VMのレガシーbox_trait::ResultBox経路を削除、boxes::ResultBoxに統一）
- VM BoxRef演算フォールバック拡張（toString→parse<i64>、比較/四則/混在を広くカバー）
- ロガー導入（環境変数ONでのみ出力）:
  - NYASH_DEBUG_EFFECTS / NYASH_DEBUG_VERIFIER / NYASH_DEBUG_MIR_PRINTER / NYASH_DEBUG_TRYCATCH
  - NYASH_VM_DEBUG_REF / NYASH_VM_DEBUG_BIN / 既存の NYASH_VM_DEBUG_* 系
- VMユーザBoxのBoxCallをInstance関数へ橋渡し（InstanceBox → Class.method/arity へ委譲）

## 🧪 テスト概況（要点）
- 緑: 175件 / 赤: 1件（`mir::verification::tests::test_if_merge_uses_phi_not_predecessor`）
- 失敗要旨: Mergeブロックで前任値使用と判定（DominatorViolation/MergeUsesPredecessorValue）
- 生成MIR（CLI/Builder）はmergeにphiを含むため、Verifier側の検査条件かvariable_map束縛の拾い漏れの可能性。

### デバッグ用コマンド（ログON例）
```bash
# Effects純度
NYASH_DEBUG_EFFECTS=1 cargo test --lib mir::effect::tests::test_effect_mask_creation -- --nocapture

# Verifier（phi/merge）
NYASH_DEBUG_VERIFIER=1 cargo test --lib mir::verification::tests::test_if_merge_uses_phi_not_predecessor -- --nocapture

# Try/CatchのLowering/Printer
NYASH_DEBUG_TRYCATCH=1 NYASH_DEBUG_MIR_PRINTER=1 cargo test --lib mir::tests::test_try_catch_compilation -- --nocapture

# VM 参照/演算
NYASH_VM_DEBUG_REF=1 cargo test --lib backend::vm::tests::test_vm_user_box_birth_and_method -- --nocapture
NYASH_VM_DEBUG_BIN=1 cargo test --lib tests::mir_vm_poc::test_boxref_arith -- --nocapture
```

## 🎯 次の着手（残1の精密駆逐）
1) Verifierのmerge-phi検査を補強（phi dst/variable束縛の対応づけ強化 or 条件緩和の適用）
2) Builderのvariable_map束縛拾い漏れがあれば補修（Program直下パターンの明示）
3) 代表MIRダンプをfixture化して回帰チェック（phi存在の有無を機械判定）

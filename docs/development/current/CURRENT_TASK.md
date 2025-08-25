# 🎯 CURRENT TASK - 2025年8月25日（状況整理）

## 🚨 現在の状況（2025-08-25）
1. **✅ MIRビルダーリファクタリング完了🎉**
   - mir/builder.rs: 1547行の大規模モジュール → **モジュール分割完了**
   - 新構造: `src/mir/builder/` ディレクトリ
     - `mod.rs`: 公開API定義
     - `core.rs`: MirBuilder本体 + コア機能 (205行)
     - `expressions.rs`: 式変換処理 (621行) - 全expression関数実装済み
     - `statements.rs`: 文変換処理 (プレースホルダー) 
     - `control_flow.rs`: 制御フロー構築 (プレースホルダー)
     - `box_handlers.rs`: Box関連処理 (プレースホルダー)
   - **ビルド確認**: 新構造でコンパイル正常完了 ✅
   - expressions.rsに移動した主要関数:
     - build_expression (巨大match文)
     - build_literal, build_binary_op, build_unary_op
     - build_variable_access, build_field_access
     - build_function_call, build_method_call
     - build_from_expression, build_me_expression
     - build_assignment, build_field_assignment
     - build_new_expression, build_await_expression
   
### 🎯 次のリファクタリング計画
**MIRビルダーの分割案（40関数を機能別に分類）**:

1. **`mir/builder/core.rs`**: MirBuilder本体とコア機能（8関数）
   - `new()`, `emit_instruction()`, `ensure_block_exists()`, `start_new_block()`
   - `emit_type_check()`, `emit_cast()`, `emit_weak_new()`, `emit_weak_load()`
   - `emit_barrier_read()`, `emit_barrier_write()`

2. **`mir/builder/expressions.rs`**: 式の変換処理（11関数）
   - `build_expression()`, `build_literal()`, `build_binary_op()`, `build_unary_op()`
   - `build_variable_access()`, `build_function_call()`, `build_field_access()`
   - `build_me_expression()`, `build_method_call()`, `build_from_expression()`
   - `build_await_expression()`

3. **`mir/builder/statements.rs`**: 文の変換処理（9関数）
   - `build_module()`, `build_block()`, `build_assignment()`, `build_field_assignment()`
   - `build_print_statement()`, `build_local_statement()`, `build_return_statement()`
   - `build_throw_statement()`, `build_nowait_statement()`

4. **`mir/builder/control_flow.rs`**: 制御フロー構築（3関数）
   - `build_if_statement()`, `build_loop_statement()`, `build_try_catch_statement()`

5. **`mir/builder/box_handlers.rs`**: Box関連の特殊処理（3関数）
   - `build_new_expression()`, `build_box_declaration()`, `build_static_main_box()`

### 📊 分析結果
- **最も大きい関数**: `build_method_call()` (157行)
- **複雑度が高い関数**: `build_expression()` (183行のmatch文)
- **特に分離すべき部分**: 式ビルダー（全体の27.5%）

### 🔧 実装計画
1. **✅ Phase 1完了**: モジュール構造の作成
   - ✅ `mir/builder/`ディレクトリ作成
   - ✅ `mod.rs`で公開APIを定義
   - ✅ 各サブモジュールファイルを作成
   - ✅ ビルド確認: 新構造でコンパイル成功

2. **✅ Phase 2完了**: 演算子関数群移動
   - ✅ build_literal() → expressions.rs (17行)
   - ✅ build_binary_op() → expressions.rs (25行)  
   - ✅ build_unary_op() → expressions.rs (15行)
   - ✅ convert_binary_operator() + convert_unary_operator() (24行)
   - ✅ BinaryOpType enum定義追加
   - **総移動**: 81行の式処理ロジック完了
   - **ビルド確認**: 正常完了（警告数：51個）
   - **中間commit**: cc2a5c2 完了

3. **🎯 Phase 3候補**: 大型関数移動

3. **Phase 3**: テストとビルド確認
   - 各段階でビルドが通ることを確認
   - 既存のMIRテストが動作することを確認

### ⚠️ リファクタリング時の注意点
- `pub(super)`の可視性に注意（モジュール間で調整が必要）
- `self`参照が多いため、トレイトの導入も検討
- SSA構築に関わる`variable_map`等の共有状態に注意
- **nekocodeの精度について**: 60%信頼度は誤検出が多いため参考程度に
  - 外部ツール（cargo clippy等）が使えない環境では精度が低下
  - 実際に使われている関数も未使用と判定される可能性高い


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
最終更新: 2025年8月23日（VM強化・E2E拡張・me参照安定化・TypeOp/WeakRef/Barrier PoC完了／次段はプリンタ拡張・スナップショット・is/as導線）

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

# テスト実行ガイド

## 📁 **テストファイル配置ルール（超重要！）**

⚠️ **ルートディレクトリの汚染防止ルール** ⚠️
```bash
# ❌ 絶対ダメ：ルートで実行
./target/release/nyash test.nyash        # ログがルートに散乱！
cargo test > test_output.txt             # 出力ファイルがルートに！

# ✅ 正しい方法：必ずディレクトリを使う
cd local_tests && ../target/release/nyash test.nyash
./target/release/nyash local_tests/test.nyash
```

**必須ルール：**
- **テストファイル**: 必ず `local_tests/` に配置
- **ログファイル**: 環境変数で `logs/` に出力するか、実行後即削除
- **デバッグ出力**: `local_tests/` または `logs/` に保存
- **一時ファイル**: `/tmp/` を使用

**なぜ毎回ルートが散らかるのか：**
1. テスト実行時にカレントディレクトリにログ出力
2. エラー時のデバッグファイルが自動削除されない
3. VM統計やMIRダンプがデフォルトでカレントに出力

## 🧪 テスト実行

```bash
# 基本機能テスト
cargo test

# テストファイル作成・実行例
mkdir -p local_tests
echo 'print("Hello Nyash!")' > local_tests/test_hello.nyash
./target/debug/nyash local_tests/test_hello.nyash

# 演算子統合テスト（local_testsから実行）
./target/debug/nyash local_tests/test_comprehensive_operators.nyash

# 実用アプリテスト
./target/debug/nyash app_dice_rpg.nyash

# JIT 実行フラグ（CLI）
./target/release/nyash --backend vm \
  --jit-exec --jit-stats --jit-dump --jit-threshold 1 \
  --jit-phi-min --jit-hostcall --jit-handle-debug \
  examples/jit_branch_demo.nyash
# 既存の環境変数でも可: 
#   NYASH_JIT_EXEC/NYASH_JIT_STATS(/_JSON)/NYASH_JIT_DUMP/NYASH_JIT_THRESHOLD
#   NYASH_JIT_PHI_MIN/NYASH_JIT_HOSTCALL/NYASH_JIT_HANDLE_DEBUG

# HostCallハンドルPoCの例
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_array_param_call.nyash
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_map_param_call.nyash
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_map_int_keys_param_call.nyash
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_string_param_length.nyash
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_string_is_empty.nyash
```

## PHI ポリシー（Phase‑15）と検証トグル

- Phase‑15 では PHI‑on（MIR14）が既定だよ。MIR ビルダーがブロック先頭へ `Phi` を配置し、検証も SSA 前提で実施するよ。
- レガシー検証で edge-copy 互換が必要なら `NYASH_MIR_NO_PHI=1` を明示してね（`NYASH_VERIFY_ALLOW_NO_PHI=1` も忘れずに）。
- 詳細は `docs/reference/mir/phi_policy.md` を参照してね。

テスト時の環境（推奨）
```bash
# 既定: 何も設定しない → PHI-on

# レガシー PHI-off の再現が必要なときだけ明示的に切り替え
export NYASH_MIR_NO_PHI=1
export NYASH_VERIFY_ALLOW_NO_PHI=1

# さらに edge-copy 規約を厳格チェックしたい場合（任意）
export NYASH_VERIFY_EDGE_COPY_STRICT=1
```

PHI-on の補助トレース
- `NYASH_LLVM_TRACE_PHI=1` と `NYASH_LLVM_TRACE_OUT=tmp/phi.jsonl` を組み合わせると、PHI がどの predecessor から値を受け取っているかを確認できるよ。

## PHI 配線トレース（JSONL）

- 目的: LLVM 側の PHI 配線が、PHI-on で生成された SSA と legacy edge-copy (PHI-off) の両方に整合しているかを可視化・検証する。
- 出力: 1 行 JSON（JSONL）。`NYASH_LLVM_TRACE_OUT=<path>` に追記出力。
- イベント: `finalize_begin/finalize_dst/add_incoming/wire_choose/snapshot` など（pred→dst 整合が分かる）

クイック実行
```bash
# すべてPHI‑offで OK。llvmlite ハーネスと if-merge プリパスをON
NYASH_LLVM_TRACE_PHI=1 NYASH_LLVM_TRACE_OUT=tmp/phi.jsonl \
NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_PREPASS_IFMERGE=1 \
bash tools/test/smoke/llvm/phi_trace/test.sh

# 結果の検証（要: python3）
python3 tools/phi_trace_check.py --file tmp/phi.jsonl --summary
```

ショートカット
- `tools/smokes/phi_trace_local.sh`（ビルド→スモーク→チェックを一括）
- `tools/smokes/fast_local.sh` は `NYASH_LLVM_TRACE_SMOKE=1` でオプション実行


## 🔌 **プラグインテスター（BID-FFI診断ツール）**
```bash
# プラグインテスターのビルド
cd tools/plugin-tester
cargo build --release

# プラグインの診断実行
./target/release/plugin-tester ../../plugins/nyash-filebox-plugin/target/debug/libnyash_filebox_plugin.so

# 出力例：
# Plugin Information:
#   Box Type: FileBox (ID: 6)  ← プラグインが自己宣言！
#   Methods: 6
#   - birth [ID: 0] (constructor)
#   - open, read, write, close
#   - fini [ID: 4294967295] (destructor)
```

**plugin-testerの特徴**:
- Box名を決め打ちしない汎用設計
- プラグインのFFI関数4つ（abi/init/invoke/shutdown）を検証
- birth/finiライフサイクル確認
- 将来の拡張: TLV検証、メモリリーク検出

## 🐛 デバッグ

### パーサー無限ループ対策（2025-08-09実装）
```bash
# 🔥 デバッグ燃料でパーサー制御
./target/release/nyash --debug-fuel 1000 program.nyash      # 1000回制限
./target/release/nyash --debug-fuel unlimited program.nyash  # 無制限
./target/release/nyash program.nyash                        # デフォルト10万回

# パーサー無限ループが検出されると自動停止＋詳細情報表示
🚨 PARSER INFINITE LOOP DETECTED at method call argument parsing
🔍 Current token: IDENTIFIER("from") at line 17
🔍 Parser position: 45/128
```

**対応状況**: must_advance!マクロでパーサー制御完全実装済み✅  
**効果**: 予約語"from"など問題のあるトークンも安全にエラー検出

### アプリケーション デバッグ
```nyash
// DebugBox活用
DEBUG = new DebugBox()
DEBUG.startTracking()
DEBUG.trackBox(myObject, "説明")
print(DEBUG.memoryReport())
```
## Macro-based Test Runner (MVP)

Nyash provides a macro-powered lightweight test runner in Phase 16 (MVP).

- Enable and run tests in a script file:
  - `nyash --run-tests apps/tests/my_tests.nyash`
  - Discovers top-level `test_*` functions and Box `test_*` methods (static/instance).
- Filtering: `--test-filter NAME` (substring match) or env `NYASH_TEST_FILTER`.
- Entry policy when a main exists:
  - `--test-entry wrap` → run tests then call original main
  - `--test-entry override` → replace entry with test harness only
  - Force apply: `NYASH_TEST_FORCE=1`
- Parameterized tests (MVP): `NYASH_TEST_ARGS_DEFAULTS=1` injects integer `0` for each parameter (static/instance tests).
- Exit code = number of failed tests (0 on success).

Notes
- The feature is behind the macro gate; CLI `--run-tests` enables it automatically.
- Future versions will add JSON-based per-test arguments and richer reporting.

# Smoke Test Debugging Guide

注記（ブランド/エイリアス）
- 新規ドキュメントやスクリプトでは HAKO_* を優先してください。
- 互換として NYASH_* も受理されます（未設定時に HAKO_* をマップ）。
- 例: `HAKO_VM_TRACE ≡ NYASH_VM_TRACE`, `HAKO_CLI_VERBOSE ≡ NYASH_CLI_VERBOSE`。

---

## 新機能（開発に便利）

1) 失敗の再現バンドル採取（SMOKES_CAPTURE=1）
- 失敗時に `tmp/smokes_capture/<test>_<kind>_<timestamp>/` へ expected/actual/env を保存
- 有効化: `SMOKES_CAPTURE=1 tools/smokes/v2/run.sh --profile quick`

2) 即席パリティチェッカー
- `tools/parity_check.sh file.nyash` または `-c 'code'` で VM↔LLVM を正規化比較
- スモーク無しの軽量デバッグに最適

3) ドクター（診断）
- `tools/ny_doctor.sh` で HAKO_ROOT/hako.toml/usingの有効状態（strategy/file許可）を一覧表示
- よくあるハマりへのアドバイスを自動表示

4) PHI の Fail‑Fast（開発時）
- `NYASH_VERIFY_PHI_STRICT=1` で、PHI inputs が到達可能 predecessor を網羅しているか検証
- 片側Only/ネスト else-if テストが quick/core/phi に常設

---

## 出力ノイズの扱い

- ランタイム末尾 `Result: <n>` は既定で抑止（`NYASH_NYRT_SILENT_RESULT=1`）
- ログは stderr へ。stdout はプログラム出力のみ
- 新規ノイズは smokes の `filter_noise` に集約

---

## プロファイルごとの流儀

- quick: dev 便利ON、軽い検証（PHI strictはON）
- integration-core: VM↔LLVM パリティ（プラグイン無し）
- plugins: プラグイン依存（未配置は SKIP）
- integration: apps 系（ハーネス＋VM比較、AOTリンクはバイパス）

詳細は `docs/guides/smokes-policy.md` を参照。

## Overview

このガイドでは、スモークテスト失敗時のデバッグ・トラブルシューティング手順を説明します。

## Quick Start: 失敗したテストのデバッグ

```bash
# Step 1: 失敗したテストを単体で再実行
bash tools/smokes/v2/profiles/quick/selfhost/test_name.sh

# Step 2: 詳細ログ付きで再実行
NYASH_CLI_VERBOSE=1 bash tools/smokes/v2/profiles/quick/selfhost/test_name.sh

# Step 3: テスト内のコマンドを手動実行
./target/release/hako apps/path/to/test.hako --dump-mir
```

---

## 📋 1. エラー出力の読み方

### 典型的な失敗出力フォーマット

```
Running test_name... FAIL (exit=1, 0.123s)
[WARN] Test file: /path/to/test.sh
----- LOG (tail -n 80) -----
[INFO] Environment check passed
[FAIL] Output mismatch:
[FAIL]   Expected: [42]
[FAIL]   Actual:   [43]
----- END LOG -----
```

### 重要な情報

| 項目 | 説明 | 例 |
|------|------|-----|
| **exit** | 終了コード | `exit=1` (失敗), `exit=0` (成功) |
| **Duration** | 実行時間 | `0.123s` |
| **Test file** | テストファイルのフルパス | `/home/.../test.sh` |
| **Expected** | 期待される出力 | `[42]` |
| **Actual** | 実際の出力 | `[43]` |
| **LOG** | 最後80行のログ | `tail -n 80` 相当 |

---

## 📁 2. ログ・一時ファイル

### ログファイルの場所

```bash
# テスト実行中の一時ログ
/tmp/nyash_smoke_<timestamp>_<pid>.log

# テストスクリプトが作成する一時ディレクトリ
/tmp/<test_name>_$$
/tmp/selfhost_<test_name>_$$

# 例
/tmp/selfhost_mir_m2_eq_true_vm_1234567/
```

### 一時ファイルの特徴

- **作成時**: テスト開始時
- **削除時**: テスト終了時（成功・失敗問わず）
- **保持方法**: テストスクリプト内の `rm -rf` をコメントアウト

### 一時ファイルを保持する方法

```bash
# 方法1: テストスクリプトを直接編集
# rm -rf "$TMP_DIR" をコメントアウト

# 方法2: テストスクリプトをコピーして修正
cp tools/smokes/v2/profiles/quick/selfhost/test.sh /tmp/debug_test.sh
# 編集: rm -rf "$TMP_DIR" 行をコメントアウト
bash /tmp/debug_test.sh
```

---

## 🔧 3. デバッグ手順（ステップバイステップ）

### Step 1: エラー内容確認

```bash
# 失敗したテストを再実行
tools/smokes/v2/run.sh --profile quick --filter "test_name"
```

**出力例**:
```
[FAIL] test_name output mismatch:
[FAIL]   Expected: [42]
[FAIL]   Actual:   [43]
```

### Step 2: 詳細ログ有効化

```bash
# デバッグ環境変数を有効化
NYASH_CLI_VERBOSE=1 \
SMOKES_DEV_LOG=1 \
bash tools/smokes/v2/profiles/quick/selfhost/test_name.sh
```

**出力例**:
```
----- [DEV LOG] full output begin -----
[using/alias] Resolving selfhost.vm.entry
[builder] Creating MirJsonBuilderMin
Hello World
----- [DEV LOG] full output end -----
```

### Step 3: テスト単体実行

```bash
# テストスクリプトを直接実行
bash tools/smokes/v2/profiles/quick/selfhost/test_name.sh
```

**利点**:
- プロファイルのオーバーヘッドなし
- 直接エラーメッセージが見える
- 一時ファイルのパスが分かる

### Step 4: コマンド単体実行

テストスクリプト内のコマンドを手動で実行:

```bash
# テストスクリプトから抽出したコマンド
NYASH_CLI_VERBOSE=1 \
./target/release/hako /tmp/test_dir/test.nyash
```

### Step 5: MIR確認

```bash
# MIRダンプで内部構造を確認
./target/release/hako --dump-mir /tmp/test_dir/test.nyash

# MIR詳細版
./target/release/hako --dump-mir --mir-verbose /tmp/test_dir/test.nyash

# JSON形式で出力
./target/release/hako --emit-mir-json /tmp/mir.json /tmp/test_dir/test.nyash
jq . /tmp/mir.json  # 整形表示
```

---

## 🎯 4. よくある失敗パターンと対処法

### パターン1: 出力ミスマッチ

**症状**:
```
[FAIL] Output mismatch:
[FAIL]   Expected: [42]
[FAIL]   Actual:   [43]
```

**原因リスト**:
1. ロジックバグ（コード側の問題）
2. 期待値が間違っている（テストスクリプト側の問題）
3. ノイズフィルタが不十分（デバッグメッセージが混入）
4. 出力フォーマットの変更（改行・空白など）

**対処法**:

```bash
# 1. 実際の出力を確認
bash test.sh 2>&1 | tee /tmp/output.txt

# 2. ノイズを確認
grep '\[' /tmp/output.txt  # デバッグメッセージを探す

# 3. 期待値と実際の値を比較
echo "Expected: [42]"
echo "Actual:   [43]"

# 4. MIRを確認（ロジックバグの場合）
./target/release/hako --dump-mir test.hako
```

### パターン2: タイムアウト

**症状**:
```
[FAIL] test_name (exit=124, 30.0s)
timeout exceeded
```

**原因リスト**:
1. 無限ループ（コード側のバグ）
2. タイムアウト設定が短すぎる
3. 重いテストケース（大量のデータ処理）
4. VM fuel制限に到達

**対処法**:

```bash
# 1. タイムアウトを延長
SMOKES_TIMEOUT_SEC=60 bash test.sh

# 2. VM fuel制限を緩和
NYASH_VM_MAX_INSTRUCTIONS=5000000 \
NYASH_VM_MAX_BLOCK_EXEC=1000000 \
bash test.sh

# 3. 無限ループの場合はトレース
NYASH_VM_TRACE=1 bash test.sh 2>&1 | tail -100

# 4. プロファイリング（時間がかかっている箇所を特定）
time ./target/release/hako test.hako
```

### パターン3: 依存関係エラー

**症状**:
```
[FAIL] missing_dep: selfhost.vm.entry
[using/resolve] Module not found: selfhost.vm.entry
```

**原因リスト**:
1. `hako.toml` に依存モジュールが未登録
2. ファイルパスが間違っている
3. `NYASH_USING=1` が未設定
4. モジュールエイリアスの設定ミス

**対処法**:

```bash
# 1. hako.toml を確認
cat hako.toml
# [modules.aliases]
# "selfhost.vm.entry" = "apps/selfhost/vm/entry.hako"

# 2. ファイルの存在確認
ls -la apps/selfhost/vm/entry.hako

# 3. using resolver 有効化
NYASH_USING=1 bash test.sh

# 4. デバッグトレース
NYASH_RESOLVE_TRACE=1 bash test.sh
```

### パターン4: プラグインエラー

**症状**:
```
[FAIL] Plugin not found: nyash-string-plugin
[WARN] Missing dynamic plugins: stringbox
```

**原因リスト**:
1. プラグインがビルドされていない
2. プラグインパスが間違っている
3. プラグインとの互換性問題

**対処法**:

```bash
# 1. プラグインをビルド
cd tools/plugin-tester
cargo build --release --bin plugin-tester
./target/release/plugin-tester build-all

# 2. プラグインを無効化して実行（回避策）
NYASH_DISABLE_PLUGINS=1 bash test.sh

# 3. プラグインチェックをスキップ
SMOKES_DISABLE_PLUGIN_CHECKS=1 bash test.sh
```

### パターン5: MIRエラー

**症状**:
```
[FAIL] MIR builder error: unknown instruction
Invalid instruction: operation on unborn instance
```

**原因リスト**:
1. MIR生成バグ（コンパイラ側）
2. birth() 呼び出し忘れ（使用側）
3. 未サポートの命令使用

**対処法**:

```bash
# 1. MIRダンプで確認
./target/release/hako --dump-mir test.hako

# 2. 詳細MIR
./target/release/hako --dump-mir --mir-verbose test.hako

# 3. birth契約チェック
./target/release/hako --dump-mir test.hako | grep -A5 "newbox"
```

---

## 💡 5. デバッグ環境変数一覧

### 実行環境制御

| 変数 | 用途 | 値 | 例 |
|------|------|-----|-----|
| `NYASH_CLI_VERBOSE` | 詳細診断 | 1 | `NYASH_CLI_VERBOSE=1` |
| `NYASH_DISABLE_PLUGINS` | プラグイン無効化 | 1 | `NYASH_DISABLE_PLUGINS=1` |
| `NYASH_USING` | using resolver有効化 | 1 | `NYASH_USING=1` |
| `NYASH_QUIET` | 出力抑制 | 1 | `NYASH_QUIET=1` |

### デバッグ出力

| 変数 | 用途 | 値 | 出力内容 |
|------|------|-----|----------|
| `NYASH_DUMP_MIR` | MIR出力 | 1 | MIR命令列 |
| `NYASH_VM_TRACE` | VM実行トレース | 1 | 1命令ごとの実行状況 |
| `NYASH_RESOLVE_TRACE` | Using解決トレース | 1 | モジュール解決過程 |
| `HAKO_VM_TRACE` | すけすけトレース | `op=boxcall;regs=1` | 命令詳細 |
| `HAKO_VM_STEP` | ステッパ機能 | 1 | 対話デバッグ |

### VM制限

| 変数 | 用途 | デフォルト | 推奨値（デバッグ時） |
|------|------|-----------|---------------------|
| `NYASH_VM_MAX_INSTRUCTIONS` | 最大命令数 | 1000000 | 5000000 |
| `NYASH_VM_MAX_BLOCK_EXEC` | 最大ブロック実行回数 | 200000 | 1000000 |
| `NYASH_VM_TOLERATE_VOID` | Void許容 | 0 | 1（開発時） |

### LLVM関連

| 変数 | 用途 | 値 | 説明 |
|------|------|-----|------|
| `NYASH_LLVM_USE_HARNESS` | Harness使用 | 1 | llvmlite経由実行 |
| `NYASH_LLVM_DUMP_IR` | LLVM IR出力 | 1 | `.ll` ファイル生成 |

### スモークテスト専用

| 変数 | 用途 | 値 | 説明 |
|------|------|-----|------|
| `SMOKES_DEV_LOG` | Dev log有効化 | 1 | using解決ログ表示 |
| `SMOKES_TIMEOUT_SEC` | タイムアウト | 秒数 | デフォルト12秒 |
| `SMOKES_DISABLE_PLUGIN_CHECKS` | Plugin check無効化 | 1 | プラグインエラー回避 |
| `SMOKES_USE_DEV` | --devフラグ付加 | 1 | 開発モード |
| `SMOKES_CLEAN_ENV` | 環境変数クリーン | 1 | テスト間の分離 |
| `SMOKES_ASI_STRIP_SEMI` | セミコロン除去 | 1 | デフォルト有効 |

---

## 🚀 6. クイック診断チートシート

### テスト失敗後の初動（1分以内）

```bash
# 1. 失敗したテスト単体実行
bash tools/smokes/v2/profiles/quick/selfhost/test_name.sh

# 2. エラーメッセージを確認
# - Expected vs Actual
# - exit code
# - LOG (tail -n 80)

# 3. テストファイルのパスを確認
# [WARN] Test file: /path/to/test.sh
```

### 詳細調査（5分以内）

```bash
# 1. 詳細モードで実行
NYASH_CLI_VERBOSE=1 bash test.sh 2>&1 | tee /tmp/debug.log

# 2. MIR確認
./target/release/hako --dump-mir /tmp/test_dir/test.hako

# 3. プラグイン無効で実行（回避策）
NYASH_DISABLE_PLUGINS=1 bash test.sh
```

### 深掘り調査（15分以内）

```bash
# 1. すけすけトレース
HAKO_VM_TRACE="op=boxcall,externcall;regs=1" \
./target/release/hako test.hako 2>&1 | less

# 2. ステッパ機能（対話デバッグ）
HAKO_VM_STEP=1 ./target/release/hako test.hako

# 3. JSON MIR出力・解析
./target/release/hako --emit-mir-json /tmp/mir.json test.hako
jq '.functions[0].blocks' /tmp/mir.json
```

---

## 📚 7. 関連ドキュメント

- **環境変数完全ガイド**: [docs/guides/env-variables.md](env-variables.md)
- **スモークプロファイル**: [docs/guides/smokes-profiles.md](smokes-profiles.md)
- **実行モードガイド**: [docs/guides/execution-modes-guide.md](execution-modes-guide.md)
- **Mini-VMデバッグ**: [docs/guides/mini-vm-debugging.md](mini-vm-debugging.md)
- **テストガイド**: [docs/guides/testing-guide.md](testing-guide.md)

---

## 🔍 8. 実践例

### 例1: 出力ミスマッチのデバッグ

```bash
# 症状
[FAIL] selfhost_mir_m2_eq_true_vm expected 1, got: 0

# Step 1: テスト単体実行
bash tools/smokes/v2/profiles/quick/selfhost/selfhost_mir_m2_eq_true_vm.sh

# Step 2: 詳細ログ
SMOKES_DEV_LOG=1 bash tools/smokes/v2/profiles/quick/selfhost/selfhost_mir_m2_eq_true_vm.sh

# Step 3: MIR確認
./target/release/hako --dump-mir /tmp/selfhost_mir_m2_eq_true_vm_*/driver.nyash

# Step 4: すけすけトレース
HAKO_VM_TRACE="op=compare;regs=1" \
NYASH_DISABLE_PLUGINS=1 \
./target/release/hako /tmp/selfhost_mir_m2_eq_true_vm_*/driver.nyash

# 出力例:
# [vm] bb=0 inst=2 compare kind=Eq lhs=v%1(7) rhs=v%2(7) dst=v%3 → 1
```

### 例2: タイムアウトのデバッグ

```bash
# 症状
[FAIL] test_name (exit=124, 30.0s)
timeout exceeded

# Step 1: タイムアウト延長
SMOKES_TIMEOUT_SEC=120 bash test.sh

# Step 2: VM fuel増加
NYASH_VM_MAX_INSTRUCTIONS=10000000 \
NYASH_VM_MAX_BLOCK_EXEC=2000000 \
bash test.sh

# Step 3: 無限ループ検出
NYASH_VM_TRACE=1 bash test.sh 2>&1 | tail -200 | grep "bb="
# 同じブロックが繰り返し実行されていないか確認
```

### 例3: Using解決エラーのデバッグ

```bash
# 症状
[FAIL] missing_dep: selfhost.vm.entry

# Step 1: hako.toml確認
cat hako.toml | grep -A5 modules.aliases

# Step 2: ファイル存在確認
ls -la apps/selfhost/vm/entry.hako

# Step 3: 解決トレース
NYASH_RESOLVE_TRACE=1 \
NYASH_USING=1 \
bash test.sh

# Step 4: 直接パス指定（回避策）
NYASH_ALLOW_USING_FILE=1 bash test.sh
```

---

## ⚡ 9. トラブルシューティング FAQ

### Q1: テストが全て SKIP される

**原因**: 環境変数による gate が有効

**対処**:
```bash
# 例: LLVM テストの場合
SMOKES_FORCE_LLVM=1 tools/smokes/v2/run.sh --profile integration

# 例: Dev テストの場合
SMOKES_ENABLE_DEV_PROGRAM2=1 bash test.sh
```

### Q2: ログが途中で切れる

**原因**: `SMOKES_NOTIFY_TAIL` のデフォルトが80行

**対処**:
```bash
# ログ行数を増やす
SMOKES_NOTIFY_TAIL=200 tools/smokes/v2/run.sh --profile quick

# または、テスト単体実行
bash test.sh 2>&1 | tee /tmp/full.log
```

### Q3: 一時ファイルが見つからない

**原因**: テスト終了時に自動削除される

**対処**:
```bash
# テストスクリプトをコピーして修正
cp test.sh /tmp/debug_test.sh
# 編集: rm -rf "$TMP_DIR" をコメントアウト
bash /tmp/debug_test.sh
```

### Q4: プラグインエラーが出る

**対処**:
```bash
# 方法1: プラグインを無効化
NYASH_DISABLE_PLUGINS=1 bash test.sh

# 方法2: プラグインチェックをスキップ
SMOKES_DISABLE_PLUGIN_CHECKS=1 bash test.sh

# 方法3: プラグインをビルド
cd tools/plugin-tester
./target/release/plugin-tester build-all
```

### Q5: MIR が出力されない

**対処**:
```bash
# 方法1: CLIフラグ使用（最も確実）
./target/release/hako --dump-mir test.hako

# 方法2: 環境変数（VM実行時）
NYASH_VM_DUMP_MIR=1 ./target/release/hako test.hako

# 方法3: JSON形式
./target/release/hako --emit-mir-json /tmp/mir.json test.hako
```

---

## 🎓 10. デバッグのベストプラクティス

### 1. テスト失敗時の基本手順

1. **エラーメッセージを読む** - Expected vs Actual を確認
2. **テスト単体実行** - プロファイルを経由せず直接実行
3. **詳細ログ有効化** - `NYASH_CLI_VERBOSE=1`
4. **MIR確認** - `--dump-mir` で内部構造を確認
5. **すけすけトレース** - `HAKO_VM_TRACE` で実行状況を観測

### 2. 効率的なデバッグ

- **段階的に詳細化**: まず簡単な方法から試す
- **ログを保存**: `tee` でログをファイルに保存
- **環境を分離**: `SMOKES_CLEAN_ENV=1` でテスト間の干渉を防ぐ
- **最小再現**: 失敗するテストケースを最小化

### 3. よくある罠

❌ **やってはいけないこと**:
- いきなり `HAKO_VM_TRACE` を使う（ノイズが多い）
- エラーメッセージを読まずにコードを変更
- 複数の環境変数を同時に変更

✅ **推奨する手順**:
- まずエラーメッセージを読む
- テスト単体実行で再現
- 段階的に詳細化（VERBOSE → MIR → TRACE）

---

## 📝 11. サンプル: デバッグセッション

完全なデバッグセッションの例:

```bash
# === 初期状態 ===
# テストが失敗
$ tools/smokes/v2/run.sh --profile quick
[FAIL] selfhost_mir_m2_eq_true_vm expected 1, got: 0

# === Step 1: テスト単体実行 ===
$ bash tools/smokes/v2/profiles/quick/selfhost/selfhost_mir_m2_eq_true_vm.sh
[FAIL] selfhost_mir_m2_eq_true_vm expected 1, got: 0

# === Step 2: 詳細ログ ===
$ SMOKES_DEV_LOG=1 bash tools/smokes/v2/profiles/quick/selfhost/selfhost_mir_m2_eq_true_vm.sh
----- [DEV LOG] full output begin -----
[using/alias] Resolving selfhost.vm.entry
0
----- [DEV LOG] full output end -----
# → 出力は "0" だが、期待値は "1"

# === Step 3: 一時ディレクトリを確認 ===
$ ls /tmp/selfhost_mir_m2_eq_true_vm_*/
driver.nyash

# === Step 4: MIR確認 ===
$ ./target/release/hako --dump-mir /tmp/selfhost_mir_m2_eq_true_vm_*/driver.nyash
function main:
  bb0:
    %1 = const i64 7
    %2 = const i64 7
    %3 = compare Eq %1, %2
    ret %3

# === Step 5: すけすけトレース ===
$ HAKO_VM_TRACE="op=compare;regs=1" \
  NYASH_DISABLE_PLUGINS=1 \
  ./target/release/hako /tmp/selfhost_mir_m2_eq_true_vm_*/driver.nyash
[vm] bb=0 inst=2 compare kind=Eq lhs=v%1(7) rhs=v%2(7) dst=v%3 → 1
1
# → compare命令は正しく "1" を返している！

# === Step 6: 原因特定 ===
# Mini-VM内部で ret命令の処理が間違っている可能性
# → Mini-VM (apps/selfhost/vm/boxes/mir_vm_min.hako) のコードを確認

# === Step 7: 修正・検証 ===
# （コード修正）
$ bash tools/smokes/v2/profiles/quick/selfhost/selfhost_mir_m2_eq_true_vm.sh
[PASS] selfhost_mir_m2_eq_true_vm prints 1
```

---

## 🎉 まとめ

スモークテスト失敗時のデバッグは以下の手順で効率的に行えます：

1. **エラーメッセージを読む** - Expected vs Actual
2. **テスト単体実行** - `bash test.sh`
3. **詳細ログ** - `NYASH_CLI_VERBOSE=1`
4. **MIR確認** - `--dump-mir`
5. **すけすけトレース** - `HAKO_VM_TRACE`

詳細は各セクションを参照してください。

# Phase 4 リスク分析とRollback戦略

**生成日**: 2025-10-14
**ステータス**: Phase 4実装前リスク評価
**目的**: Phase 4 (Dual Parser Harness) の全リスクを分析し、Rollback戦略を策定

---

## エグゼクティブサマリー

### 総リスク評価

- **総リスク数**: 18個
- **高リスク**: 3個（C言語メモリ管理、既存テスト破壊、デバッグ困難性）
- **中リスク**: 8個（ABI不一致、ビルド破壊、パフォーマンス等）
- **低リスク**: 7個（プラットフォーム依存、ドキュメント等）

### 推奨Rollback手順

**緊急時（1コマンド）**:
```bash
# Phase 4前の状態に戻す（最速）
git reset --hard <phase3完了時のcommit>
cargo build --release
bash tools/smokes/v2/run.sh --profile quick-selfhost
```

### 実行推奨度

⚠️ **条件付き実行推奨**

**理由**:
- ✅ MIR疎結合により致命的リスクは低い
- ✅ Rollback戦略は明確（3レベル）
- ⚠️ 以下の対策が**必須**:
  1. C ABI層を100-200行に限定
  2. feature flag 無効化機構を事前実装
  3. 継続的モニタリング（quick-selfhost毎回実行）

---

## 1. 技術的リスク分析

### 1.1 C言語実装リスク

#### リスク#1: メモリリーク
- **内容**: `malloc()`/`strdup()`したメモリを`free()`し忘れ
- **発生確率**: 中（C言語の典型的問題）
- **影響度**: 中（長時間実行で問題化、1回のparse程度なら無視可能）
- **検出容易性**: 高（valgrindで検出可能）
- **リスクスコア**: (中×中)/高 = **0.67** (中リスク)

**事前対策**:
- C ABI層を100-200行に限定
- メモリ所有権を単純化（参照渡しのみ、所有権移譲なし）
- RAII風パターン（init/cleanup対を必ず実装）

**検出方法**:
```bash
# valgrind でメモリリークチェック
valgrind --leak-check=full --show-leak-kinds=all \
  ./target/release/hako test.hako 2>&1 | tee valgrind.log
```

**対処方法**:
- free()呼び出し追加
- 所有権トランスファーを明確化（コメント記載）

---

#### リスク#2: バッファオーバーフロー
- **内容**: 固定長バッファへの書き込みで境界チェック不足
- **発生確率**: 低（最小実装、固定長バッファ不使用予定）
- **影響度**: 高（segfault, セキュリティ問題）
- **検出容易性**: 中（AddressSanitizer で検出可能）
- **リスクスコア**: (低×高)/中 = **1.0** (高リスク)

**事前対策**:
- 固定長バッファを使わない（動的確保のみ）
- `strncpy`の代わりに`strndup`使用
- 長さチェックを明示的に実装

**検出方法**:
```bash
# AddressSanitizer でビルド
RUSTFLAGS="-Z sanitizer=address" cargo build --target x86_64-unknown-linux-gnu
./target/x86_64-unknown-linux-gnu/debug/hako test.hako
```

**対処方法**:
- 境界チェック追加
- assert()による事前条件チェック

---

#### リスク#3: NULL ポインタ参照
- **内容**: Rust側から渡されたポインタがNULLの場合の未対処
- **発生確率**: 中（Rust Option<T> → C *T 変換で発生可能）
- **影響度**: 高（segfault）
- **検出容易性**: 高（即座にcrash）
- **リスクスコア**: (中×高)/高 = **1.0** (高リスク)

**事前対策**:
```c
// すべてのポインタ引数で NULLチェック
int hako_parse_c(const char *input, size_t len, char **out_json) {
    if (!input || !out_json) return -1;  // NULLチェック
    // ...
}
```

**検出方法**:
- 単体テスト（NULL渡しテスト）
- デバッグビルドで`assert(ptr != NULL)`

**対処方法**:
- エラーコード返却（-1 = invalid argument）
- Rust側でOption<T>を明示的にunwrap

---

#### リスク#4: ABI ミスマッチ（Rust ↔ C）
- **内容**: 構造体レイアウト、呼び出し規約、アライメントの不一致
- **発生確率**: 低（`#[repr(C)]`使用で回避可能）
- **影響度**: 高（segfault、データ破損）
- **検出容易性**: 中（実行時にcrash、原因特定は困難）
- **リスクスコア**: (低×高)/中 = **1.0** (高リスク)

**事前対策**:
```rust
// Rust側で明示的に#[repr(C)]
#[repr(C)]
pub struct ParserResult {
    status: i32,
    json: *mut c_char,  // C側がfree()責任
}
```

**検出方法**:
- 単体テスト（構造体サイズ確認）
```c
// C側
static_assert(sizeof(struct parser_result) == 16, "ABI mismatch");
```

**対処方法**:
- 構造体を最小化（プリミティブ型のみ）
- ポインタ渡しを基本とする

---

#### リスク#5: コンパイラ依存性
- **内容**: gcc vs clang で挙動が異なる
- **発生確率**: 低（最小実装、標準Cのみ使用）
- **影響度**: 中（特定環境でビルド失敗）
- **検出容易性**: 高（ビルド時エラー）
- **リスクスコア**: (低×中)/高 = **0.5** (低リスク)

**事前対策**:
- C99標準のみ使用
- コンパイラ固有拡張を使わない
- `-Wall -Wextra` で警告を有効化

**検出方法**:
```bash
# gcc でビルド
gcc -std=c99 -Wall -Wextra parser_harness.c

# clang でビルド
clang -std=c99 -Wall -Wextra parser_harness.c
```

**対処方法**:
- CI で複数コンパイラテスト
- 標準準拠コードに修正

---

### 1.2 Hako ABI実装リスク

#### リスク#6: C ABI連携失敗
- **内容**: Hakorune側からC ABI呼び出しが失敗
- **発生確率**: 中（FFI呼び出しの典型的問題）
- **影響度**: 高（Selfhost Parserが動作不可）
- **検出容易性**: 高（即座にエラー）
- **リスクスコア**: (中×高)/高 = **1.0** (中リスク)

**事前対策**:
- C ABI層の単体テスト先行実装
- Rust側で動作確認後、Hako実装

**検出方法**:
```bash
# Rust側単体テスト
cargo test --test test_parser_harness

# Hako側テスト
bash tools/smokes/v2/profiles/quick-selfhost/parser_facade_min_vm.sh
```

**対処方法**:
- FFI呼び出しログ追加
- エラーコードを詳細化

---

#### リスク#7: JSON v0 ヘッダ生成の不備
- **内容**: MIR JSON v0フォーマットのヘッダ生成が不正確
- **発生確率**: 中（フォーマット仕様の複雑性）
- **影響度**: 高（VM実行失敗）
- **検出容易性**: 高（VM即座にエラー）
- **リスクスコア**: (中×高)/高 = **1.0** (中リスク)

**事前対策**:
- JSON v0 仕様を明文化（ドキュメント）
- ヘッダ生成ロジックを独立テスト

**検出方法**:
```bash
# JSON v0 バリデーション
jq . output.json  # 正常にparseできるか確認
```

**対処方法**:
- JSON schemaバリデーション
- Rust Parser出力と比較

---

#### リスク#8: 比較ロジックのバグ
- **内容**: Rust Parser出力 vs Hako Parser出力の比較で誤判定
- **発生確率**: 中（比較ロジックの複雑性）
- **影響度**: 中（誤ったPass/Fail判定）
- **検出容易性**: 高（手動確認可能）
- **リスクスコア**: (中×中)/高 = **0.67** (中リスク)

**事前対策**:
- 比較ロジックを独立テスト
- 既知の正解データでテスト

**検出方法**:
```bash
# 手動確認
diff <(./hakorune-rust --emit-mir-json test.hako) \
     <(./hakorune-hako --emit-mir-json test.hako)
```

**対処方法**:
- JSON正規化（キー順序統一）
- 無視すべきフィールドを明確化

---

### 1.3 統合リスク

#### リスク#9: 既存テスト破壊
- **内容**: quick-selfhost 177 PASS → X PASS に減少
- **発生確率**: 中（Phase 4実装中の典型的問題）
- **影響度**: 高（Phase 3完了条件違反）
- **検出容易性**: 高（スモークテストで即検出）
- **リスクスコア**: (中×高)/高 = **1.0** (高リスク)

**事前対策**:
- 各コミット後にquick-selfhost実行（自動化）
- 失敗時即座にRollback

**検出方法**:
```bash
# 各コミット後
bash tools/smokes/v2/run.sh --profile quick-selfhost
```

**対処方法**:
- 失敗したコミットをrevert
- 失敗原因を修正後、再コミット

---

#### リスク#10: パフォーマンス劣化
- **内容**: C ABI経由でのparse がRust直接より遅い
- **発生確率**: 中（FFI呼び出しオーバーヘッド）
- **影響度**: 低（開発時のみ使用、本番は直接呼び出し）
- **検出容易性**: 高（ベンチマーク測定）
- **リスクスコア**: (中×低)/高 = **0.3** (低リスク)

**事前対策**:
- パフォーマンス要件を明確化（Rust版の2倍まで許容）
- feature flag で無効化可能に

**検出方法**:
```bash
# ベンチマーク
time SMOKES_PARSER_MODE=rust cargo build --release
time SMOKES_PARSER_MODE=hako cargo build --release
```

**対処方法**:
- 許容範囲内なら問題なし
- 許容外ならC ABI層を最適化 or 無効化

---

#### リスク#11: ビルドシステム破壊
- **内容**: build.rs修正でビルド失敗
- **発生確率**: 中（build.rs変更の典型的問題）
- **影響度**: 高（全開発停止）
- **検出容易性**: 高（ビルド時エラー）
- **リスクスコア**: (中×高)/高 = **1.0** (中リスク)

**事前対策**:
- build.rs変更前にバックアップ
- 段階的変更（最小単位でコミット）

**検出方法**:
```bash
# ビルドテスト
cargo clean
cargo build --release
```

**対処方法**:
```bash
# 直前のコミットに戻す
git revert HEAD
cargo build --release
```

---

#### リスク#12: CI/CD破壊
- **内容**: GitHub Actions等のCI/CDが失敗
- **発生確率**: 中（環境依存問題）
- **影響度**: 中（ローカル開発は継続可能）
- **検出容易性**: 高（CI即座に失敗）
- **リスクスコア**: (中×中)/高 = **0.67** (中リスク)

**事前対策**:
- CI設定を更新（C compiler追加）
- ローカルで再現可能にする

**検出方法**:
- CI/CDログ確認

**対処方法**:
- CI設定修正
- 必要に応じてC ABI層を無効化

---

### 1.4 運用リスク

#### リスク#13: Rollback困難性
- **内容**: Phase 4実装後、元に戻せない
- **発生確率**: 低（MIR疎結合により回避可能）
- **影響度**: 高（開発停止）
- **検出容易性**: 低（問題が起きてから判明）
- **リスクスコア**: (低×高)/低 = **1.33** (中リスク)

**事前対策**:
- Rollback手順を事前に文書化
- feature flag で無効化機構を実装

**検出方法**:
- Rollback手順を実際に試す（dry-run）

**対処方法**:
- 3レベルRollback戦略を実施（後述）

---

#### リスク#14: デバッグ困難性
- **内容**: C + Rust + Hakorune の3層で問題特定が困難
- **発生確率**: 高（多層アーキテクチャの典型的問題）
- **影響度**: 中（開発速度低下）
- **検出容易性**: 低（原因特定に時間がかかる）
- **リスクスコア**: (高×中)/低 = **2.0** (高リスク)

**事前対策**:
- 各層のログ機構を充実
- 環境変数でトレース有効化

**検出方法**:
```bash
# 3層トレース
NYASH_C_ABI_TRACE=1 \
NYASH_VM_TRACE="op=*" \
HAKO_DEBUG=1 \
./target/release/hako test.hako
```

**対処方法**:
- 層別に問題を切り分け
- 最小再現ケースを作成

---

#### リスク#15: メンテナンス負荷増加
- **内容**: C ABI層の保守が追加負担
- **発生確率**: 中（コードベース増加）
- **影響度**: 低（100-200行のみ）
- **検出容易性**: N/A（長期的問題）
- **リスクスコア**: (中×低)/N/A = **0.5** (低リスク)

**事前対策**:
- C ABI層を最小化（100-200行）
- ドキュメント充実

**対処方法**:
- 定期的なコードレビュー
- 不要になったらfeature flagで無効化

---

### 1.5 プラットフォーム依存リスク

#### リスク#16: Linux vs macOS vs Windows
- **内容**: プラットフォーム間でビルド・実行が異なる
- **発生確率**: 中（C言語の典型的問題）
- **影響度**: 中（一部環境で動作不可）
- **検出容易性**: 中（各環境でテスト必要）
- **リスクスコア**: (中×中)/中 = **1.0** (中リスク)

**事前対策**:
- 標準C99のみ使用
- プラットフォーム固有コードを#ifdefで分離

**検出方法**:
- CI で複数プラットフォームテスト

**対処方法**:
- プラットフォーム別の対応を追加

---

#### リスク#17: アーキテクチャ依存（x86 vs ARM）
- **内容**: x86_64 vs aarch64 でバイナリ互換性なし
- **発生確率**: 低（動的リンクでなければ問題なし）
- **影響度**: 低（再ビルドで対応可能）
- **検出容易性**: 高（ビルド時エラー）
- **リスクスコア**: (低×低)/高 = **0.2** (低リスク)

**事前対策**:
- アーキテクチャ固有コードを使わない

**対処方法**:
- 各アーキテクチャで再ビルド

---

#### リスク#18: ドキュメント不足
- **内容**: C ABI層の使い方が不明確
- **発生確率**: 高（ドキュメント作成忘れ）
- **影響度**: 低（コード読めば理解可能）
- **検出容易性**: 高（レビューで指摘）
- **リスクスコア**: (高×低)/高 = **0.5** (低リスク)

**事前対策**:
- 実装と同時にドキュメント作成
- コード内コメント充実

**対処方法**:
- ドキュメント追加・更新

---

## 2. リスクマトリックス

| リスクID | 内容 | 確率 | 影響 | 検出 | スコア | 優先度 |
|---------|------|------|------|------|--------|--------|
| #2 | バッファオーバーフロー | 低 | 高 | 中 | 1.0 | 高 |
| #3 | NULL参照 | 中 | 高 | 高 | 1.0 | 高 |
| #4 | ABIミスマッチ | 低 | 高 | 中 | 1.0 | 高 |
| #9 | 既存テスト破壊 | 中 | 高 | 高 | 1.0 | 高 |
| #14 | デバッグ困難性 | 高 | 中 | 低 | 2.0 | **最高** |
| #6 | C ABI連携失敗 | 中 | 高 | 高 | 1.0 | 中 |
| #7 | JSON v0ヘッダ不備 | 中 | 高 | 高 | 1.0 | 中 |
| #11 | ビルド破壊 | 中 | 高 | 高 | 1.0 | 中 |
| #16 | プラットフォーム依存 | 中 | 中 | 中 | 1.0 | 中 |
| #1 | メモリリーク | 中 | 中 | 高 | 0.67 | 中 |
| #8 | 比較ロジックバグ | 中 | 中 | 高 | 0.67 | 中 |
| #12 | CI/CD破壊 | 中 | 中 | 高 | 0.67 | 中 |
| #13 | Rollback困難 | 低 | 高 | 低 | 1.33 | 中 |
| #10 | パフォーマンス劣化 | 中 | 低 | 高 | 0.3 | 低 |
| #5 | コンパイラ依存 | 低 | 中 | 高 | 0.5 | 低 |
| #15 | メンテナンス負荷 | 中 | 低 | N/A | 0.5 | 低 |
| #18 | ドキュメント不足 | 高 | 低 | 高 | 0.5 | 低 |
| #17 | アーキテクチャ依存 | 低 | 低 | 高 | 0.2 | 低 |

**最高優先度リスク**: #14 (デバッグ困難性) - スコア 2.0

**対策**: 各層のトレース機構を充実させ、問題の切り分けを容易にする

---

## 3. Rollback戦略

### 3.1 Rollback Level 1: ファイル削除（最速）

**状況**: C ABI層のみ削除したい

```bash
# C ABI層削除（1コマンド）
rm -rf src/parser_harness/
git checkout build.rs

# ビルドテスト
cargo build --release

# 検証
bash tools/smokes/v2/run.sh --profile quick-selfhost
# 期待: 177 PASS（現状維持）
```

**所要時間**: 5分
**リスク**: 低（ファイル削除のみ）
**適用条件**: C ABI層のバグが修正不可能

---

### 3.2 Rollback Level 2: 機能無効化（feature flag）

**状況**: C ABIを一時的に無効化したい

#### 事前準備（Phase 4実装時に実施）

```toml
# Cargo.toml に feature追加
[features]
default = []
parser-c-abi = []  # C ABI有効化フラグ
```

```rust
// build.rs 修正
fn main() {
    #[cfg(feature = "parser-c-abi")]
    {
        // C ABI層のビルド
        cc::Build::new()
            .file("src/parser_harness/parser_harness.c")
            .compile("parser_harness");
    }
}
```

#### 無効化手順

```bash
# C ABI無効化ビルド
cargo build --release

# 有効化ビルド
cargo build --release --features parser-c-abi
```

**所要時間**: 2分
**リスク**: 極低（ビルドフラグのみ）
**適用条件**: 一時的にC ABIをバイパスしたい

---

### 3.3 Rollback Level 3: Full Rollback（Git revert）

**状況**: Phase 4全体を巻き戻す

```bash
# 現在のコミット確認
git log --oneline -5

# Phase 4関連コミット特定
# 例: a0ef3d66 Phase 15.75 foundation work + plugin-on refinements
#     （この直前がPhase 3完了）

# Phase 4全削除（複数コミット）
git revert --no-commit HEAD~3..HEAD  # 直近3コミット取り消し
git commit -m "Rollback: Phase 4全削除（C ABI実装に問題）"

# ビルドテスト
cargo clean
cargo build --release

# quick-selfhost 緑確認
bash tools/smokes/v2/run.sh --profile quick-selfhost
# 期待: Passed: 177 (Phase 3完了時の状態)
```

**所要時間**: 10-30分（ビルド時間含む）
**リスク**: 低（Gitで管理されている）
**適用条件**: Phase 4全体が失敗、修正不可能

---

### 3.4 Rollback判断基準

以下のいずれかに該当する場合、**即座にRollback実施**:

- [ ] quick-selfhost PASS数が **170未満**に低下
- [ ] ビルド失敗が **3回連続**
- [ ] segfault が **1回でも発生**（原因特定困難の場合）
- [ ] メモリリークが **10KB以上**
- [ ] ビルド時間が **20%以上増加**（開発効率低下）
- [ ] 修正に **2日以上**かかる見込み（費用対効果不良）

---

## 4. テスト戦略

### 4.1 単体テスト

#### C ABI層テスト

```bash
# メモリリークテスト
valgrind --leak-check=full --show-leak-kinds=all \
  ./target/release/hako apps/selfhost-compiler/parser/parser.hako 2>&1 | \
  tee valgrind.log

# 期待: "All heap blocks were freed -- no leaks are possible"

# ABIテスト（C側）
cd src/parser_harness
gcc -o test_c_abi test_c_abi.c parser_harness.c
./test_c_abi
# 期待: "All tests passed"
```

#### Hako ABI層テスト

```bash
# 基本テスト
bash tools/smokes/v2/profiles/quick-selfhost/parser_facade_min_vm.sh
# 期待: PASS

# JSON v0 ヘッダ検証
./target/release/hako --emit-mir-json test.json test.hako
jq '.format_version' test.json
# 期待: "v0"
```

---

### 4.2 統合テスト

#### Rust Parser vs Hako Parser 比較

```bash
# SMOKES_PARSER_MODE=rust（Rust Parser使用）
SMOKES_PARSER_MODE=rust \
  bash tools/smokes/v2/profiles/quick-selfhost/parser_facade_min_vm.sh
# 期待: PASS

# SMOKES_PARSER_MODE=hako（Hako Parser使用）
SMOKES_PARSER_MODE=hako \
  bash tools/smokes/v2/profiles/quick-selfhost/parser_facade_min_vm.sh
# 期待: PASS

# SMOKES_PARSER_MODE=both（比較モード）
SMOKES_PARSER_MODE=both \
  bash tools/smokes/v2/profiles/quick-selfhost/parser_facade_min_vm.sh
# 期待: "Rust output == Hako output"
```

---

### 4.3 回帰テスト

```bash
# quick-selfhost 全実行（177 PASS維持確認）
bash tools/smokes/v2/run.sh --profile quick-selfhost
# 期待: Passed: 177, Failed: 33

# 失敗数増加は許容しない（Phase 3完了条件）
# 177→170以下に減少したら即Rollback
```

---

### 4.4 パフォーマンステスト

```bash
# 既定Rust速度への影響を測定
time SMOKES_PARSER_MODE=rust cargo build --release
# 例: 2.5秒

time SMOKES_PARSER_MODE=hako cargo build --release
# 許容: 5.0秒以内（2倍まで）
# 警告: 5.0-10.0秒（2-4倍）
# NG: 10.0秒超（4倍以上）
```

---

## 5. モニタリング指標

### 5.1 継続監視指標

| 指標 | 目標 | 警告 | 危険 | 測定方法 | 頻度 |
|------|------|------|------|---------|------|
| quick-selfhost PASS数 | 177 | <172 | <170 | スモークテスト | 毎コミット |
| ビルド成功率 | 100% | <95% | <90% | cargo build | 毎コミット |
| ビルド時間 | ±0% | +10% | +20% | time cargo build | 毎日 |
| メモリリーク | 0KB | >1KB | >10KB | valgrind | 週1回 |
| segfault数 | 0 | 1 | 2+ | dmesg | 即座 |
| C ABI層行数 | 100-200 | 250 | 300+ | wc -l | 毎コミット |

### 5.2 Phase 4完了判定

以下の条件を**すべて**満たすこと:

- [ ] **quick-selfhost 177 PASS 維持**（最重要）
- [ ] **SMOKES_PARSER_MODE=both 成功**（Rust == Hako 確認）
- [ ] **ビルド時間増加 10% 以内**
- [ ] **メモリリーク 0KB**
- [ ] **valgrind エラー 0件**
- [ ] **segfault 0件**
- [ ] **C ABI層 100-200行**（行数制限）
- [ ] **ドキュメント更新完了**（README, 使い方）
- [ ] **feature flag実装**（無効化機構）

---

## 6. 推奨事項

### 高優先度（実装前に対処）P0

1. **feature flag 実装** ⭐最重要
   - C ABIを無効化できる機構
   - デフォルトOFF（opt-in）
   - 理由: Rollback Level 2を可能にする

2. **C ABI層を最小実装（100-200行）に限定** ⭐最重要
   - メモリ管理を単純化
   - 所有権移譲なし（参照渡しのみ）
   - 理由: リスク#1,#2,#3の発生確率を低下

3. **各コミット後にquick-selfhost実行** ⭐最重要
   - GitHub Actions等で自動化
   - 失敗時は即座にRollback
   - 理由: リスク#9の早期検出

4. **トレース機構の充実** ⭐最重要
   - C ABI層: `NYASH_C_ABI_TRACE=1`
   - Hako層: `HAKO_DEBUG=1`
   - Rust VM層: `HAKO_VM_TRACE="op=*"`
   - 理由: リスク#14（デバッグ困難性）の軽減

---

### 中優先度（実装中に対処）P1

5. **valgrind を毎回実行**
   - メモリリーク検出
   - 週1回定期実行
   - 理由: リスク#1の早期検出

6. **コミット粒度を細かく**
   - 1ファイル修正 → 1コミット
   - Rollback容易化
   - 理由: Rollback Level 3の効率化

7. **NULL チェックを徹底**
   - すべてのポインタ引数でチェック
   - assert()による事前条件確認
   - 理由: リスク#3の回避

8. **#[repr(C)] を明示**
   - Rust側構造体すべてに付与
   - サイズ確認テスト追加
   - 理由: リスク#4の回避

---

### 低優先度（実装後に対処）P2

9. **パフォーマンスベンチマーク**
   - 週1回測定
   - 推移をグラフ化
   - 理由: リスク#10の監視

10. **クロスプラットフォーム検証**
    - CI で Linux/macOS/Windows テスト
    - 理由: リスク#16の検出

11. **ドキュメント充実**
    - C ABI使い方
    - トラブルシューティング
    - 理由: リスク#18の軽減

---

## 7. 実装手順（リスク軽減版）

### Phase 4.1: 基盤実装（Week 1-2）

**目標**: C ABI層最小実装

```bash
# Step 1: feature flag 実装（1日）
# - Cargo.toml に parser-c-abi 追加
# - build.rs 修正

# Step 2: C ABI層実装（2-3日）
# - parser_harness.c（100-200行）
# - NULLチェック、メモリ管理明確化

# Step 3: 単体テスト（1日）
# - test_c_abi.c 作成
# - valgrind 実行

# Step 4: Rust連携（1日）
# - src/parser_harness/mod.rs 実装
# - #[repr(C)] 構造体定義

# 検証: quick-selfhost 177 PASS維持
bash tools/smokes/v2/run.sh --profile quick-selfhost
```

**判断**: PASS数が170未満に低下したら即Rollback

---

### Phase 4.2: Hako ABI実装（Week 3-4）

**目標**: Hakorune側からC ABI呼び出し

```bash
# Step 5: Hako ABI層実装（3-4日）
# - selfhost/compiler/parser_harness_box.hako 作成
# - C ABI呼び出しロジック

# Step 6: JSON v0 ヘッダ生成（2日）
# - フォーマット仕様明文化
# - 生成ロジック実装

# Step 7: 比較ロジック実装（2日）
# - Rust vs Hako 出力比較
# - SMOKES_PARSER_MODE=both 実装

# Step 8: 統合テスト（1日）
# - parser_facade_min_vm.sh 実行
# - SMOKES_PARSER_MODE=both 成功確認

# 検証: quick-selfhost 177 PASS維持
bash tools/smokes/v2/run.sh --profile quick-selfhost
```

**判断**: 統合テスト失敗 → Rollback Level 2（C ABI無効化）

---

### Phase 4.3: 最終検証（Week 5）

**目標**: 完了条件すべて満たす

```bash
# Step 9: 最終検証（全項目）
- [ ] quick-selfhost 177 PASS 維持
- [ ] SMOKES_PARSER_MODE=both 成功
- [ ] ビルド時間増加 10% 以内
- [ ] メモリリーク 0KB
- [ ] valgrind エラー 0件
- [ ] segfault 0件
- [ ] C ABI層 100-200行
- [ ] ドキュメント更新完了
- [ ] feature flag実装

# すべてOK → Phase 4完了
# 1つでもNG → 原因修正 or Rollback
```

---

## 8. 緊急対応フローチャート

```
問題発生
    ↓
┌───▼────────────────────┐
│ 重大度判定              │
└───┬────────────────────┘
    │
    ├─ 軽微（テスト1-2件失敗）
    │   → 修正試行（1日以内）
    │   → 修正成功 → 継続
    │   → 修正失敗 → Rollback Level 1
    │
    ├─ 中程度（テスト5-10件失敗、ビルド失敗）
    │   → Rollback Level 2（C ABI無効化）
    │   → 原因調査（2日以内）
    │   → 修正 → 再実装
    │
    └─ 重大（segfault、quick-selfhost <170）
        → **即座にRollback Level 3**
        → Phase 4中止判断
        → ユーザー報告
```

---

## 9. 結論とNext Steps

### Phase 4 の実行可否判断

⚠️ **条件付き実行推奨**

**実行条件**（すべて満たす必要あり）:
1. ✅ feature flag実装（無効化機構）
2. ✅ C ABI層を100-200行に限定
3. ✅ quick-selfhost毎回実行（自動化）
4. ✅ トレース機構充実（3層デバッグ可能）
5. ✅ Rollback手順を事前確認（dry-run）

**実行条件を満たさない場合**:
- ❌ Phase 4を延期
- ✅ Phase 5（他の削減タスク）を先行実施

---

### 最終推奨

**Phase 4は実行可能** ✅

**理由**:
1. MIR疎結合により致命的リスクは低い
2. C ABI層を100-200行に限定すればリスクは管理可能
3. Rollback戦略は明確（3レベル、最速5分）
4. 継続的モニタリングで早期検出可能

**ただし、以下の対策が必須** ⚠️:
- feature flag実装（デフォルトOFF）
- 各コミット後quick-selfhost実行
- トレース機構充実
- Rollback手順の事前確認

---

## 10. 関連ドキュメント

- [Phase 15.75 INDEX](./INDEX.md) - エントリーポイント
- [STRATEGY.md](./STRATEGY.md) - Bootstrap戦略とツールチェーン
- [ROADMAP.md](./ROADMAP.md) - 全体計画とPhase別タスク
- [TODO.md](./TODO.md) - 次のアクション
- [TEST_COMPLEXITY_REPORT.md](../../analysis/TEST_COMPLEXITY_REPORT.md) - テスト複雑度分析

---

## 📝 変更履歴

- 2025-10-14: 初版作成（Claude Code）
  - 18個のリスクを分析
  - 3レベルRollback戦略を策定
  - モニタリング指標と完了判定条件を定義
  - 実装手順（リスク軽減版）を提案

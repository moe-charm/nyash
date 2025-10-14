# Phase 15.76 Guard Rails - 安全性設計

## 🔒 セキュリティ原則

### 基本方針
1. **Deny by Default** - すべてのextern_cはデフォルト拒否
2. **Explicit Allowlist** - 明示的に許可したもののみ実行
3. **Fail-Fast** - 不正な呼び出しは即座にエラー
4. **Audit Trail** - 許可リストは1ファイルで管理

---

## 🛡️ 許可モデル（4層防御）

### Layer 1: 既定ホワイトリスト（最小セット）

**場所**: `src/runtime/ffi/default_allowlist.rs`

```rust
pub const DEFAULT_ALLOWLIST: &[&str] = &[
    "getpid",   // プロセスID取得（安全）
    "strlen",   // 文字列長取得（安全）
    "system",   // コマンド実行（制御必要）
];
```

**選定基準**:
- ✅ 読み取り専用または副作用が限定的
- ✅ デバッグ・診断に有用
- ✅ 一般的なC標準関数

**除外基準**:
- ❌ ファイル書き込み（`fopen`, `fwrite`）
- ❌ ネットワーク（`socket`, `bind`）
- ❌ プロセス生成（`fork`, `exec`）
- ❌ メモリ操作（`malloc`, `free`）

### Layer 2: TOML設定（プロジェクト固有）

**場所**: `hako.toml` または `nyash.toml`

```toml
# プロジェクトで必要な追加関数
[ffi.dynamic]
allow = [
    "getppid",     # 親プロセスID
    "getuid",      # ユーザーID
    "gethostname", # ホスト名取得
]
```

**運用方針**:
- プロジェクトに必要な関数のみ追加
- コミット時にレビュー必須
- 理由をコメントで明記

### Layer 3: ENV変数（一時的追加）

**場所**: 環境変数 `HAKO_FFI_ALLOW_LIST`

```bash
# 一時的に getppid を許可
HAKO_FFI_ALLOW_LIST=getppid ./hako program.hako

# 複数関数を許可（カンマ区切り）
HAKO_FFI_ALLOW_LIST=getppid,getuid,gethostname ./hako program.hako
```

**用途**:
- 開発中の試行錯誤
- CI/CDでの特定テスト
- デバッグ用途

**制限**:
- TOMLとマージ（上書きではない）
- 本番環境では使用禁止

### Layer 4: Dev許可（開発専用）⚠️

**場所**: 環境変数 `HAKO_FFI_ALLOW_ALL=1`

```bash
# 全関数を許可（開発専用）
HAKO_FFI_ALLOW_ALL=1 ./hako program.hako
```

**⚠️ 重大な制限**:
- **CI不可**: CIでこのフラグが立っていたらエラー
- **本番不可**: 本番環境では絶対に使用禁止
- **監査必須**: 使用時はログに記録

**検出方法**:
```bash
# CI/CDで検出
if [ "$HAKO_FFI_ALLOW_ALL" = "1" ]; then
    echo "❌ ERROR: HAKO_FFI_ALLOW_ALL=1 is not allowed in CI"
    exit 1
fi
```

---

## 🎯 許可の優先順位

```
高 ↑ 1. 既定ホワイトリスト（DEFAULT_ALLOWLIST）
   | 2. TOML設定（hako.toml, nyash.toml）
   | 3. ENV変数（HAKO_FFI_ALLOW_LIST）
低 ↓ 4. Dev許可（HAKO_FFI_ALLOW_ALL=1）← CI不可

※ 上層で許可されていれば、下層の設定は不要
※ マージ動作：すべて「追加」（上書きではない）
```

---

## 🔍 実行時チェック

### 呼び出し時の検証フロー

```rust
// src/backend/mir_interpreter/handlers/extern_c.rs

pub fn execute_extern_c(
    symbol: &str,
    args: Vec<Value>,
    allowlist: &AllowList,
) -> Result<Value, RuntimeError> {
    // Step 1: 許可チェック
    if !allowlist.is_allowed(symbol) {
        return Err(RuntimeError::ExternCDenied {
            symbol: symbol.to_string(),
            reason: "not in allowlist".to_string(),
        });
    }

    // Step 2: 引数型チェック
    validate_args(symbol, &args)?;

    // Step 3: 動的ロード
    let func = load_symbol(symbol)?;

    // Step 4: 実行（Fail-Fast）
    call_extern_function(func, args)
}
```

### エラーメッセージ例

```
❌ ERROR: ExternCDenied
Symbol: my_custom_func
Reason: not in allowlist

To allow this function:
1. Add to hako.toml: [ffi.dynamic] allow = ["my_custom_func"]
2. Use ENV: HAKO_FFI_ALLOW_LIST=my_custom_func
3. Dev only: HAKO_FFI_ALLOW_ALL=1 (⚠️ CI not allowed)
```

---

## 🧪 テストケース

### ✅ 許可された関数（PASS）

```bash
# 既定関数
./hako -c 'extern_c "getpid"()'  # ✅ PASS

# TOML許可
# hako.toml: [ffi.dynamic] allow = ["getppid"]
./hako -c 'extern_c "getppid"()'  # ✅ PASS

# ENV許可
HAKO_FFI_ALLOW_LIST=getuid ./hako -c 'extern_c "getuid"()'  # ✅ PASS
```

### ❌ 拒否された関数（FAIL）

```bash
# 許可されていない関数
./hako -c 'extern_c "fork"()'  # ❌ FAIL: ExternCDenied

# 出力例:
# ERROR: ExternCDenied
# Symbol: fork
# Reason: not in allowlist
```

### 🔧 Dev許可（開発専用）

```bash
# 開発時のみ全許可
HAKO_FFI_ALLOW_ALL=1 ./hako -c 'extern_c "fork"()'  # ✅ PASS (dev only)

# CIでは拒否
# CI環境でHAKO_FFI_ALLOW_ALL=1が設定されていたらエラー
```

---

## 📋 監査ガイドライン

### プロジェクト設定の監査

```bash
# hako.toml の許可リストを確認
grep -A 10 '\[ffi.dynamic\]' hako.toml

# チェックポイント:
# ✅ 各関数の用途が明確か？
# ✅ 最小権限原則が守られているか？
# ✅ 危険な関数（fork/exec/socket等）が含まれていないか？
```

### 実行時ログ（将来実装）

```bash
# extern_c呼び出しをログに記録
HAKO_FFI_LOG=1 ./hako program.hako

# 出力例:
# [FFI] Called: getpid() → 12345
# [FFI] Called: strlen("hello") → 5
# [FFI] Denied: fork() (not in allowlist)
```

---

## 🚨 危険な関数リスト（参考）

### 絶対に許可すべきでない関数

```c
// プロセス制御
fork()      // プロセス複製
exec*()     // プログラム実行
system()    // ⚠️ 既定に含まれるが制御必要

// メモリ操作
malloc()    // メモリ確保
free()      // メモリ解放
realloc()   // メモリ再確保

// ファイル書き込み
fopen()     // ファイルオープン
fwrite()    // ファイル書き込み
remove()    // ファイル削除

// ネットワーク
socket()    // ソケット生成
bind()      // ポートバインド
listen()    // 接続待ち受け
```

### 慎重に許可すべき関数

```c
// 読み取り専用（比較的安全）
getpid()    // ✅ プロセスID
getuid()    // ✅ ユーザーID
strlen()    // ✅ 文字列長

// 副作用あり（用途確認必要）
system()    // ⚠️ コマンド実行
getenv()    // ⚠️ 環境変数取得
setenv()    // ❌ 環境変数設定
```

---

## 🔐 ライブラリ探索のセキュリティ

### 探索パスの優先順位

```
高 ↑ 1. target/release/          （ビルド成果物）
   | 2. $NYASH_ROOT/target/release/（プロジェクトルート）
   | 3. カレントディレクトリ       （実行場所）
低 ↓ 4. $HAKO_FFI_LIB_PATHS       （ENV指定）
```

### セキュリティ制約

```bash
# ✅ 安全: プロジェクト内のライブラリ
./target/release/libllvm_backend.so

# ⚠️ 注意: システムライブラリ
/usr/lib/libcustom.so

# ❌ 危険: 任意のパス
/tmp/malicious.so  # 探索パスに含めない
```

### 実装

```rust
pub fn find_library(name: &str) -> Option<PathBuf> {
    // Step 1: ホワイトリスト確認
    if !is_allowed_library(name) {
        return None;
    }

    // Step 2: 安全なパスのみ探索
    let safe_paths = [
        "target/release",
        &format!("{}/target/release", env!("NYASH_ROOT")),
        ".",
    ];

    for path in safe_paths {
        let lib_path = Path::new(path).join(name);
        if lib_path.exists() {
            return Some(lib_path);
        }
    }

    None
}
```

---

## 📊 セキュリティチェックリスト

### 開発時
- [ ] 新しいextern_c呼び出しはTOMLに追加したか？
- [ ] 各関数の用途を明確に説明できるか？
- [ ] 最小権限原則が守られているか？

### コミット前
- [ ] hako.toml の [ffi.dynamic] をレビューしたか？
- [ ] 危険な関数（fork/exec等）が含まれていないか？
- [ ] テストで許可機構の動作を確認したか？

### リリース前
- [ ] 本番環境で HAKO_FFI_ALLOW_ALL=1 が使われていないか？
- [ ] CIでセキュリティチェックが通っているか？
- [ ] 監査ログを確認したか？

---

## 🎯 推奨運用方針

### ステージング別設定

```toml
# hako.dev.toml（開発環境）
[ffi.dynamic]
allow = [
    "getpid", "strlen", "system",  # 既定
    "getppid", "getuid",            # デバッグ用
]

# hako.prod.toml（本番環境）
[ffi.dynamic]
allow = [
    "getpid", "strlen",  # 最小限
]
```

### CI/CD設定

```yaml
# .github/workflows/test.yml
- name: Security Check
  run: |
    # HAKO_FFI_ALLOW_ALL=1 の検出
    if [ "$HAKO_FFI_ALLOW_ALL" = "1" ]; then
      echo "❌ ERROR: HAKO_FFI_ALLOW_ALL=1 not allowed in CI"
      exit 1
    fi

    # 危険な関数の検出
    if grep -r "fork\|exec\|socket" hako.toml; then
      echo "⚠️ WARNING: Dangerous functions found in hako.toml"
    fi
```

---

**作成日**: 2025-10-14
**レビュー**: セキュリティ設計確定
**関連**: Phase 15.76, extern_c許可モデル, セキュリティ

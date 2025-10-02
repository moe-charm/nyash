# hako.toml メタ設定例

**作成**: 2025-10-02
**ソース**: ChatGPT Pro UltraThink Mode
**用途**: Python統合のメタデータ設定リファレンス

---

## 📋 概要

Python-Hakorune統合における`hako.toml`の設定例集です。
Effect/Capability/Contract/Policyの具体的な設定方法を示します。

---

## 🎯 基本設定

### 最小構成

```toml
# hako.toml (minimal)
[box.PyRuntimeBox]
effects.allow = ["py.import:math"]
```

### 標準構成

```toml
# hako.toml (standard)
[box.PyRuntimeBox]
effects.allow = [
    "py.import:math",
    "py.import:json",
    "py.time:monotonic"
]
capabilities.enforce = true
```

---

## 🎭 Effect設定

### 利用可能なEffect一覧

```toml
[box.PyRuntimeBox.effects]
available = [
    # モジュールインポート
    "py.import",

    # ファイルシステム
    "py.fs.read",
    "py.fs.write",

    # ネットワーク
    "py.net.http",
    "py.net.socket",

    # 環境変数
    "py.env.read",
    "py.env.write",

    # 時刻
    "py.time.monotonic",
    "py.time.real",

    # ランダム
    "py.random",

    # プロセス
    "py.subprocess"
]
```

### Effect許可例

```toml
[box.PyRuntimeBox.effects]
allow = [
    "py.import:math",
    "py.import:json",
    "py.import:re",
    "py.time:monotonic",
    "py.fs.read:/tmp/**"
]
```

---

## 🛡️ Capability設定

### 基本Capability

```toml
[box.PyRuntimeBox.capabilities]
# 許可リスト
allow = [
    "py.import:math",
    "py.import:json"
]

# 拒否リスト
deny = [
    "py.subprocess",
    "py.net"
]

# 厳格モード
enforce = true
```

### 詳細Capability（モジュール別）

```toml
[box.PyRuntimeBox.capabilities.import]
# 標準ライブラリ
stdlib = [
    "math",
    "json",
    "re",
    "datetime",
    "collections"
]

# サードパーティ（ホワイトリスト）
allow_third_party = [
    "numpy",
    "pandas"
]

# 拒否（ブラックリスト）
deny = [
    "os",
    "subprocess",
    "socket"
]
```

### ファイルシステムCapability

```toml
[box.PyRuntimeBox.capabilities.fs]
# 読み取り許可
read_paths = [
    "/tmp/**",
    "/data/input/**",
    "~/.config/app/**"
]

# 書き込み許可
write_paths = [
    "/tmp/**",
    "/data/output/**"
]

# 拒否
deny_paths = [
    "/etc/**",
    "/sys/**",
    "~/.ssh/**"
]
```

### ネットワークCapability

```toml
[box.PyRuntimeBox.capabilities.net]
# HTTP許可
http_allow = [
    "https://api.example.com/**",
    "https://data.example.org/api/**"
]

# Socket許可（ホスト:ポート）
socket_allow = [
    "localhost:8080",
    "127.0.0.1:5432"
]

# 拒否
deny = [
    "0.0.0.0:*",  # すべてのインターフェース
    "*:22",       # SSH
    "*:3389"      # RDP
]
```

---

## ✅ Contract設定

### PyFunctionBox Contract

```toml
[contracts.PyFunctionBox.exec]
# Pre条件（引数チェック）
pre = [
    "args.len <= 8",              # 引数は8個まで
    "bytes_total <= 1_000_000",   # 合計1MB以下
    "no_file_descriptors"         # ファイルディスクリプタ禁止
]

# Post条件（返り値チェック）
post = [
    "result.size <= 1_000_000",   # 返り値1MB以下
    "no_exception",               # 例外禁止
    "allow_none = true",          # None許可
    "execution_time <= 5.0"       # 5秒以内
]

# 違反時の挙動
on_violation = "error"  # "error" | "warn" | "ignore"
```

### PyModuleBox Contract

```toml
[contracts.PyModuleBox.import]
pre = [
    "module_name.len <= 100",     # モジュール名100文字以内
    "no_relative_import"          # 相対インポート禁止
]

post = [
    "module_size <= 10_000_000",  # モジュールサイズ10MB以下
    "no_native_code"              # ネイティブコード禁止（オプション）
]
```

### PyInstanceBox Contract

```toml
[contracts.PyInstanceBox.call_method]
pre = [
    "method_name.len <= 50",      # メソッド名50文字以内
    "args.len <= 16"              # 引数16個まで
]

post = [
    "result.size <= 1_000_000",   # 返り値1MB以下
    "no_side_effects"             # 副作用禁止（Pure）
]
```

---

## 🎛️ Policy設定

### 開発モード（Dev）

```toml
[policy.dev]
# Capability
capabilities.enforce = false  # 観測のみ
capabilities.log = true       # ログ記録

# Contract
contracts.enforce = true      # 厳格チェック
contracts.on_violation = "error"  # 違反時エラー

# Deterministic
deterministic = false         # 非決定的許可

# Trace
trace.effects = true          # 効果トレース
trace.calls = true            # 呼び出しトレース
trace.errors = true           # エラートレース

# Verifier
verify.phi = true             # PHI検証
verify.ssa = true             # SSA検証
verify.on_fail = "error"      # 検証失敗時エラー
```

### 本番モード（Prod）

```toml
[policy.prod]
# Capability
capabilities.enforce = true   # 厳格適用
capabilities.log = false      # ログなし

# Contract
contracts.enforce = true      # 厳格チェック
contracts.on_violation = "warn"  # 違反時警告

# Deterministic
deterministic = true          # 決定的実行

# Trace
trace.effects = false         # トレースなし
trace.calls = false
trace.errors = true           # エラーのみ

# Verifier
verify.phi = true             # PHI検証
verify.ssa = true             # SSA検証
verify.on_fail = "warn"       # 検証失敗時警告
```

### テストモード（Test）

```toml
[policy.test]
# Capability
capabilities.enforce = true   # 厳格適用
capabilities.log = true       # ログ記録

# Contract
contracts.enforce = true      # 厳格チェック
contracts.on_violation = "error"  # 違反時エラー

# Deterministic
deterministic = true          # 決定的実行（再現性）

# Trace
trace.effects = true          # 全トレース
trace.calls = true
trace.errors = true

# Verifier
verify.phi = true             # 全検証
verify.ssa = true
verify.on_fail = "error"      # 検証失敗時エラー
```

---

## 🔧 環境変数によるオーバーライド

### 優先順位

1. 環境変数（最優先）
2. hako.toml
3. hakorune.toml
4. デフォルト値

### 環境変数例

```bash
# Capability
export HAKO_PLUGIN_CAPS_ENFORCE=1
export HAKO_TRACE_EFFECTS=1

# Contract
export HAKO_CHECK_CONTRACTS=1
export HAKO_CONTRACT_VIOLATION=error

# Deterministic
export HAKO_DETERMINISTIC=1

# ABI
export HAKO_ABI_VTABLE=1
export HAKO_ABI_STRICT=1

# Verifier
export HAKO_VERIFY_ALLOW_NO_PHI=0
export HAKO_VERIFY_ON_FAIL=error

# LLVM
export HAKO_LLVM_USE_HARNESS=1
```

---

## 📊 実用例

### 例1: 数学計算のみ許可

```toml
# hako.toml
[box.PyRuntimeBox]
effects.allow = ["py.import:math"]
capabilities.enforce = true

[contracts.PyFunctionBox.exec]
pre = ["args.len <= 4"]
post = ["no_exception", "execution_time <= 1.0"]
```

```hakorune
using "python"

box Main {
    flow main() {
        let py = PyRuntimeBox();
        let math = py.import("math");
        let sqrt = math.get("sqrt");
        let r = sqrt.exec([2.0]);
        print(r);  // OK
        py.fini();
    }
}
```

### 例2: データ処理（ファイル読み取り）

```toml
# hako.toml
[box.PyRuntimeBox]
effects.allow = [
    "py.import:json",
    "py.fs.read:/data/input/**"
]

[box.PyRuntimeBox.capabilities.fs]
read_paths = ["/data/input/**"]
write_paths = []  # 書き込み禁止
```

```hakorune
using "python"

box Main {
    flow main() {
        let py = PyRuntimeBox();
        let json = py.import("json");

        // ファイル読み取り（許可）
        let data = py.eval("open('/data/input/test.json').read()");
        let parsed = json.get("loads").exec([data]);

        print(parsed);
        py.fini();
    }
}
```

### 例3: API呼び出し（ネットワーク）

```toml
# hako.toml
[box.PyRuntimeBox]
effects.allow = [
    "py.import:urllib",
    "py.net.http"
]

[box.PyRuntimeBox.capabilities.net]
http_allow = [
    "https://api.example.com/**"
]

[contracts.PyFunctionBox.exec]
pre = ["args.len <= 2"]
post = [
    "result.size <= 1_000_000",
    "execution_time <= 10.0"
]
```

```hakorune
using "python"

box Main {
    flow main() {
        let py = PyRuntimeBox();

        let code = "
import urllib.request
def fetch(url):
    with urllib.request.urlopen(url) as r:
        return r.read()
";
        py.exec(code);
        let fetch = py.get("fetch");

        // API呼び出し（許可）
        let result = fetch.exec(["https://api.example.com/data"]);
        print(result);

        py.fini();
    }
}
```

---

## 🔍 デバッグ設定

### 詳細ログ

```toml
[debug]
# すべてのトレース有効化
trace.all = true

# 個別トレース
trace.effects = true
trace.capabilities = true
trace.contracts = true
trace.python_calls = true
trace.gil = true

# ログレベル
log_level = "debug"

# ログ出力先
log_file = "/tmp/hakorune-python-debug.log"
```

### パフォーマンスプロファイル

```toml
[profile]
# タイミング記録
timing.enabled = true
timing.threshold = 0.1  # 100ms以上

# メモリ使用量
memory.track = true
memory.threshold = 10_000_000  # 10MB以上

# GC統計
gc.enabled = true
```

---

## 🔗 関連ドキュメント

- [強化版アーキテクチャv2](enhanced-architecture-v2.md) - 設計詳細
- [マイルストーン](../planning/milestones.md) - 実装計画
- [リスクと対策](risks-and-mitigations.md) - リスク管理
- [Phase 20 README](../README.md) - 全体概要

---

**最終更新**: 2025-10-02
**作成者**: ChatGPT Pro (UltraThink Mode)
**ステータス**: 設定リファレンス

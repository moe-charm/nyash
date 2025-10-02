# Python統合マイルストーン（M0〜M6）

**作成**: 2025-10-02
**ソース**: ChatGPT Pro UltraThink Mode
**ステータス**: 計画・Phase 15完了後に実装

---

## 📋 概要

Python-Hakorune統合を**段階的に**進めるためのマイルストーン定義です。
最小から始めて、機能を段階的に追加していきます。

---

## 🎯 M0: PyRuntimeBox の最小化

### 目的
`import("math")/call("sqrt")` が走る最小実装

### 成果物
- [ ] PyRuntimeBoxの基本実装
- [ ] PyFunctionBoxの基本実装
- [ ] math.sqrtの実行確認

### 実装内容

#### PyRuntimeBox（最小版）
```rust
pub struct PyRuntimeBox {
    py: GILGuard,
    modules: HashMap<String, Py<PyModule>>,
}

impl PyRuntimeBox {
    pub fn init() -> Self {
        pyo3::prepare_freethreaded_python();
        let gil = Python::acquire_gil();
        PyRuntimeBox {
            py: gil,
            modules: HashMap::new(),
        }
    }

    pub fn import(&mut self, name: &str) -> PyModuleBox {
        let py = self.py.python();
        let module = PyModule::import(py, name).unwrap();
        self.modules.insert(name.to_string(), module.into());
        PyModuleBox::new(module)
    }
}
```

#### テストケース
```hakorune
using "python"

box Main {
    flow main() {
        let py = PyRuntimeBox();
        let math = py.import("math");
        let sqrt = math.get("sqrt");
        let result = sqrt.exec([2.0]);
        assert(result.to_float() == 1.4142135623730951);
        py.fini();
    }
}
```

### Capability/Contract
- Capability/Contractは**ログのみ**（観測）
- `HAKO_TRACE_EFFECTS=1` で呼び出しログ記録

### DoD（完了定義）
- ✅ math.sqrt(2.0)が正常実行
- ✅ 結果が正しい値を返す
- ✅ メモリリークなし
- ✅ スモークテスト追加

### 期間
**1週間**

---

## 🛡️ M1: Capability ゲート

### 目的
**ホワイトリスト**運用開始

### 成果物
- [ ] Capability管理機構
- [ ] hako.toml設定サポート
- [ ] 違反時のエラーハンドリング

### 実装内容

#### hako.toml設定
```toml
[box.PyRuntimeBox.capabilities]
allow = [
    "py.import:math",
    "py.import:json",
    "py.time:monotonic"
]
deny = [
    "py.subprocess",
    "py.net"
]
enforce = true
```

#### Capability検証
```rust
impl PyRuntimeBox {
    pub fn import(&mut self, name: &str) -> Result<PyModuleBox> {
        // Capability確認
        if !self.capabilities.allows_import(name) {
            return Err(CapabilityError::ImportDenied(name.to_string()));
        }

        // 通常のimport処理
        // ...
    }
}
```

#### テストケース
```hakorune
using "python"

box Main {
    flow main() {
        let py = PyRuntimeBox();

        // OK: 許可されたモジュール
        let math = py.import("math");

        // ERROR: 拒否されたモジュール
        try {
            let subprocess = py.import("subprocess");
        } catch(e) {
            assert(e.type == "CapabilityError");
        }

        py.fini();
    }
}
```

### トグル
```bash
# Dev: 違反時fail-fast
HAKO_PLUGIN_CAPS_ENFORCE=1

# Prod: 違反時警告（継続）
HAKO_PLUGIN_CAPS_ENFORCE=0
```

### DoD（完了定義）
- ✅ allow/denyリスト動作確認
- ✅ 違反時の適切なエラー
- ✅ hako.toml読み込み動作
- ✅ トグル切り替え動作

### 期間
**1-2週間**

---

## ✅ M2: Contract 検証

### 目的
`pre/post` 条件検証の実装

### 成果物
- [ ] Contract検証機構
- [ ] pre条件検証（引数チェック）
- [ ] post条件検証（返り値チェック）

### 実装内容

#### Contract定義
```toml
[contracts.PyFunctionBox.exec]
pre = [
    "args.len <= 8",
    "bytes_total <= 1_000_000"
]
post = [
    "result.size <= 1_000_000",
    "no_exception",
    "allow_none = true"
]
```

#### Contract検証（Python側wrapper）
```python
# hakorune_contract.py
def verify_contract(func, pre_conditions, post_conditions):
    def wrapper(*args, **kwargs):
        # Pre条件検証
        if len(args) > 8:
            raise ContractViolation("args.len > 8")

        total_bytes = sum(sys.getsizeof(a) for a in args)
        if total_bytes > 1_000_000:
            raise ContractViolation("bytes_total > 1_000_000")

        # 実行
        result = func(*args, **kwargs)

        # Post条件検証
        if sys.getsizeof(result) > 1_000_000:
            raise ContractViolation("result.size > 1_000_000")

        return result

    return wrapper
```

#### テストケース
```hakorune
using "python"

box Main {
    flow main() {
        let py = PyRuntimeBox();

        // OK: Contract満たす
        let math = py.import("math");
        let sqrt = math.get("sqrt");
        let r1 = sqrt.exec([2.0]);

        // ERROR: 引数多すぎ
        try {
            let r2 = sqrt.exec([1,2,3,4,5,6,7,8,9]);
        } catch(e) {
            assert(e.type == "ContractViolation");
        }

        py.fini();
    }
}
```

### DoD（完了定義）
- ✅ pre条件違反検出
- ✅ post条件違反検出
- ✅ ErrorBox化
- ✅ トグル切り替え（strict/warn）

### 期間
**2週間**

---

## 🎲 M3: Deterministic

### 目的
`random/time/os.urandom` に shims を適用

### 成果物
- [ ] Deterministic shims実装
- [ ] random固定化
- [ ] time論理クロック化
- [ ] スモーク再現性確認

### 実装内容

#### Deterministic Random
```python
# hakorune_deterministic.py
class DeterministicRandom:
    def __init__(self, seed=42):
        self._rng = random.Random(seed)

    def random(self):
        return self._rng.random()

    def randint(self, a, b):
        return self._rng.randint(a, b)

# モンキーパッチ
if os.getenv('HAKO_DETERMINISTIC') == '1':
    random.random = DeterministicRandom().random
```

#### Logical Clock
```python
class LogicalClock:
    def __init__(self):
        self._tick = 0

    def monotonic(self):
        self._tick += 1
        return float(self._tick)

    def time(self):
        return self.monotonic()

# モンキーパッチ
if os.getenv('HAKO_DETERMINISTIC') == '1':
    time.monotonic = LogicalClock().monotonic
```

#### テストケース
```hakorune
using "python"

box Main {
    flow main() {
        let py = PyRuntimeBox();

        // Deterministic mode
        let random = py.import("random");

        // 同じseedなら同じ結果
        let r1 = random.get("random").exec([]);
        let r2 = random.get("random").exec([]);
        assert(r1 == r2);  // Deterministic!

        py.fini();
    }
}
```

### トグル
```bash
# Deterministic モード有効化
HAKO_DETERMINISTIC=1
HAKO_TRACE_EFFECTS=1  # 効果ログ収集
```

### DoD（完了定義）
- ✅ random固定化動作
- ✅ time論理クロック動作
- ✅ スモーク再現性100%
- ✅ トレースログ記録

### 期間
**1-2週間**

---

## 🏗️ M4: Type/Instance 反射

### 目的
クラス→`PyTypeBox`、インスタンス→`PyInstanceBox`

### 成果物
- [ ] PyTypeBox実装
- [ ] PyInstanceBox実装
- [ ] get/setField統一
- [ ] call_method統一

### 実装内容

#### PyTypeBox
```rust
pub struct PyTypeBox {
    py_type: Py<PyType>,
}

impl PyTypeBox {
    pub fn get_name(&self) -> String {
        // クラス名取得
    }

    pub fn get_bases(&self) -> Vec<PyTypeBox> {
        // 基底クラスリスト
    }

    pub fn new_instance(&self, args: Vec<Value>) -> PyInstanceBox {
        // インスタンス化
    }
}
```

#### PyInstanceBox
```rust
pub struct PyInstanceBox {
    py_obj: Py<PyAny>,
}

impl PyInstanceBox {
    pub fn get_field(&self, name: &str) -> Value {
        // フィールド取得
    }

    pub fn set_field(&self, name: &str, value: Value) {
        // フィールド設定
    }

    pub fn call_method(&self, name: &str, args: Vec<Value>) -> Value {
        // メソッド呼び出し
    }
}
```

#### テストケース
```hakorune
using "python"

box Main {
    flow main() {
        let py = PyRuntimeBox();

        // クラス取得
        let code = "
class Player:
    def __init__(self, name):
        self.name = name
        self.health = 100

    def heal(self, amount):
        self.health += amount
";
        py.exec(code);
        let PlayerClass = py.get("Player");  // PyTypeBox

        // インスタンス化
        let player = PlayerClass.new_instance(["Alice"]);  // PyInstanceBox

        // フィールドアクセス
        let name = player.get_field("name");
        assert(name == "Alice");

        // メソッド呼び出し
        player.call_method("heal", [20]);
        let health = player.get_field("health");
        assert(health == 120);

        py.fini();
    }
}
```

### DoD（完了定義）
- ✅ PyTypeBox動作確認
- ✅ PyInstanceBox動作確認
- ✅ get/setField統一動作
- ✅ call_method統一動作

### 期間
**2-3週間**

---

## 🔄 M5: ABI 昇格

### 目的
C ABI→Hakorune ABI（vtable）へPoC

### 成果物
- [ ] Hakorune ABI実装
- [ ] vtable生成
- [ ] C ABI互換レイヤー
- [ ] 段階切り替え確認

### 実装内容

#### Hakorune ABI（vtable）
```rust
#[repr(C)]
pub struct HakoruneABI {
    vtable: *const VTable,
    data: *mut c_void,
}

#[repr(C)]
pub struct VTable {
    init: extern "C" fn(*mut c_void),
    fini: extern "C" fn(*mut c_void),
    invoke: extern "C" fn(*mut c_void, method_id: u32, args: *const TLV) -> TLV,
}
```

#### トグル切り替え
```bash
# C ABI使用
HAKO_ABI_VTABLE=0

# Hakorune ABI使用
HAKO_ABI_VTABLE=1
HAKO_ABI_STRICT=1  # フォールバック禁止
```

#### テストケース
```hakorune
using "python"

box Main {
    flow main() {
        // どちらのABIでも同じコードが動く
        let py = PyRuntimeBox();
        let math = py.import("math");
        let sqrt = math.get("sqrt");
        let result = sqrt.exec([2.0]);
        assert(result.to_float() == 1.4142135623730951);
        py.fini();
    }
}
```

### DoD（完了定義）
- ✅ Hakorune ABI動作確認
- ✅ C ABI互換性維持
- ✅ トグル切り替え動作
- ✅ パフォーマンス劣化なし

### 期間
**3-4週間**

---

## 📦 M6: AOT/EXE の包み込み（10.5b連動）

### 目的
Python依存を**スタティック or 埋め込み**方針で詰め、`hako` EXE生成ラインに統合

### 成果物
- [ ] スタティックリンク対応
- [ ] 埋め込みPython対応
- [ ] EXE生成パイプライン統合
- [ ] クロスプラットフォーム確認

### 実装内容

#### スタティックリンク
```toml
# Cargo.toml
[dependencies]
pyo3 = { version = "0.20", features = ["auto-initialize", "extension-module"] }

[build-dependencies]
pyo3-build-config = "0.20"
```

#### 埋め込みPython
```rust
// Python標準ライブラリを埋め込み
#[cfg(feature = "embed-python")]
const PYTHON_STDLIB: &[u8] = include_bytes!("python3.11.zip");
```

#### EXE生成
```bash
# AOTコンパイル
hakorune --backend llvm --embed-python script.hkr -o app

# 生成されたEXE
./app  # Python依存なしで実行可能
```

### プラットフォーム対応

| Platform | Python | 方式 |
|----------|--------|------|
| Linux | 3.8+ | 動的/静的両対応 |
| macOS | 3.8+ | 動的優先 |
| Windows | 3.8+ | 埋め込み推奨 |

### DoD（完了定義）
- ✅ Linux EXE生成
- ✅ macOS EXE生成
- ✅ Windows EXE生成
- ✅ Python依存最小化
- ✅ サイズ最適化

### 期間
**4-6週間**

---

## 📊 全体スケジュール

| マイルストーン | 期間 | 開始条件 |
|--------------|------|---------|
| M0: PyRuntimeBox最小化 | 1週間 | Phase 15完了 |
| M1: Capabilityゲート | 1-2週間 | M0完了 |
| M2: Contract検証 | 2週間 | M1完了 |
| M3: Deterministic | 1-2週間 | M2完了 |
| M4: Type/Instance反射 | 2-3週間 | M3完了 |
| M5: ABI昇格 | 3-4週間 | M4完了 |
| M6: AOT/EXE統合 | 4-6週間 | M5完了 |

**合計**: 約3-5ヶ月

---

## 🔗 関連ドキュメント

- [強化版アーキテクチャv2](../design/enhanced-architecture-v2.md) - 設計詳細
- [メタ設定例](../design/meta-config-examples.md) - hako.toml設定例
- [リスクと対策](../design/risks-and-mitigations.md) - 既知のリスク
- [Phase 20 README](../README.md) - 全体概要

---

**最終更新**: 2025-10-02
**作成者**: ChatGPT Pro (UltraThink Mode)
**ステータス**: 計画・Phase 15完了後に実装

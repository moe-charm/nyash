# Python-Hakorune統合 強化版アーキテクチャ v2

**作成**: 2025-10-02
**ソース**: ChatGPT Pro UltraThink Mode
**ステータス**: 設計提案

---

## 📋 概要

Phase 10.5の「Active」設計を踏まえつつ、**強化版Hakorune**（Effect/Capability/Contract/Policy・PHI検証・BlockVMap等）の上に載せる、**安全で境界に問題を閉じ込めたPythonブリッジ**の設計書です。

---

## 🎯 ねらい（Phase 10.5の「Active」意図との整合）

- 最小コアで **"箱（Box）に閉じ込めたPython実行環境"** を提供
- VM/LLVM/AOTラインから同じ呼び出し規約で扱える
- **当面はVMハーネス（llvmlite側）を使って決定実行とスモークの安定性を優先**、AOT/EXEは段階導入

---

## 🏗️ 1. Box階層（木構造）の基本セット

### PyRuntimeBox（ルート・単一 or 少数）

**役割**:
- Pythonランタイム（CPython埋め込み）
- `import` 解決
- GIL管理
- モジュールキャッシュ

**ライフサイクル**:
- `init`: インタプリタ起動
- `fini`: 停止
- **Capability**で"import可能モジュール／FS/NET/ENV"を限定

**実装指針**:
- まずは既存 **Plugin-First**（C ABI・TLV）でブリッジ
- 後段で **Hakorune ABI（vtable）** に段階移行
- `HAKO_ABI_VTABLE` 等のgateを活用

```hakorune
box PyRuntimeBox {
    // 初期化・終了
    init()
    fini()

    // モジュール
    import(name: StringBox) -> PyModuleBox
    reflect(module_name: StringBox) -> PyModuleBox

    // 実行
    eval(code: StringBox) -> PyObjectBox
    exec(code: StringBox)
}
```

---

### PyModuleBox

**役割**:
- `import "pkg.mod"` の結果をBoxとして公開
- 属性 = フィールド
- 関数 = MethodBox

**ライフサイクル**:
- `PyRuntimeBox` が所有
- `fini` で参照解除（refcnt を減らす）

```hakorune
box PyModuleBox {
    // 属性アクセス
    get(name: StringBox) -> PyObjectBox
    set(name: StringBox, value: PyObjectBox)

    // 列挙
    list_attributes() -> ArrayBox
}
```

---

### PyTypeBox / PyInstanceBox

**役割**:
- Pythonのクラス定義とインスタンスをBox化
- `get/setField` と `call_method` を統一導線に

**所有権タグ**:
- **shared**（CPython RC前提）
- `unique` が必要な場合は"借用トークン"で一時的に排他操作を許可

```hakorune
box PyTypeBox {
    // クラス情報
    get_name() -> StringBox
    get_bases() -> ArrayBox
    get_methods() -> ArrayBox

    // インスタンス化
    new_instance(args: ArrayBox) -> PyInstanceBox
}

box PyInstanceBox {
    // フィールドアクセス
    get_field(name: StringBox) -> PyObjectBox
    set_field(name: StringBox, value: PyObjectBox)

    // メソッド呼び出し
    call_method(name: StringBox, args: ArrayBox) -> PyObjectBox
}
```

---

### PyFunctionBox / PyCallableBox

**役割**:
- 呼出しを `exec(args)->result`（Pulse的）に落として**最小ライフサイクル**で扱える

```hakorune
box PyFunctionBox {
    // Pulse: 完全な一発実行
    exec(args: ArrayBox) -> PyObjectBox

    // メタ情報
    get_name() -> StringBox
    get_signature() -> StringBox
}
```

---

### PyIteratorBox / PyGeneratorBox（必要に応じて）

**役割**:
- `__iter__`/`__next__` をBox化
- flow実行と相性を取る

```hakorune
box PyIteratorBox {
    next() -> PyObjectBox
    has_next() -> BoolBox
}

box PyGeneratorBox {
    send(value: PyObjectBox) -> PyObjectBox
    throw(error: ErrorBox)
}
```

> **Phase 10.5の設計メモと整合**："最小の箱から着手 → 後から拡張"

---

## 🔄 2. ライフサイクル対応（Hakorune流）

### init/fini

**PyRuntimeBox.init()**:
```rust
fn init() {
    Py_Initialize();
    // import policyのロード
    // キャッシュ構築
}
```

**PyRuntimeBox.fini()**:
```rust
fn fini() {
    // weakref.finalize併用で安全解放（循環対策）
    Py_Finalize();
}
```

### Pulse

**PyFunctionBox.exec** は完全Pulse（init/fini不要）:
- **作用範囲はmetaのCapabilityに閉じ込め**
- 副作用は明示的にEffect宣言

### 所有権タグ

- 既定: **shared**（RC準拠）
- **unique** が必要な経路: `BorrowMutToken` を発行して**単発処理中のみ**排他

> これらは **管理棟のトグル**とも親和性が高い（dev: 緩める / prod: 厳格）

---

## 🎭 3. Effect / Capability / Contract の外付け（meta）

### Effect（論理的効果名）

```toml
[box.PyRuntimeBox.effects]
available = [
    "py.import",
    "py.fs",
    "py.net",
    "py.env",
    "py.time",
    "py.random",
    "py.subprocess"
]
```

### Capability（許可リスト）

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
enforce = true  # devではfalse→観測のみ
```

### Contract

**Pre条件**:
```toml
[contracts.PyFunctionBox.exec]
pre = [
    "args.len <= 8",
    "bytes_total <= 1_000_000"
]
```

**Post条件**:
```toml
[contracts.PyFunctionBox.exec]
post = [
    "result.size <= 1_000_000",
    "no_exception",
    "allow_none = true"
]
```

### 監査トグル

```bash
# Dev/CI
HAKO_PLUGIN_CAPS_ENFORCE=1
HAKO_CHECK_CONTRACTS=1

# Prod
HAKO_PLUGIN_CAPS_ENFORCE=1
HAKO_CHECK_CONTRACTS=0  # 警告のみ
```

---

## 🔁 4. Deterministic / Repro の扱い

### Deterministic モード

**問題**:
- `random`, `time`, `os.urandom` 等は非決定的

**解決策**:
- **Capability トークン経由**で提供するshimsを使用

**実装例**:

```python
# py.random shim
class DeterministicRandom:
    def __init__(self, seed):
        self._rng = random.Random(seed)

    def random(self):
        return self._rng.random()

# py.time shim
class LogicalClock:
    def __init__(self):
        self._tick = 0

    def monotonic(self):
        self._tick += 1
        return float(self._tick)
```

**トグル**:
```bash
# Deterministic モード有効化
HAKO_DETERMINISTIC=1
HAKO_TRACE_EFFECTS=1  # 効果ログ収集
```

**メリット**:
- **flow Main.main** 実行のスナップショット再現が可能
- デバッグ容易性向上
- テストの再現性保証

---

## 🌳 5. 木構造への「落とし方」（反射 → Box化）

### ビルダー（反射）

**PyRuntimeBox.reflect(module_name: String) -> PyModuleBox**:

1. `dir(module)` と `inspect` で**属性ツリーを走査**
2. **Box**を生成
3. **規約**:
   - 公開対象は `__all__` 優先
   - ない場合は `_` 先頭を除外
   - `@hako_export` デコレータがあれば優先で `FunctionBox` 化

**実装擬似コード**:

```python
# Python側（リフレクションヘルパー）
def reflect_module(module_name):
    mod = importlib.import_module(module_name)

    # 属性リスト取得
    if hasattr(mod, '__all__'):
        attrs = mod.__all__
    else:
        attrs = [a for a in dir(mod) if not a.startswith('_')]

    # Box化
    boxes = {}
    for attr_name in attrs:
        attr = getattr(mod, attr_name)

        if callable(attr):
            boxes[attr_name] = create_function_box(attr)
        elif inspect.isclass(attr):
            boxes[attr_name] = create_type_box(attr)
        else:
            boxes[attr_name] = create_object_box(attr)

    return boxes
```

### 使用例

```hakorune
using "python"

box Main {
    flow main() {
        // init: CPython起動
        let py = PyRuntimeBox();

        // PyModuleBox
        let m = py.reflect("math");

        // PyFunctionBox
        let sqrt = m.get("sqrt");

        // 実行
        let r = sqrt.exec([2.0]);
        print(r);  // -> 1.4142...

        // 正常停止
        py.fini();
    }
}
```

> 反射で"木"を作り、**Box統一API（get/set/call）で走る** — Phase 10.5の「Activeは最小の箱から導入」を踏襲

---

## ⚠️ 6. 例外とエラー境界

### Python例外 → ErrorBox

```hakorune
box ErrorBox {
    type: StringBox       // "ZeroDivisionError"
    message: StringBox    // "division by zero"
    traceback: StringBox  // フルトレースバック
}
```

**変換例**:
```python
# Python側
try:
    result = some_function()
except Exception as e:
    return ErrorBox(
        type=type(e).__name__,
        message=str(e),
        traceback=traceback.format_exc()
    )
```

### Contract違反

**Strict モード**:
```bash
HAKO_EXTERN_STRICT=1  # fail-fast
```

**通常モード**:
```bash
HAKO_EXTERN_STRICT=0  # 警告 + ErrorBox返却
```

---

## 🎛️ 7. 実行ポリシーとトグル（管理棟）

### ハーネス

```bash
# 既定でPythonハーネス使用
HAKO_LLVM_USE_HARNESS=1  # デフォルトON
```

### PHIオプション

**ブリッジ互換**（PHI-off）:
```bash
HAKO_VERIFY_ALLOW_NO_PHI=1  # 互換用途のみ
```

**統一**（PHI-on）:
```bash
HAKO_VERIFY_ALLOW_NO_PHI=0  # 推奨
# Verifierを**PHI直後に常時実行**
```

**モード設定**:
- Dev: fail-fast
- Prod: 警告

### ABI

**初期**（C ABI - TLV）:
```bash
HAKO_ABI_VTABLE=0  # C ABI使用
```

**段階移行**（Hakorune ABI - vtable）:
```bash
HAKO_ABI_VTABLE=1
HAKO_ABI_STRICT=1  # フォールバック禁止
```

### Plugin観測

```bash
# CI/Dev
HAKO_TRACE_EFFECTS=1
HAKO_PLUGIN_META=1
HAKO_PLUGIN_CAPS_ENFORCE=1
```

---

## 🚀 強化版Hakoruneでの設計図アップデート

### 1. 境界検証を"必須フェーズ"に

**実装**:
- `finalize_phis()` の直後で **SSA/支配関係/到達性**を検証
- Dev: 例外
- Prod: 警告＋トレース保全

**効果**:
- Pythonブリッジ有無に関わらず**毎回**回る
- IR品質を境界に閉じ込める

### 2. BlockVMap型でvmap/_current_vmapを統合

**実装**:
- 参照は必ず `BlockVMap::at(block)` 経由
- 呼び出し側から**どの視点が"真"か**を隠蔽

**効果**:
- Python連携でもブロック境界で値がぶれない
- 事故防止

### 3. InstructionContextを必須引数化

**実装**:
- すべての `lower_*` に統一コンテキストを渡す
- **ログ/例外に命令位置・BB名を強制添付**

**効果**:
- 原因追跡が一気に楽に

### 4. flow Main.main を"仕様"として固定

**実装**:
- Phase 10.5のActive方針と揃えて **flow優先**
- staticを覚える必要なし

**効果**:
- 入口の一意化
- セルフホストの再現ビルド検証にも効く

### 5. 開発→本番の昇格パスを明確化

**Dev**:
```bash
HAKO_*_ENFORCE=0
HAKO_TRACE=1
# fail-fast ON
```

**Prod**:
```bash
HAKO_*_ENFORCE=1
HAKO_TRACE=0
# fail-fast OFF（ErrorBox＋警告）
```

**効果**:
- **管理棟env**にすでに集約されている設計
- 運用で迷わない

---

## 🔗 関連ドキュメント

- [マイルストーン](../planning/milestones.md) - M0〜M6の詳細計画
- [メタ設定例](meta-config-examples.md) - hako.toml設定例
- [リスクと対策](risks-and-mitigations.md) - 既知のリスクと対策
- [Phase 20 README](../README.md) - 全体概要

---

**最終更新**: 2025-10-02
**作成者**: ChatGPT Pro (UltraThink Mode)
**レビュー**: 未実施
**ステータス**: 設計提案・Phase 15完了後に実装予定

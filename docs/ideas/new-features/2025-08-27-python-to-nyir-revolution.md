# Python → NyIR 革命 - 箱に被せてVM/ネイティブ/WASM

作成日: 2025-08-27
Status: 構想段階
Priority: High
Related: ChatGPT5との共同構想

## 🎯 ビジョン

**PythonをBoxで包んで、GCオン/オフ切替可能、GIL不要、VM/ネイティブ/WASM全部で動く世界**

既存のPythonコンパイラが解決できなかった問題を「Everything is Box」で根本解決。

## 🚀 三段ロケット構想

### 1️⃣ ブリッジ層（今すぐ動く）
```nyash
// CPythonを"箱で包む"
ForeignBox<PyObject>  // PyObjを不透明ハンドルで保持
ProxyBox             // GIL・イベントループはバス越しで処理
ny_py_host          // C-ABI: ny_py_call/finalize/pin/unpin

// fini ↔ CPython finalizerを相互接続（二重解放防止）
```

### 2️⃣ 部分コンパイル層（静的部分をNyIRへ）
```python
@ny_static  # 型固定アノテーション
def calculate(numbers: List[float]) -> float:
    total = 0.0
    for n in numbers:
        total += n * n
    return total
```
↓
```nyash
// NyIRに変換 → VM/Cranelift/WASMで実行
function calculate(numbers) {
    local total = 0.0
    local i = 0
    loop(i < numbers.length()) {
        total = total + numbers.get(i) * numbers.get(i)
        i = i + 1
    }
    return total
}
```

### 3️⃣ トレースJIT層（ホット部自動最適化）
- CPython実行をプロファイル
- ホットトレースだけNyIR生成
- CraneliftでJIT（PyPy風だが後段はNyIR）

## 📊 既存Pythonコンパイラとの決定的違い

| 既存 | Nyash+Python |
|------|-------------|
| GC必須（参照カウント＋GC） | **GCオン/オフ切替可能** |
| GIL必須で真の並列困難 | **SyncBoxでGIL不要並列化** |
| C++変換やPyPy VM限定 | **VM/Cranelift/WASM/LLVM全対応** |
| 言語固有の最適化 | **NyIRで多言語統合基盤** |

## 🏗️ マッピング設計

| Python | Nyash/NyIR |
|--------|-----------|
| int/float/bool/str | Value/IntegerBox/StringBox |
| list/tuple | VecBox（不変は`look`） |
| dict | MapBox |
| class/object | Box（public/privateフィールド） |
| with | `with … {}`（RAII） |
| async/await | ChannelBox/Bus |
| exception | `Throw/Try`（or `Result`） |

反射・動的属性・メタクラスは`ForeignBox<PyObject>`で残してOK。

## 💎 期待される効果

### 性能面
- 数値/ループ/データ処理で**2-20倍高速化**
- GCオフで**リアルタイム性能保証**
- SyncBoxで**真の並列実行**

### 開発面
- **段階導入可能** - 関数単位で移行
- **GCオンで開発** - メモリリーク即検出
- **同一コードで多プラットフォーム** - VM/WASM/ネイティブ

### 革新性
- **言語統合基盤** - Python/JS/Lua全部NyIRへ
- **GCは練習用補助輪** - 本番では不要
- **箱=ライフサイクル** - 統一メモリモデル

## 📅 実装ロードマップ（1週間MVP）

### Day 1-2: ny_py_host（C-ABIブリッジ）
- [ ] call/finalize/pin/unpin実装
- [ ] ForeignBox<PyObject>基盤

### Day 3-4: py-nyc v0（基本構文）
- [ ] def/for/if/return変換
- [ ] int/float/list/dict対応
- [ ] @ny_staticアノテーション

### Day 5-6: 混在実行
- [ ] 関数単位でNyIR or CPython選択
- [ ] ベンチマーク3本作成

### Day 7: Cranelift JIT
- [ ] ホット関数のみコンパイル
- [ ] 性能測定・デモ準備

## 🔧 技術的詳細

### ブリッジ設計
```c
// C-ABI最小セット
ny_py_call(func_id, argc, argv)
ny_py_finalize(obj_id)
ny_py_pin(obj_id)    // GC防止
ny_py_unpin(obj_id)  // GC許可
```

### 安全性保証
- **二重解放防止**: fini ↔ finalizer相互接続
- **GIL隔離**: ProxyBox経由でPython処理
- **型安全**: @ny_static領域のみコンパイル

### リスク回避
- **動的機能**: @ny_static外はブリッジ経由
- **C拡張**: 初期はForeignBox、将来NyIR実装
- **GIL**: NyIR部分はGIL非依存で多コア化

## 🎭 デモシナリオ

```python
# 同一ソースを3実行
python calculate.py      # CPython
nyash calculate.ny       # Nyash VM  
nyash --wasm calculate   # ブラウザ実行

# 結果: 完全一致 & 速度グラフ表示
# GCオン/オフ切替デモも同時実施
```

## 💡 世界が驚くポイント

「PythonをVMでもWASMでもRust並速度で動かせる」だけでなく：

**「GCもGILも必須じゃない、Box原理で制御できる」**

つまり：**Pythonコードの見た目のまま、Rustに迫る制御性を後付けできる**

---

*「箱に被せれば、どの言語もVM/ネイティブ/WASMで走る」- 言語統合基盤の誕生*
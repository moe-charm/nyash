# Transpiler - Python→Hakorune トランスパイラー

## 📋 概要

PythonコードをHakoruneコードに変換するトランスパイラーの設計・仕様です。

## 📁 ファイル一覧

- **[python-to-hakorune-spec.md](python-to-hakorune-spec.md)** - Python→Hakorune変換仕様

## 🎯 目的

### トランスパイラーの役割

1. **構文変換**
   - Python構文 → Hakorune構文
   - イディオム変換
   - スタイル変換

2. **型注釈の活用**
   - Pythonの型ヒント → Hakorune型
   - 型推論の補助
   - 型安全性の向上

3. **最適化**
   - Hakorune向け最適化
   - 不要なコードの削除
   - パフォーマンス改善

## 🔄 変換例

### 基本構文

```python
# Python
class Player:
    def __init__(self, name: str):
        self.name = name
        self.health = 100

    def heal(self, amount: int):
        self.health += amount
```

```hakorune
// Hakorune
box Player {
    name: StringBox
    health: IntegerBox

    birth(name: StringBox) {
        me.name = name
        me.health = integer.create(100)
    }

    heal(amount: IntegerBox) {
        me.health = me.health.add(amount)
    }
}
```

### 制御構文

```python
# Python
for i in range(10):
    print(i)
```

```hakorune
// Hakorune
local i = integer.create(0)
loop(i.compare("<", 10)) {
    console.log(i.to_string())
    i = i.add(1)
}
```

### リスト操作

```python
# Python
items = [1, 2, 3]
items.append(4)
```

```hakorune
// Hakorune
local items = array.create()
items.push(integer.create(1))
items.push(integer.create(2))
items.push(integer.create(3))
items.push(integer.create(4))
```

## 🏗️ アーキテクチャ

### トランスパイラーのフェーズ

1. **パース**
   - Pythonコードをパース
   - AST生成

2. **型推論**
   - 型ヒントの解析
   - 型推論の実行
   - 型情報の付与

3. **変換**
   - Python AST → Hakorune AST
   - イディオム変換
   - 最適化

4. **コード生成**
   - Hakorune ASTからコード生成
   - フォーマット適用
   - 出力

### 変換ルール

| Python構文 | Hakorune構文 | 備考 |
|-----------|-------------|------|
| `class` | `box` | クラス→Box |
| `__init__` | `birth` | コンストラクタ |
| `self` | `me` | 自己参照 |
| `for ... in range()` | `loop(...)` | ループ |
| `if ... elif ... else` | `if ... elif ... else` | 条件分岐 |
| `def func()` | `func()` | 関数定義 |
| `x = y` | `local x = y` | 変数宣言 |
| `list[index]` | `list.get(index)` | 配列アクセス |
| `dict[key]` | `dict.get(key)` | 辞書アクセス |

## 🔧 技術的課題

### 1. Python固有機能の扱い

#### サポートする機能
- 基本データ型（int, str, list, dict）
- クラス・メソッド
- 制御構文（if, for, while）
- 関数定義

#### サポートしない/制限する機能
- メタクラス
- デコレーター（一部のみ）
- ジェネレーター
- 内包表記（将来対応）
- async/await（Phase 15以降）

### 2. 型推論

- 型ヒントを優先使用
- 未注釈の場合は推論
- 推論不可能な場合は`AnyBox`

### 3. ネーミング変換

- snake_case → camelCase（オプション）
- Python予約語の回避
- Hakorune予約語との衝突回避

## 📊 実装ステータス

| コンポーネント | ステータス | 備考 |
|--------------|----------|------|
| 変換仕様 | ✅ 完了 | 設計書完成 |
| パーサー統合 | 📅 未実装 | - |
| 型推論 | 📅 未実装 | - |
| AST変換 | 📅 未実装 | - |
| コード生成 | 📅 未実装 | - |

## 🎯 使用例

```bash
# コマンドライン（将来）
hakorune transpile script.py -o script.hkr

# プログラマティック（将来）
local transpiler = new PythonTranspiler()
local hakorune_code = transpiler.transpile(python_code)
```

## ⚠️ 制限事項

1. **完全互換性は保証しない**
   - Pythonのすべての機能はサポートしない
   - 一部機能は手動移植が必要

2. **実行時動的機能**
   - `eval()`, `exec()` は変換不可
   - 動的属性追加は制限

3. **標準ライブラリ**
   - Python標準ライブラリは自動変換しない
   - Hakorune equivalent への手動マッピング必要

## 🔗 関連ドキュメント

- [Phase 20 メインREADME](../README.md)
- [Parser Integration](../parser-integration/)
- [Core Implementation](../core-implementation/)

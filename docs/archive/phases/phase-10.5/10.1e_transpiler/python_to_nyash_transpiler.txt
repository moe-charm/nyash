# Python → Nyashトランスパイラー機能
～PythonParserBoxの応用による自動変換～

## 🎯 概要

PythonParserBoxでPython AST → Nyash ASTの変換ができるなら、それを**Nyashソースコードとして出力**できる！

## 🚀 実現可能な機能

### 1. Python → Nyashファイル変換
```python
# input.py
def calculate(x, y):
    result = x * 2 + y
    return result

for i in range(10):
    print(calculate(i, 5))
```

↓ 変換

```nyash
# output.nyash
function calculate(x, y) {
    local result
    result = x * 2 + y
    return result
}

local i
for i in range(0, 10) {
    print(calculate(i, 5))
}
```

### 2. 実装方法

```rust
impl PythonParserBox {
    // Python → Nyashファイル出力
    pub fn transpile_to_file(&self, python_code: &str, output_path: &str) -> Result<(), String> {
        // 1. Python → JSON AST
        let json_ast = self.parse_to_json(python_code)?;
        
        // 2. JSON AST → Nyash AST
        let nyash_ast = self.json_to_nyash_ast(&json_ast)?;
        
        // 3. Nyash AST → Nyashソースコード
        let nyash_code = self.ast_to_nyash_source(&nyash_ast)?;
        
        // 4. ファイルに出力
        std::fs::write(output_path, nyash_code)?;
        
        Ok(())
    }
    
    // AST → Nyashソースコード生成
    fn ast_to_nyash_source(&self, ast: &NyashAst) -> Result<String, String> {
        let mut output = String::new();
        let formatter = NyashFormatter::new();
        
        for item in &ast.items {
            formatter.format_item(item, &mut output)?;
        }
        
        Ok(output)
    }
}
```

## 📋 使用例

### コマンドライン版
```bash
# PythonファイルをNyashに変換
nyash-transpile input.py -o output.nyash

# 標準出力に出力
nyash-transpile script.py

# 部分変換（サポートされる構文のみ）
nyash-transpile --partial complex_script.py
```

### Nyashスクリプト内での使用
```nyash
local transpiler = new PythonParserBox()

// Pythonコードを読み込み
local python_code = FileBox.read("algorithm.py")

// Nyashに変換
local nyash_code = transpiler.to_nyash_source(python_code)

// ファイルに保存
FileBox.write("algorithm.nyash", nyash_code)

// または直接実行
eval(nyash_code)
```

## 🎨 変換マッピング例

### 基本構文
| Python | Nyash |
|--------|-------|
| `def func():` | `function func() {` |
| `if x > 0:` | `if (x > 0) {` |
| `for i in range(10):` | `for i in range(0, 10) {` |
| `while x < 10:` | `while (x < 10) {` |
| `x = 5` | `local x; x = 5` または `x = 5`（スコープによる）|

### データ型
| Python | Nyash |
|--------|-------|
| `x = 42` | `x = 42` |
| `s = "hello"` | `s = "hello"` |
| `lst = [1, 2, 3]` | `lst = new ArrayBox([1, 2, 3])` |
| `d = {"a": 1}` | `d = new MapBox(); d.set("a", 1)` |

### 特殊なケース
```python
# Pythonのfor/else
for i in items:
    if condition:
        break
else:
    print("No break")
```

```nyash
# Nyashでの実装（フラグを使用）
local broke = false
for i in items {
    if (condition) {
        broke = true
        break
    }
}
if (not broke) {
    print("No break")
}
```

## 🌟 利点

### 1. 段階的移行支援
- 既存のPythonプロジェクトを段階的にNyashに移行
- 変換されたコードを手動で最適化可能

### 2. 学習ツールとして
- PythonユーザーがNyash構文を学ぶ
- 両言語の違いを理解

### 3. コード生成
- Pythonで書いたアルゴリズムをNyashネイティブコードに
- より高速な実行のための前処理

### 4. 逆方向変換の可能性
- Nyash → Pythonも将来的に可能
- 真のバイリンガル環境

## ⚠️ 制限事項と課題

### 1. 完全な互換性は不可能
- Pythonの動的機能すべては変換できない
- 一部の構文は手動調整が必要

### 2. 意味論の違い
```python
# Pythonのデフォルト引数（定義時評価）
def f(x, lst=[]):
    lst.append(x)
    return lst
```

```nyash
// Nyashでは毎回新しいリスト（異なる挙動）
function f(x, lst) {
    if (lst == null) {
        lst = new ArrayBox()
    }
    lst.push(x)
    return lst
}
```

### 3. サポートレベルの明示
```nyash
// 生成されたファイルのヘッダー
// Generated from Python by Nyash Transpiler v1.0
// Original file: script.py
// Conversion rate: 85% (15/17 functions transpiled)
// Manual review recommended for: async_func, generator_func
```

## 📊 実装フェーズ

### Phase 1.5: 基本トランスパイラ（Phase 1と並行）
- [ ] NyashFormatter実装（AST → ソースコード）
- [ ] 基本構文の出力（def, if, for, while）
- [ ] インデント管理
- [ ] コメント保持（可能な範囲で）

### Phase 2.5: 高度な変換
- [ ] クラス → Box変換
- [ ] 特殊メソッド → Nyashメソッド
- [ ] import文の処理

### Phase 3.5: ツール化
- [ ] コマンドラインツール
- [ ] VSCode拡張機能
- [ ] オンライン変換ツール

## 🎉 期待される効果

1. **Pythonエコシステムの資産をNyashネイティブ化**
2. **パフォーマンスクリティカルな部分をNyash/MIR/JITで高速化**
3. **両言語間のシームレスな相互運用**
4. **Nyashの採用障壁を大幅に下げる**

---

これは**PythonParserBoxの価値をさらに高める**素晴らしい応用です！
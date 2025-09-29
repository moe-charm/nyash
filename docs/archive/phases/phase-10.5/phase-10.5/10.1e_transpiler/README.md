[Archived] 旧10.1系ドキュメントです。最新は ../INDEX.md を参照してください。

# Phase 10.1e - Python → Nyashトランスパイラー

## 🎯 このフェーズの目的
Python ASTをNyashソースコードとして出力する機能を実装する。

## 📁 実装ドキュメント
- **`python_to_nyash_transpiler.txt`** - トランスパイラー設計

## 🔧 実装機能

### 1. AST → Nyashソース生成
```rust
impl PythonParserBox {
    pub fn to_nyash_source(&self, python_code: &str) -> Result<String, String> {
        // Python → JSON AST → Nyash AST → Nyashソース
    }
}
```

### 2. 変換例
```python
# Python入力
def add(x, y):
    return x + y

result = add(10, 5)
```

```nyash
# Nyash出力
function add(x, y) {
    return x + y
}

local result
result = add(10, 5)
```

### 3. 出力フォーマッター
- インデント管理
- 括弧の追加（Nyashは明示的）
- コメント保持（可能な範囲で）

## 🛠️ コマンドラインツール
```bash
# 基本変換
nyash-transpile input.py -o output.nyash

# 変換統計付き
nyash-transpile --stats complex.py
# Output: Converted 15/17 functions (88%)

# 部分変換（サポート関数のみ）
nyash-transpile --partial script.py
```

## ✅ 完了条件
- [ ] `to_nyash_source()` メソッドが動作する
- [ ] 基本的なPythonコードが正しいNyashに変換される
- [ ] インデントが正しく管理される
- [ ] 変換統計が表示される
- [ ] ファイル出力ができる

## 🌟 期待される利用シーン
1. **学習ツール** - PythonユーザーがNyash構文を学ぶ
2. **段階的移行** - 既存Pythonコードの移行
3. **性能最適化** - ホットパスをNyashネイティブに

## ⏭️ 次のフェーズ
→ Phase 10.1f (テストとベンチマーク)
[Archived] 旧10.1系ドキュメントです。最新は ../INDEX.md を参照してください。

Note: 本来は「PythonをNyashで動かすフェーズ（パーサー統合）」の位置づけでしたが、現在は順番を変更し、先に 10.5b（MIR→VM→ネイティブビルド基盤）を進めています。

# Phase 10.1c - パーサー統合実装

## 🎯 このフェーズの目的
pyo3を使ってCPythonパーサーをNyashに統合し、Python AST → JSON → Nyash ASTの変換パイプラインを構築する。

## 📁 実装ドキュメント
- **`python_parser_box_implementation_plan.txt`** - 技術的実装計画
- **`builtin_box_implementation_flow.txt`** - ビルトインBox実装フロー

## 🔧 実装タスク

### 1. PythonParserBoxの基本構造
```rust
pub struct PythonParserBox {
    base: BoxBase,
    py_helper: Arc<Mutex<PyHelper>>,
}
```

### 2. GIL管理の実装
```rust
// ✅ 良い例：GILを最小限に
let json_ast = Python::with_gil(|py| {
    py_helper.parse_to_json(py, code)
})?;

// GIL外でRust処理
let nyash_ast = py.allow_threads(|| {
    convert_json_to_nyash(json_ast)
});
```

### 3. Python側ヘルパー実装
- `ast.parse()` → JSON変換
- 位置情報の保持（lineno, col_offset）
- Python 3.11固定チェック

### 4. 関数単位フォールバック判定
```rust
pub fn can_compile(&self, func_def: &PythonAst) -> CompileResult {
    // サポートされているノードかチェック
    // CompileResult::Compile or CompileResult::Fallback
}
```

## ✅ 完了条件
- [ ] PythonParserBoxがビルトインBoxとして登録されている
- [ ] `parse_to_json()` メソッドが動作する
- [ ] GIL管理が適切に実装されている
- [ ] テレメトリー基盤が組み込まれている
- [ ] 簡単なPythonコードでJSON ASTが取得できる

## 🧪 動作確認
```nyash
local py = new PythonParserBox()
local json_ast = py.parse_to_json("def hello(): return 'Hello'")
print(json_ast)  // JSON ASTが表示される
```

## ⏭️ 次のフェーズ
→ Phase 10.1d (Core実装)

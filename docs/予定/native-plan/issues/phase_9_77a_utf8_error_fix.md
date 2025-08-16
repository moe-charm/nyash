# Phase 9.77a: WASM UTF-8エラー原因特定と修正

## 🚨 緊急度: 最高

**前提**: Phase 9.77の Task 1.1（BoxCall実装）と Task 1.2（wasmtime更新）は完了済み。Task 1.3のUTF-8エラーのみ未解決。

## 🐛 問題の詳細

### エラー内容
```bash
$ ./target/debug/nyash --compile-wasm local_tests/test_simple_wasm.nyash
🌐 Nyash WASM Compiler - Processing file: local_tests/test_simple_wasm.nyash 🌐
❌ Generated WASM is not valid UTF-8
```

### テストケース（最小再現）
```nyash
# local_tests/test_simple_wasm.nyash
local result = 42
```

### 🔍 調査済み内容

1. **エラーメッセージの発生元が不明**
   - `grep -r "Generated WASM is not valid UTF-8"` で見つからない
   - `grep -r "not valid UTF-8"` でも見つからない
   - ソースコード内に該当文字列が存在しない

2. **実装済み修正（効果なし）**
   ```rust
   // src/backend/wasm/mod.rs
   fn wat_to_wasm(&self, wat_source: &str) -> Result<Vec<u8>, WasmError> {
       // UTF-8検証を追加
       if !wat_source.is_ascii() {
           return Err(WasmError::WasmValidationError(
               "WAT source contains non-ASCII characters".to_string()
           ));
       }
       
       // wabt::wat2wasm に as_bytes() を追加
       let wasm_bytes = wabt::wat2wasm(wat_source.as_bytes())?;
       Ok(wasm_bytes)
   }
   ```

3. **デバッグ出力を追加済み（しかし表示されない）**
   ```rust
   eprintln!("🔍 WAT Source Debug (length: {}):", wat_source.len());
   eprintln!("WAT Content:\n{}", wat_source);
   eprintln!("✅ WAT source is ASCII-compatible");
   eprintln!("🔄 Converting WAT to WASM bytes...");
   ```
   - これらのデバッグ出力が一切表示されない
   - wat_to_wasm() が呼ばれていない可能性

## 🎯 調査すべきポイント

### 1. エラーメッセージの発生元
- [ ] main.rs または runner.rs でのエラー処理を確認
- [ ] --compile-wasm オプションの処理フローを追跡
- [ ] 外部プロセスやツールがエラーを出力している可能性

### 2. WASM生成パイプライン全体
```
Nyashソース → AST → MIR → WAT → WASM
                              ↑
                         ここで失敗？
```

### 3. 可能性のある原因
- wabt crate 以外の場所でWASM生成している？
- ファイル出力時にUTF-8エラーが発生？
- 標準出力への書き込みでエラー？

## 🔧 具体的な作業手順

### Step 1: エラーメッセージの発生元特定
```bash
# 1. main.rs の --compile-wasm 処理を確認
# 2. runner.rs の compile_wasm メソッドを追跡
# 3. エラーメッセージがどこで出力されているか特定
```

### Step 2: デバッグ情報の追加
```rust
// エラーが発生している場所に以下を追加
eprintln!("DEBUG: 処理フロー確認ポイント");
eprintln!("DEBUG: 変数の内容 = {:?}", variable);
```

### Step 3: 修正案
1. **エラーメッセージがwabt外部から来ている場合**
   - 正しいエラーハンドリングを実装
   - UTF-8検証を適切な場所に移動

2. **ファイル出力でエラーの場合**
   - バイナリファイル出力を明示的に指定
   - stdout への出力方法を見直し

3. **WAT生成に問題がある場合**
   - WAT形式の検証強化
   - 特殊文字のエスケープ処理追加

## 📝 テスト方法

```bash
# 1. 最小テストケースで確認
./target/debug/nyash --compile-wasm local_tests/test_simple_wasm.nyash

# 2. デバッグ出力付きで実行
RUST_LOG=debug ./target/debug/nyash --compile-wasm local_tests/test_simple_wasm.nyash 2>&1 | tee debug.log

# 3. WAT出力のみテスト（もし可能なら）
./target/debug/nyash --compile-wat local_tests/test_simple_wasm.nyash
```

## 🎯 成功基準

1. エラーメッセージの発生元が特定される
2. 最小テストケース（`local result = 42`）がWASMにコンパイルできる
3. 生成されたWASMファイルが wasmtime で実行可能

## 🚀 期待される成果

Phase 9.77完了により、NyashのWASMバックエンドが復旧し、以下が可能になる：
- BoxCall命令（toString, print等）がWASMで動作
- AOTコンパイル（.cwasm）が生成可能
- ブラウザでのNyash実行への道筋

---

**優先度**: 🔥 最高（WASMバックエンド全体がブロックされている）
**推定作業時間**: 2-4時間
**依存関係**: Phase 9.77 Task 1.1, 1.2は完了済み
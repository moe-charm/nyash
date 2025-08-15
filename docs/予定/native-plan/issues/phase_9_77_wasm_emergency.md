# Phase 9.77: WASM緊急復旧 - 詳細実装計画

## 🎯 概要
BoxCall命令未実装により基本的なNyash機能がWASMで全停止している緊急事態を段階的に解決する。

## 🚨 現在の緊急問題

### 1. **BoxCall命令未実装** (最高優先度)
**症状**: 基本的なBox操作が全て使用不可
```nyash
// ❌ 全て実行不可
toString()    // Box → 文字列変換
print()       // 基本出力
equals()      // 比較
clone()       // 複製
```

**エラー詳細**:
```bash
❌ WASM compilation error: Unsupported instruction: BoxCall { 
    dst: Some(ValueId(6)), 
    box_val: ValueId(4), 
    method: "toString", 
    args: [], 
    effects: EffectMask(16) 
}
```

**修正ファイル**: `src/backend/wasm/codegen.rs`

### 2. **wasmtimeバージョン互換性問題**
**症状**: AOT(.cwasm)ファイルが実行不可
```bash
Error: Module was compiled with incompatible Wasmtime version '18.0.4'
System wasmtime: 35.0.0
```

**原因**: Cargo.tomlとシステムwasmtimeの不一致
```toml
# Cargo.toml
wasmtime = "18.0"      # ← 古いバージョン

# システム
wasmtime 35.0.0        # ← 新しいバージョン
```

### 3. **WASM出力バイナリエラー**
**症状**: WAT → WASM変換でUTF-8エラー
```bash
❌ Generated WASM is not valid UTF-8
```

**推測原因**: WAT生成またはwabt crate連携の問題

## 📋 詳細実装計画

### Phase 1: 緊急復旧 (1週間)

#### Task 1.1: BoxCall命令実装 (3-4日)
**ファイル**: `src/backend/wasm/codegen.rs`

**実装アプローチ**:
```rust
fn generate_box_call(&mut self, box_call: &BoxCall) -> Result<String> {
    match box_call.method.as_str() {
        "toString" => {
            // Box → 文字列変換のWASM実装
            self.generate_to_string_call(box_call)
        }
        "print" => {
            // print関数のWASM実装 
            self.generate_print_call(box_call)
        }
        "equals" => {
            // 比較処理のWASM実装
            self.generate_equals_call(box_call)
        }
        "clone" => {
            // クローン処理のWASM実装
            self.generate_clone_call(box_call)
        }
        _ => Err(format!("Unsupported BoxCall method: {}", box_call.method))
    }
}

fn generate_to_string_call(&mut self, box_call: &BoxCall) -> Result<String> {
    // 1. Box型判定
    // 2. 型に応じた文字列変換
    // 3. StringBox作成・返却
    Ok(format!(r#"
        ;; toString() implementation
        (local.get ${})
        (call $box_to_string)
        (local.set ${})
    "#, 
    self.get_value_local(box_call.box_val),
    self.get_value_local(box_call.dst.unwrap())
    ))
}
```

**テストケース**:
```nyash
// test_boxcall_basic.nyash
local num = 42
local str = num.toString()
print(str)
print("Expected: 42")
```

#### Task 1.2: wasmtimeバージョン統一 (1日)
**修正**: `Cargo.toml`
```toml
# 変更前
wasmtime = "18.0"

# 変更後  
wasmtime = "35.0.0"
```

**互換性確認**:
```bash
# システムバージョン確認
wasmtime --version

# Cargoバージョン確認
cargo tree | grep wasmtime

# 実行テスト
./target/release/nyash --aot test_simple.nyash
wasmtime --allow-precompiled test_simple.cwasm
```

#### Task 1.3: WASM出力エラー修正 (2日)
**対象**: `src/backend/wasm/codegen.rs` WAT生成部分

**デバッグ手順**:
1. WAT出力の文字エンコーディング確認
2. wabt crate APIの正しい使用方法確認
3. バイナリ変換パイプラインの検証

**修正例**:
```rust
// WAT → WASM変換の修正
fn wat_to_wasm(&self, wat_source: &str) -> Result<Vec<u8>> {
    // UTF-8検証を追加
    if !wat_source.is_ascii() {
        return Err("WAT source contains non-ASCII characters".into());
    }
    
    // wabt crate使用方法の修正
    let wasm_bytes = wabt::wat2wasm(wat_source.as_bytes())?;
    Ok(wasm_bytes)
}
```

### Phase 2: 機能拡充 (1週間)

#### Task 2.1: RuntimeImports完全実装 (3日)
**ファイル**: `src/backend/wasm/runtime.rs`

**未実装機能**:
- Box メモリ管理 (malloc, free)
- 型キャスト・変換  
- 配列・Map操作
- 例外ハンドリング

#### Task 2.2: メモリ管理改善 (2日)
**ファイル**: `src/backend/wasm/memory.rs`

**最適化項目**:
- Box ヘッダーサイズ最適化
- メモリレイアウト効率化
- 基本的なガベージコレクション

#### Task 2.3: 統合テスト・検証 (2日)
**テストスイート**:
```bash
# 基本機能テスト
./target/release/nyash --compile-wasm test_boxcall.nyash
./target/release/nyash --compile-wasm test_basic_io.nyash

# AOTテスト
./target/release/nyash --aot test_comprehensive.nyash
wasmtime test_comprehensive.cwasm

# 互換性テスト
./scripts/test_wasm_compatibility.sh
```

## 🎯 成功基準・検証方法

### Phase 1完了時
- [ ] `toString()` がWASMで正常動作
- [ ] `print()` による出力が成功
- [ ] AOT(.cwasm)ファイルが実行可能
- [ ] WASM出力エラーが解消

### Phase 2完了時
- [ ] 全基本BoxCall命令が動作
- [ ] メモリ管理が安定動作
- [ ] 統合テストが全て成功
- [ ] 実用的なNyashプログラムがWASMで実行可能

### 検証用プログラム
```nyash
// test_wasm_recovery.nyash - 復旧確認用
static box Main {
    main() {
        local console = new ConsoleBox()
        console.log("🎉 WASM復旧テスト開始")
        
        // 基本型テスト
        local num = 42
        local str = num.toString()
        console.log("数値→文字列: " + str)
        
        // Box操作テスト
        local arr = new ArrayBox()
        arr.push("Hello")
        arr.push("WASM")
        console.log("配列長: " + arr.length().toString())
        
        console.log("✅ WASM復旧完了！")
        return "success"
    }
}
```

## 📊 リスク分析・対策

### 高リスク
- **BoxCall実装複雑化**: 段階的実装で複雑性管理
- **wasmtime API変更**: 公式ドキュメント参照、互換性テスト

### 中リスク  
- **メモリ管理不具合**: 小規模テストから開始
- **パフォーマンス劣化**: ベンチマーク継続測定

### 対策
- **毎日ビルドチェック**: `cargo check` で早期発見
- **段階的リリース**: 小さな修正を積み重ね
- **後戻り計画**: Git branchで安全な実験環境

## 🔗 関連ドキュメント
- `docs/予定/wasm/current_issues.md` - 問題詳細分析
- `docs/説明書/reference/box-design/ffi-abi-specification.md` - 将来のAPI拡張仕様
- `src/backend/wasm/` - WASM実装ソースコード
- `tests/wasm/` - WASMテストケース

---

**目標**: Phase 1完了でWASM基本機能復旧、Nyash WASMが実用レベルに到達
**期限**: 2週間以内（Phase 1: 1週間、Phase 2: 1週間）
**責任者**: Copilot (Claude協力)
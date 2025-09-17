# 🚀 Phase 9: AOT WASM実装（最優先）

## 📋 Summary
wasmtime compileによるAOT実行ファイル生成で確実なユーザー価値提供。既存WASM基盤を活用し、配布可能なネイティブ実行ファイルを短期間で実現する。

## 🎯 実装目標
```bash
# 目標実装
nyash --compile-native app.nyash -o app.exe    # AOT実行ファイル生成
nyash --aot app.nyash                          # 短縮形
./app.exe                                       # 起動高速化（JIT起動コスト除去）

# 内部実装
wasmtime compile app.wasm -o app.cwasm         # 事前コンパイル
Module::deserialize_file("app.cwasm")          # ランタイム読み込み
```

## 🔧 技術的実装詳細

### 1. wasmtime::Config統一実装
```rust
// 追加予定: src/backend/aot/mod.rs
pub struct AOTBackend {
    config: wasmtime::Config,
    engine: wasmtime::Engine,
}

impl AOTBackend {
    pub fn compile_module(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>, String> {
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)?;
        module.serialize()
    }
    
    pub fn load_precompiled(&self, cwasm_bytes: &[u8]) -> Result<wasmtime::Module, String> {
        unsafe { wasmtime::Module::deserialize(&self.engine, cwasm_bytes) }
    }
}
```

### 2. CLI統合実装
```rust
// 追加予定: src/main.rs
#[derive(Parser)]
struct Args {
    // 既存オプション...
    
    /// Compile to native executable (AOT)
    #[arg(long)]
    compile_native: bool,
    
    /// AOT compilation (short form)
    #[arg(long)]
    aot: bool,
    
    /// Output file for AOT compilation
    #[arg(short, long)]
    output: Option<String>,
}
```

### 3. 単一バイナリ梱包
```rust
// 生成例: target/release/nyash_app.exe
// 内部構造:
// [nyash_runtime] + [app.cwasm (embedded)] + [metadata]

const EMBEDDED_CWASM: &[u8] = include_bytes!("app.cwasm");

fn main() {
    let engine = wasmtime::Engine::default();
    let module = unsafe { wasmtime::Module::deserialize(&engine, EMBEDDED_CWASM) }?;
    // ... 実行
}
```

## 📊 パフォーマンス目標

| 指標 | 現在 | 目標 | 改善率 |
|------|------|------|--------|
| **実行性能** | WASM JIT: 8.12ms | AOT: <1.6ms | **5倍高速化** |
| **起動時間** | JIT起動: ~50ms | AOT起動: <10ms | **5倍高速化** |
| **配布サイズ** | Runtime必要 | 単一実行ファイル | **依存関係解消** |
| **総合改善** | 13.5倍（対Interpreter） | **500倍目標** | **37倍追加向上** |

## 🛠️ 実装ステップ（2-3週間）

### Week 1: AOT基盤実装
- [ ] `src/backend/aot/mod.rs` 基本構造
- [ ] wasmtime::Config最適化設定
- [ ] .cwasm生成・ロードパイプライン
- [ ] `--compile-native` CLI基本実装

### Week 2: パッケージング・最適化
- [ ] 単一バイナリ梱包（`include_bytes!`）
- [ ] 互換性キー管理（CPU機能・wasmtimeバージョン）
- [ ] 起動時間最適化
- [ ] エラーハンドリング・デバッグ情報

### Week 3: 統合・検証
- [ ] 既存テストスイートでの動作確認
- [ ] ベンチマーク拡張（AOT性能測定）
- [ ] ドキュメント更新
- [ ] CI統合（自動AOTビルド）

## 🔍 技術的課題と対策

### 互換性管理
**課題**: wasmtimeバージョンアップで.cwasm互換性切れ
**対策**: 
- 互換性キー埋め込み（wasmtimeバージョン・CPUフラグ）
- graceful degradation（互換切れ時はJITフォールバック）

### CPU機能検出
**課題**: SIMD/CPU拡張でベンチマーク結果変動
**対策**:
- baseline/v3二段ビルド
- 実行時CPU検出で最適.cwasm選択

### デバッグ情報
**課題**: AOTで元コード位置特定困難
**対策**:
- `Config::debug_info(true)`設定
- ソースマップ埋め込み

## ✅ Acceptance Criteria

### 機能要件
- [ ] `nyash --compile-native app.nyash -o app.exe` 動作
- [ ] 生成実行ファイルが単独で動作（依存関係なし）
- [ ] 既存Nyashプログラムが100%互換で高速実行

### 性能要件
- [ ] 起動時間 < 100ms
- [ ] 実行性能 > 現在WASM JIT（8.12ms）
- [ ] 配布ファイルサイズ < 10MB

### 品質要件
- [ ] 全テストケースPASS
- [ ] エラーハンドリング適切
- [ ] CI自動テスト通過

## 🚀 期待される効果

### 即座実用価値
- **配布可能実行ファイル**: `app.exe`単体で動作
- **起動高速化**: JIT起動コスト除去
- **依存関係解消**: wasmtimeランタイム不要

### 差別化優位
- **Everything is Box**: ネイティブAOT実現
- **Web互換性**: WASM基盤活用
- **段階最適化**: JIT→AOTの技術蓄積

### LLVM準備
- **AOT基盤確立**: Phase 10での技術転用
- **最適化知見**: エスケープ解析・ボックス化解除準備
- **ベンチマーク基準**: 真の性能比較基盤

## 📖 References
- docs/予定/native-plan/copilot_issues.txt（Phase 9詳細）
- docs/予定/ai_conference_native_compilation_20250814.md（AI大会議決定）
- docs/execution-backends.md（WASM基盤情報）
- [wasmtime compile documentation](https://docs.wasmtime.dev/cli-cache.html)

---

**💡 Tip**: 短期間で確実な成果を目指し、複雑な最適化より実用価値を最優先にする戦略です。

最終更新: 2025-08-14
作成者: Claude（実用優先戦略）
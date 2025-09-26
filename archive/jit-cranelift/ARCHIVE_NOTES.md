# JIT/Cranelift アーカイブ - Phase 15

## 📅 アーカイブ日時
2025-09-23 - Phase 15 セルフホスティング集中開発期間中

## 🎯 アーカイブ理由

### 現在の開発焦点
- **Phase 15**: Nyashセルフホスティング（80k→20k行革命）
- **開発対象**: PyVMとLLVMバックエンドのみ
- **JIT/Cranelift**: 現在未使用、ビルドエラーや混乱の原因

### 具体的な問題
1. **ビルドエラー**: `cranelift-jit` フィーチャーでのビルド失敗
2. **AI開発者の混乱**: JSON開発Claude Codeが誤ってJITルートを参照
3. **リソース分散**: メンテナンスコストが高い
4. **Phase 15集中**: PyVM/LLVM開発に集中したい

## 📂 アーカイブ内容

### 移動されたディレクトリ・ファイル
```
archive/jit-cranelift/
├── src/
│   ├── jit/                           # JIT実装コア（ABI、エンジン、ローワリング等）
│   └── backend/cranelift/             # Craneliftバックエンド実装
├── scripts/
│   └── build_jit.sh                   # JITビルドスクリプト
└── tools/
    ├── jit_compare_smoke.sh           # JIT比較スモークテスト
    └── smokes/
        ├── jit_smoke.sh               # JITスモークテスト
        └── smoke_vm_jit.sh             # VM-JIT比較テスト
```

### Cargo.toml変更
- `cranelift-jit` フィーチャーをコメントアウト
- JIT関連の依存関係を無効化

## 🔄 復活手順（将来用）

### 1. ファイル復元
```bash
# アーカイブから復元
mv archive/jit-cranelift/src/jit src/
mv archive/jit-cranelift/src/backend/cranelift src/backend/
mv archive/jit-cranelift/scripts/build_jit.sh .
mv archive/jit-cranelift/tools/* tools/
```

### 2. Cargo.toml復元
```toml
[features]
cranelift-jit = ["dep:cranelift", "dep:cranelift-jit", "dep:cranelift-module"]

[dependencies]
cranelift = { version = "0.103", optional = true }
cranelift-jit = { version = "0.103", optional = true }
cranelift-module = { version = "0.103", optional = true }
```

### 3. ビルド確認
```bash
# JITビルドテスト
cargo build --release --features cranelift-jit

# スモークテスト実行
./tools/jit_smoke.sh
```

### 4. 統合テスト
- PyVM vs JIT性能比較
- LLVM vs JIT出力比較
- 全バックエンド統合テスト

## 💡 設計ノート

### JIT実装の特徴
- **Cranelift統合**: Wasmtime/Craneliftエコシステム活用
- **ホストコール最適化**: Rustネイティブ関数との高速ブリッジ
- **メモリ管理**: GCとJITの協調動作
- **デバッグ支援**: JIT統計・トレース機能

### 将来的な価値
- **高速実行**: 本格運用時の性能向上
- **AOTコンパイル**: ネイティブバイナリ生成
- **WebAssembly統合**: WASM実行環境との統一

## 📋 関連Issue・PR
- JIT/Craneliftビルドエラー修正が必要
- AIエージェント向けドキュメント整備
- Phase 15完了後の復活検討

---

**Note**: このアーカイブは一時的な措置です。Phase 15完了後、JIT/Craneliftの復活と最新化を検討します。
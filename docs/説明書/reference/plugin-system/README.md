# Nyash Plugin System Documentation

## 🎯 Quick Start

**For new developers**: Start with [BID-FFI v1 実装仕様書](./bid-ffi-v1-actual-specification.md)

## 📚 Documentation Index

### 🟢 **Current & Accurate**
- **[bid-ffi-v1-actual-specification.md](./bid-ffi-v1-actual-specification.md)** - **主要仕様書**
  - 実際に動作している実装をベースとした正確な仕様
  - FileBoxプラグインで実証済み
  - プラグイン開発者はここから始める

- **[plugin-tester.md](./plugin-tester.md)** - プラグイン診断ツール
  - プラグインの動作確認とデバッグに使用
  - `tools/plugin-tester`ツールの使用方法

- **[filebox-bid-mapping.md](./filebox-bid-mapping.md)** - 参考資料
  - FileBox APIとプラグイン実装の対応表
  - API設計の参考として有用

### 🔄 **Migration & Reference**
- **[migration-guide.md](./migration-guide.md)** - 移行ガイド
  - 古いドキュメントから現在の実装への移行方法
  - ドキュメント状況の整理

### ⚠️ **Deprecated - 非推奨**
- **[ffi-abi-specification.md](./ffi-abi-specification.md)** - ❌ 理想案、未実装
- **[plugin-system.md](./plugin-system.md)** - ❌ 将来構想
- **[nyash-toml-v2-spec.md](./nyash-toml-v2-spec.md)** - ⚠️ 部分的に古い

## 🚀 For Plugin Developers

### 1. **Read the Specification**
```bash
# 主要仕様書を読む
cat docs/説明書/reference/plugin-system/bid-ffi-v1-actual-specification.md
```

### 2. **Study Working Example**
```bash
# FileBoxプラグインを参考にする
cd plugins/nyash-filebox-plugin
cat src/lib.rs
```

### 3. **Configure Your Plugin**
```bash
# nyash.tomlで設定
cat nyash.toml  # 実際の設定形式を確認
```

### 4. **Test Your Plugin**
```bash
# プラグインテスターで確認
cd tools/plugin-tester
cargo build --release
./target/release/plugin-tester check path/to/your/plugin.so
```

## 🔧 For Nyash Core Developers

### Implementation Files
- **[plugin_loader_v2.rs](../../../../src/runtime/plugin_loader_v2.rs)** - プラグインローダー実装
- **[nyash_toml_v2.rs](../../../../src/config/nyash_toml_v2.rs)** - 設定パーサー
- **[tlv.rs](../../../../src/bid/tlv.rs)** - TLVエンコーダー/デコーダー

### Next Steps
- **Phase 3**: MIR ExternCall → plugin system 接続実装
- **Future**: HTTP系ボックスのプラグイン化

## 📞 Support & Issues

- **Working Examples**: `plugins/nyash-filebox-plugin/`
- **Issues**: Report at [GitHub Issues](https://github.com/moe-charm/nyash/issues)
- **Configuration**: `nyash.toml` in project root

---

**Status**: Phase 2 Documentation Reorganization - Completed  
**Last Updated**: 2025-08-20
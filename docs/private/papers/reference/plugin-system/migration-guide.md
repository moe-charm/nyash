# Plugin Documentation Migration Guide

## 🎯 概要

このガイドは、Nyashプラグインシステムの古いドキュメントから実際の実装に移行するためのものです。

## 📚 Documentation Status

### ✅ **Current Working Specification**
- **[BID-FFI v1 実装仕様書](./bid-ffi-v1-actual-specification.md)** - **RECOMMENDED**
  - 実際に動作している実装をベースとした正確な仕様
  - FileBoxプラグインで実証済み
  - `plugin_loader_v2.rs`の実装に基づく

### ⚠️ **Deprecated Documentation**
- **[ffi-abi-specification.md](./ffi-abi-specification.md)** - ❌ DEPRECATED
  - 理想的な設計案だが未実装
  - MIR ExternCall設計が含まれているが、実際には使われていない
  
- **[plugin-system.md](./plugin-system.md)** - ❌ DEPRECATED  
  - YAML DSLを使った将来構想
  - 現在の実装とは大きく異なる

- **[nyash-toml-v2-spec.md](./nyash-toml-v2-spec.md)** - ⚠️ PARTIALLY OUTDATED
  - 基本構造は正しいが、実際の形式と部分的に異なる

### ✅ **Still Accurate Documentation**
- **[plugin-tester.md](./plugin-tester.md)** - ✅ CURRENT
  - プラグイン診断ツールの使用方法
  - 実際のツールと一致
  
- **[filebox-bid-mapping.md](./filebox-bid-mapping.md)** - ✅ USEFUL REFERENCE
  - FileBox APIとプラグイン実装の対応表
  - 開発時の参考資料として有効

## 🔄 Migration Steps

### For Plugin Developers

1. **Start with**: [BID-FFI v1 実装仕様書](./bid-ffi-v1-actual-specification.md)
2. **Refer to**: [実際のnyash.toml](../../../../nyash.toml) for configuration format
3. **Use**: [plugin-tester](../../../../tools/plugin-tester/) for testing
4. **Study**: [FileBox plugin](../../../../plugins/nyash-filebox-plugin/) as reference implementation

### For Nyash Core Developers

1. **Phase 1**: ✅ COMPLETED - Documentation cleanup with deprecation notices
2. **Phase 2**: ✅ COMPLETED - Accurate specification creation
3. **Phase 3**: 🚧 TODO - MIR ExternCall implementation to connect with plugin system

## 🎯 Key Differences

### Old Documentation vs Reality

| Aspect | Old Docs | Reality |
|--------|----------|---------|
| Configuration | YAML DSL | TOML format |
| API Design | Complex handle system | Simple TLV + method_id |
| MIR Integration | Fully designed | Stub only |
| ABI Version | Multiple versions | BID-FFI v1 only |

### Working Configuration Format

**Old (in deprecated docs)**:
```yaml
# filebox.plugin.yaml
schema: 1
apis:
  - sig: "FileBox::open(path: string) -> FileBox"
```

**Current (actual)**:
```toml
[libraries."libnyash_filebox_plugin.so"]
boxes = ["FileBox"]
path = "./plugins/nyash-filebox-plugin/target/release/libnyash_filebox_plugin.so"

[libraries."libnyash_filebox_plugin.so".FileBox.methods]
birth = { method_id = 0 }
open = { method_id = 1 }
```

## 📞 FFI Interface

**Old (complex)**:
- Multiple entry points
- Complex handle management
- Dynamic type discovery

**Current (simple)**:
- Single entry point: `nyash_plugin_invoke`
- Fixed TLV protocol
- Static configuration in nyash.toml

## 🚀 Next Steps

1. ✅ **Documentation Cleanup**: Completed
2. 🚧 **MIR Integration**: Implement ExternCall → plugin system connection
3. 🔮 **Future**: Consider implementing some ideas from deprecated docs

---

**Last Updated**: 2025-08-20  
**Status**: Documentation reorganization Phase 2 completed
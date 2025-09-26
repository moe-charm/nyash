# Nyash Plugin System Documentation

## 🎯 Quick Start

**For new developers**: Start with [BID-FFI v1 実装仕様書](./bid-ffi-v1-actual-specification.md)

## 📚 Documentation Index

### 🟢 **Current & Accurate**
- **[bid-ffi-v1-actual-specification.md](./bid-ffi-v1-actual-specification.md)** - **主要仕様書**
  - 実際に動作している実装をベースとした正確な仕様
  - FileBoxプラグインで実証済み
  - プラグイン開発者はここから始める

- **[../architecture/dynamic-plugin-flow.md](../architecture/dynamic-plugin-flow.md)** - **動的プラグインシステムの全体フロー** 🆕
  - MIR→VM→Registry→プラグインの動的解決フロー
  - コンパイル時決め打ちなし、実行時動的判定の仕組み
  - nyash.tomlによる透過的な切り替え

- **[vm-plugin-integration.md](./vm-plugin-integration.md)** - **VM統合仕様書** 🆕
  - VMバックエンドとプラグインシステムの統合
  - BoxRef型による統一アーキテクチャ
  - パフォーマンス最適化とエラーハンドリング

- **[plugin-tester.md](./plugin-tester.md)** - プラグイン診断ツール
  - プラグインの動作確認とデバッグに使用
  - `tools/plugin-tester`ツールの使用方法

- **[plugin_lifecycle.md](./plugin_lifecycle.md)** - ライフサイクル/RAII/シングルトン/ログ
  - 共有ハンドル、scope終了時の扱い、`shutdown_plugins_v2()` の動作
  - NetPlugin（HTTP/TCP）の並列E2E時の注意点

- **[net-plugin.md](./net-plugin.md)** - Netプラグイン（HTTP/TCP PoC）
  - GET/POST、ヘッダ、Content-Length、環境変数によるログ

- **[returns-result.md](./returns-result.md)** - 可選のResultBox正規化
  - `returns_result = true` で成功/失敗を `Ok/Err` に統一（段階導入推奨）

### ⚙️ 戻り値のResult化（B案サポート）
- `nyash.toml` のメソッド定義に `returns_result = true` を付けると、
  - 成功: `Ok(value)` の `ResultBox` に包んで返す
  - 失敗（BID負エラー）: `Err(ErrorBox(message))` を返す（例外にはしない）

```toml
[libraries."libnyash_example.so".ExampleBox.methods]
dangerousOp = { method_id = 10, returns_result = true }
```

未指定の場合は従来通り（成功=生値、失敗=例外として伝播）。

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
# 新スタイル（推奨）: 中央=nyash.toml（レジストリ最小） + 各プラグイン=nyash_box.toml（仕様書）
cat nyash.toml
cat plugins/<your-plugin>/nyash_box.toml
```

中央の `nyash.toml` 例（抜粋）
```toml
[plugins]
"libnyash_filebox_plugin" = "./plugins/nyash-filebox-plugin"

[plugin_paths]
search_paths = ["./plugins/*/target/release", "./plugins/*/target/debug"]

[box_types]
FileBox = 6
```

各プラグインの `nyash_box.toml` 例（抜粋）
```toml
[box]
name = "FileBox"
version = "1.0.0"
description = "File I/O operations Box"

[provides]
boxes = ["FileBox"]

[FileBox]
type_id = 6

[FileBox.methods.open]
id = 1
args = [ { name = "path", type = "string" }, { name = "mode", type = "string", default = "r" } ]
returns = { type = "void", error = "string" }
```

ロード時は `nyash_box.toml` が優先参照され、OS差（.so/.dll/.dylib、libプリフィックス）は自動吸収されます。従来の `[libraries]` 設定も当面は後方互換で有効です。

### 4. **Test Your Plugin**
```bash
# プラグインテスターで確認
cd tools/plugin-tester
cargo build --release
./target/release/plugin-tester check path/to/your/plugin.so
```

### 5. **nyash_box.toml テンプレ & スモーク（v2）** 🆕
- テンプレート: `docs/reference/plugin-system/nyash_box.toml.template`
- スモーク実行（VM・動的プラグイン）:
```bash
tools/smokes/v2/run.sh --profile plugins
```
  - 代表ケース（Fixture/Counter/Math など）を自動検証。未配置の .so は SKIP で安全に進行
  - 事前条件: `cargo build --release` 済み。必要に応じて `tools/smokes/v2/profiles/plugins/_ensure_fixture.sh` がフィクスチャを自動構築

### 6. **プラグイン優先（ビルトイン上書き）設定** 🆕
- 既定では、ビルトインの実装が優先されます（安全第一）。
- プラグインで置き換えたい型（ConsoleBox など）がある場合は環境変数で上書き可能:
```bash
export NYASH_USE_PLUGIN_BUILTINS=1
export NYASH_PLUGIN_OVERRIDE_TYPES="ArrayBox,MapBox,ConsoleBox"
```
  - 上記により、`new ConsoleBox()` などの生成がプラグイン経路に切替わります。
  - 後方互換のため `[libraries]` にも対象プラグインを登録しておくと、解決の一貫性が高まります。

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

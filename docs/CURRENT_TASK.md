# 🎯 現在のタスク (2025-08-19 更新)

## 🔥 最優先タスク：nyash.toml v2対応

### 📍 問題の本質
**nyash.toml v2（マルチBox型）に誰も対応していない！**

1. **プラグインテスター** - 古い単一Box型前提
2. **Nyash本体のレジストリ** - 古い単一Box型前提  
3. **結果** - プラグインが正しく読み込まれない

### 🎯 正しい実装順序
1. **プラグインテスターをnyash.toml v2対応にする**
   - マルチBox型プラグイン対応
   - nyash.tomlから型情報読み取り
   
2. **プラグインテスターで動作確認**
   - FileBoxプラグインが正しく認識されるか
   - メソッド情報が正しく取得できるか
   
3. **Nyash本体のレジストリに移植**
   - プラグインテスターの実装をコピー
   - 汎用プラグインBox生成が動作

### 📝 nyash.toml v2形式（確認）
```toml
[libraries]
"libnyash_filebox_plugin.so" = {
    boxes = ["FileBox"],
    path = "./target/release/libnyash_filebox_plugin.so"
}

[libraries."libnyash_filebox_plugin.so".FileBox]
type_id = 6

[libraries."libnyash_filebox_plugin.so".FileBox.methods]
birth = { method_id = 0 }
open = { method_id = 1, args = ["path", "mode"] }
read = { method_id = 2 }
write = { method_id = 3, args = ["data"] }
close = { method_id = 4 }
fini = { method_id = 4294967295 }
```

### 🚨 現在の間違った形式
```toml
[plugins]
FileBox = "./target/release/libnyash_filebox_plugin.so"  # ← 古い形式！

[plugins.FileBox]  # ← パーサーエラーの原因
type_id = 6
```

---

## 🚀 Phase 9.75h-0: プラグインシステム完全統一（進行中）

### 進捗状況
- ✅ 設計方針決定（nyash.toml中心設計）
- ✅ FileBox決め打ちコード削除完了
- ✅ 汎用プラグインBox（GenericPluginBox）実装完了
- 🔄 **nyash.toml v2対応が必要！**

---

## ✅ 完了したタスク（要約）

### 汎用プラグインBox生成システム ✅
- `src/bid/generic_plugin_box.rs` 実装完了
- FileBox決め打ちコードを削除
- `new FileBox()`が汎用システムで動作する仕組み完成

### Phase 9.75g-0 BID-FFI Plugin System ✅
- プラグインシステム基盤完成
- plugin-tester診断ツール実装

### Phase 8.6 VM性能改善 ✅
- VM 50.94倍高速化達成！

---

## 📋 技術詳細・参考資料

### nyash.toml v2仕様
- [config/nyash_toml_v2.rs](../src/config/nyash_toml_v2.rs)
- マルチBox型プラグイン対応
- ライブラリベースの設定形式

### 開発計画
- [copilot_issues.txt](../予定/native-plan/copilot_issues.txt)

---

**最終更新**: 2025年8月19日  
**次回マイルストーン**: プラグインテスターのnyash.toml v2対応
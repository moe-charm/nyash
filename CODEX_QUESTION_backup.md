# Codex向け質問 - Phase 15.5後のテスト戦略

## 📋 背景

Phase 15.5でCore Box完全削除を実施し、すべてのBoxをプラグイン化しました。その結果：
- ✅ nyash.tomlのパス修正完了（13箇所）
- ✅ プラグインは正常にロード（.soファイル20個存在）
- ✅ 基本的な算術演算・制御構文は動作
- ❌ StringBox/IntegerBoxのメソッドが動作しない

## 🔍 現在の問題

### StringBoxプラグインの状況
```nyash
local s = new StringBox("Hello")  # ← オブジェクト生成OK（ハンドル返却）
print(s)                          # ← 空文字列（toString失敗）
s.length()                        # ← 0を返す（内部データなし）
s.toString()                      # ← 空文字列を返す
s.get()                          # ← 空文字列を返す
```

### 調査済み事項
1. プラグインは正常にロード（plugin-testerで確認）
2. nyash_plugin_invokeは実装済み（legacy v1 ABI）
3. method_id衝突を修正済み（0-3 → 4+に変更）
4. 通常の文字列リテラルは動作する
5. 算術演算は問題なし

### 🔬 根本原因（Codex調査結果）
**TypeBox v2のresolveブランチが欠落している**
- `birth`メソッドの解決パスが未実装
- `toString`メソッドの解決パスが未実装
- プラグインメソッドは呼ばれるが、結果の処理に問題

## 🎯 質問

### 1. **StringBoxメソッドが動作しない原因は？**
Phase 15.5でCore Boxを削除した影響で、プラグイン側の実装が不完全な可能性があります。
- プラグインのnyash_plugin_invoke実装を確認すべき箇所は？
- MIRビルダー側でプラグインメソッド呼び出しに特別な処理が必要？

### 2. **テスト戦略の方向性**
現状でStringBox/IntegerBoxが動作しない中で：
- A案: プラグインメソッド修正を優先
- B案: 基本機能（算術・制御）のテストを先に充実
- C案: 別のBoxプラグイン（FileBox等）でテスト

どの方向性が効率的でしょうか？

### 3. **プラグインメソッド呼び出しのデバッグ方法**
```bash
# 現在の確認方法
./tools/plugin-tester/target/release/plugin-tester check --config nyash.toml
# → プラグインロードはOK、でもメソッド実行時に問題

# より詳細なデバッグ方法は？
```

## 🔄 再現手順

### 最小再現コード
```bash
# test_stringbox.nyash
local s = new StringBox("Hello World")
print("StringBox created")
print(s)  # 期待: "Hello World", 実際: ""
local len = s.length()
print("Length: " + len)  # 期待: 11, 実際: 0
```

### 実行コマンド
```bash
# プラグインロード確認
./tools/plugin-tester/target/release/plugin-tester check --config nyash.toml

# テスト実行
NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 ./target/release/nyash test_stringbox.nyash
```

### デバッグ情報収集
```bash
# 詳細ログ
NYASH_CLI_VERBOSE=1 ./target/release/nyash test_stringbox.nyash

# MIRダンプ確認
./target/release/nyash --dump-mir test_stringbox.nyash
```

## 📁 関連ファイル

- `nyash.toml` - プラグイン設定（method_id修正済み）
- `plugins/nyash-string-plugin/src/lib.rs` - StringBoxプラグイン実装
- `tools/smokes/v2/` - 新スモークテストシステム
- `src/box_factory/plugin.rs` - プラグインロード実装
- `src/mir/builder/builder_calls.rs` - TypeBox v2 resolve実装（問題箇所）

## 🚀 期待する回答

1. StringBoxメソッドが動作しない根本原因の特定方法
2. 効率的なテスト戦略の提案
3. プラグインメソッド呼び出しのデバッグ手法

よろしくお願いします！
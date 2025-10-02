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

### 🔬 根本原因（Codex調査結果 - 2025-09-24）

**実装レベルの具体的問題箇所を特定済み：**

1. **`string_invoke_id`に`M_BIRTH`分岐がない**
   - `plugins/nyash-string-plugin/src/lib.rs`の`string_invoke_id`にM_BIRTH分岐が無く
   - `new StringBox("test")`で生成されたIDが`INST`マップに登録されない
   - `.length()`呼び出しで`E_HANDLE`が返る

2. **`string_resolve`がtoStringを未マッピング**
   - 同ファイルの`string_resolve`が`"toString"`を`M_TO_UTF8`にマッピングしていない
   - `.toString()`は未知メソッド扱いになり空文字列/エラーでフォールバック

3. **IntegerBoxも同様の問題**
   - `plugins/nyash-integer-plugin/src/lib.rs`でも`M_BIRTH`/`M_FINI`が未実装
   - 値を保持できず`.get()`/`.set()`が失敗

## 🎯 質問

### 1. **実装修正の優先度は？**
- `string_invoke_id`と`integer_invoke_id`に`M_BIRTH`/`M_FINI`分岐を復元するのが最優先か？
- それともTypeBox共通レイヤーでフォールバック処理を追加すべきか？

### 2. **toStringメソッドの実装方針**
- `.toString()`は`toUtf8`のエイリアスにすべきか？
- 新たなメソッドIDを`nyash_box.toml`へ追加してVMに通知すべきか？

### 3. **テスト戦略の方向性**
現状でStringBox/IntegerBoxが動作しない中で：
- A案: プラグインメソッド修正を優先（M_BIRTH実装）
- B案: 基本機能（算術・制御）のテストを先に充実
- C案: 別のBoxプラグイン（FileBox等）でテスト

どの方向性が効率的でしょうか？

### 4. **既存テストの扱い**
- `tools/smokes/v2/profiles/quick/boxes`のStringBoxケースを一時的に外すか？
- 失敗を許容したまま調査用に残すか？

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
NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 ./target/release/hakorune test_stringbox.nyash
```

### デバッグ情報収集
```bash
# 詳細ログ
NYASH_CLI_VERBOSE=1 ./target/release/hakorune test_stringbox.nyash

# MIRダンプ確認
./target/release/hakorune --dump-mir test_stringbox.nyash

# 具体的な問題箇所の確認
rg "M_BIRTH" plugins/nyash-string-plugin/src/lib.rs  # 該当箇所を特定
```

## 📁 関連ファイル

- `nyash.toml` - プラグイン設定（method_id修正済み）
- `plugins/nyash-string-plugin/src/lib.rs` - StringBoxプラグイン実装（L23-L113, L205-L280）
- `plugins/nyash-integer-plugin/src/lib.rs` - IntegerBoxプラグイン実装
- `tools/smokes/v2/` - 新スモークテストシステム
- `src/box_factory/plugin.rs` - プラグインロード実装
- `src/runtime/plugin_loader_v2/enabled/loader.rs` - create_box → nyash_plugin_invoke_v2_shim
- `src/mir/builder/builder_calls.rs` - TypeBox v2 resolve実装（問題箇所）

## 🚀 期待する回答

1. M_BIRTH/M_FINI実装の具体的な修正方法
2. 効率的なテスト戦略の提案
3. プラグインメソッド呼び出しのデバッグ手法

よろしくお願いします！

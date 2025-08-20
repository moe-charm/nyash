# 🔄 ビルトインBox → プラグイン変換手順書

## 🎯 概要

ビルトインBoxをBID-FFI v1プラグインに変換する標準手順。実際の変換作業で発見された問題と解決策を蓄積し、効率的な開発手法を確立する。

## 📊 変換パターン分析

### 🏆 成功事例：FileBox変換
- **元実装**: `src/boxes/file/mod.rs` (RwLock<File>)
- **プラグイン**: `plugins/nyash-filebox-plugin/` (BID-FFI v1)
- **結果**: ✅ 完全動作、プラグイン優先使用

### 🔍 現状分析：HTTP系Box
- **実装状況**: 完全実装済み（432行の高機能HTTPサーバー）
- **問題**: Unified Registry未登録（Legacy Match使用）
- **潜在性**: 即座にプラグイン化可能

## 🚀 標準変換手順（3段階アプローチ）

### Phase 1: ビルトイン最適化
**目的**: 既存実装の性能向上・デバッグ
**期間**: 1-3日

#### 手順
1. **Unified Registry登録**
   ```rust
   // src/box_factory/builtin.rs 内
   fn register_io_types(&mut self) {
       // HTTPServerBox追加
       self.register("HTTPServerBox", |args| {
           if !args.is_empty() {
               return Err(RuntimeError::InvalidOperation {
                   message: format!("HTTPServerBox constructor expects 0 arguments, got {}", args.len()),
               });
           }
           Ok(Box::new(HTTPServerBox::new()))
       });
       // 他のHTTP系Boxも同様に追加
   }
   ```

2. **動作テスト作成**
   ```nyash
   // local_tests/test_http_builtin.nyash
   static box Main {
       main() {
           local server = new HTTPServerBox()
           server.bind("localhost", 8080)
           server.get("/test", TestHandler.handle)
           return "HTTP builtin test complete"
       }
   }
   ```

3. **性能ベンチマーク**
   - Legacy Match vs Unified Registry比較
   - メモリ使用量測定

#### 期待効果
- ✅ 高速化（Legacy Match削除）
- ✅ デバッグ環境確立
- ✅ 安定性確認

### Phase 2: プラグイン変換実装
**目的**: BID-FFI v1プラグイン実装
**期間**: 3-7日

#### 手順
1. **プラグインプロジェクト作成**
   ```bash
   mkdir plugins/nyash-http-plugin
   cd plugins/nyash-http-plugin
   cargo init --lib
   ```

2. **Cargo.toml設定**
   ```toml
   [lib]
   crate-type = ["cdylib"]
   
   [dependencies]
   once_cell = "1.0"
   # HTTP依存関係
   ```

3. **BID-FFI v1実装**
   - マルチBox対応（HTTPServerBox, HTTPClientBox, SocketBox）
   - TLV Protocol実装
   - Method ID定義

4. **nyash.toml設定**
   ```toml
   [libraries."libnyash_http_plugin.so"]
   boxes = ["HTTPServerBox", "HTTPClientBox", "SocketBox"]
   
   [libraries."libnyash_http_plugin.so".HTTPServerBox]
   type_id = 10
   [libraries."libnyash_http_plugin.so".HTTPServerBox.methods]
   birth = { method_id = 0 }
   bind = { method_id = 1, args = ["address", "port"] }
   listen = { method_id = 2, args = ["backlog"] }
   start = { method_id = 3 }
   stop = { method_id = 4 }
   fini = { method_id = 4294967295 }
   ```

### Phase 3: 移行・検証
**目的**: 完全移行とパフォーマンス検証
**期間**: 1-2日

#### 手順
1. **プラグイン優先テスト**
   - 同じテストケースでビルトイン vs プラグイン比較
   - メモリリーク検証
   - エラーハンドリング確認

2. **ビルトイン実装削除**
   - `src/boxes/http_*` ファイル削除
   - BUILTIN_BOXES リストから除去
   - コンパイル確認

3. **本格アプリテスト**
   ```nyash
   // apps/http_example/
   // 実用的なHTTPサーバーアプリで動作確認
   ```

## 🔧 BID-FFI v1必須要件

### ✅ **絶対必須の2つのメソッド**

すべてのBID-FFI v1プラグインで実装必須：

**🔧 birth() - コンストラクタ (METHOD_ID = 0)**
```rust
const METHOD_BIRTH: u32 = 0;  // Constructor
```
- **機能**: インスタンス作成、instance_id返却
- **必須実装**: インスタンス管理、メモリ確保
- **戻り値**: TLV形式のinstance_id (u32)

**🧹 fini() - デストラクタ (METHOD_ID = u32::MAX)**
```rust
const METHOD_FINI: u32 = u32::MAX;  // Destructor (4294967295)
```
- **機能**: インスタンス解放、メモリクリーンアップ
- **必須実装**: INSTANCES.remove(), リソース解放
- **戻り値**: 成功ステータス

### 📝 設定例
```toml
[libraries."libnyash_example_plugin.so".ExampleBox.methods]
birth = { method_id = 0 }                    # 🔧 必須
# ... カスタムメソッド ...
fini = { method_id = 4294967295 }           # 🧹 必須
```

## 🐛 発見済み問題と解決策

### Problem 1: toString()メソッドエラー
**現象**: `Unknown method 'toString' for FileBox`
```
❌ Interpreter error: Invalid operation: Unknown method 'toString' for FileBox
```

**原因**: プラグインにtoString()メソッド未定義
**解決策**: nyash.tomlでtoStringメソッド追加
```toml
toString = { method_id = 5 }
```

### Problem 2: Unified Registry未登録Box
**現象**: `Falling back to legacy match statement`
```
🔍 Unified registry failed for HTTPServerBox: Unknown Box type
🔍 Falling back to legacy match statement
```

**原因**: BuiltinBoxFactory.register_io_types()未登録
**解決策**: HTTP系Box登録追加

### Problem 3: 複雑な依存関係
**予想問題**: HTTPServerBox → SocketBox → OS固有API
**解決策**: プラグイン内で依存関係完結

## 📋 チェックリスト

### ✅ Phase 1完了条件
- [ ] Unified Registry登録完了
- [ ] Legacy Match削除確認
- [ ] 基本動作テスト成功
- [ ] パフォーマンス改善確認

### ✅ Phase 2完了条件
- [ ] プラグインビルド成功
- [ ] BID-FFI v1インターフェース実装
- [ ] 全メソッドTLV対応
- [ ] plugin-testerで検証成功

### ✅ Phase 3完了条件
- [ ] プラグイン優先動作確認
- [ ] ビルトイン実装削除成功
- [ ] 実用アプリケーション動作確認
- [ ] メモリリーク・エラーなし

## 🚀 期待効果

### 短期効果（Phase 1）
- **5-10倍高速化**: Legacy Match → Unified Registry
- **保守性向上**: 統一的なファクトリパターン
- **デバッグ環境**: 安定したテスト基盤

### 長期効果（Phase 3）
- **プラグイン化完了**: 外部配布可能
- **アーキテクチャ改善**: コア軽量化
- **拡張性向上**: 独立開発可能

## 🎯 次期対象Box候補

### 優先度高（実装済み）
1. **HTTP系**: HTTPServerBox, HTTPClientBox, SocketBox
2. **BufferBox**: バイナリデータ処理
3. **RegexBox**: 正規表現処理

### 優先度中（要調査）
1. **MathBox, RandomBox**: プラグイン実装あり（第1世代C ABI）
2. **JSONBox**: データ交換
3. **StreamBox**: ストリーム処理

## 📝 学習記録

### 成功パターン
- FileBox: 単純構造、明確API → スムーズ変換
- プラグイン優先システム動作確認済み

### 注意点
- toString()等の基本メソッド必須
- 依存関係の循環に注意
- メモリ管理の完全分離

---

**最終更新**: 2025年8月20日 - 初版作成  
**Phase**: 9.75g-0 完了後 - HTTP系Box変換準備完了  
**Next**: Phase 1実装→Phase 2プラグイン化
# 🎯 現在のタスク (2025-08-19 更新)

## 🚧 作業中: Phase 9.78e 動的メソッドディスパッチ統合

### 🎯 **Phase 9.78e: call_method実装と型変換問題**
**状況**: 基本実装完了、型変換とインスタンス混在で複雑化

#### ✅ **完了事項**
- ✅ NyashBoxトレイトにcall_method追加
- ✅ StringBoxでcall_method実装（全メソッド対応）
- ✅ InstanceBoxでデリゲーションパターン実装
- ✅ RuntimeErrorに必要なバリアント追加

#### 🚧 **課題**
- ❌ **2つのInstanceBox実装の混在問題**
  - 古い`instance.rs`と新しい`instance_v2.rs`が並存
  - メソッドシグネチャの不一致（`set_field`等）
  - 型変換の複雑化（Box ↔ Arc<Mutex> ↔ NyashValue）

### 🔧 **新戦略: instance_v2を主体とした段階的移行**
**方針**: instance_v2.rsに旧instance.rsの機能を内包（上からのフロー）

1. **Phase 1**: instance_v2にレガシー互換レイヤー追加 ✅
   - レガシーフィールド（fields, weak_fields_union等）を追加
   - 互換メソッド実装（get_field_legacy, set_field_legacy等）
   - ビルドエラー解消

2. **Phase 2**: 型変換の実装 🚧
   - **TODO**: SharedNyashBox → NyashValue の適切な変換実装
   - 現在は一時的にNullを設定（instance_v2.rs:218, 238）
   - Arc<dyn NyashBox> → Arc<Mutex<dyn NyashBox>> の変換方法検討

3. **Phase 3**: インタープリター移行
   - instance.rs → instance_v2.rs への参照切り替え
   - テストによる動作確認

### ⚠️ **次のアクション**
1. ✅ Git変更を一旦リセット
2. ✅ ビルドが通る状態を確認  
3. ✅ instance_v2にレガシー互換実装
4. 🚧 型変換の適切な実装（重要TODO）
5. インタープリターでinstance_v2使用開始

---

## ✅ 完了: Phase 9.78a-d BoxFactory革命

### 🎉 Phase 9.78d 達成結果  
**InstanceBox簡素化統一実装成功！**

#### 🏭 実装完了内容
1. **✅ Phase 9.78a: BoxFactory基盤実装**
   - 統合レジストリアーキテクチャ完成
   - 600+行match文 → 30行に削減

2. **✅ Phase 9.78b: ビルトインBox統合**  
   - 20+種類のBox型統合完了
   - **ビルド時間: 4分 → 43秒 (5.6倍高速化！)**

3. **✅ Phase 9.78c: プラグインBox統合**
   - BID-FFI Step 1-3実装成功
   - plugin-testerツール完成

4. **✅ Phase 9.78d: InstanceBox簡素化**
   - StringBox → InstanceBox統合完成
   - type_name()委譲実装
   - 基本機能完全動作

### 📊 新しいビルドコマンド
```bash
# 高速ビルド（通常開発用）: ~43秒
cargo build --release -j32

# WASM機能付きビルド（必要時のみ）: ~4分
cargo build --release -j32 --features wasm-backend
```

---

## 🎯 今後の優先事項（copilot_issues.txt参照）

### Phase 8.4: AST→MIR Lowering完全実装
- MIR命令セット設計済み（35命令）
- Lowering実装開始準備

### Phase 8.5: MIRダイエット（35→20命令）
- 命令セット最適化による性能改善

### Phase 8.6: VM性能改善（0.9倍→2倍以上）
- レジスタ割り当て最適化
- インライン展開

最終更新: 2025-08-19 - Phase 9.78e instance_v2主体の移行戦略に変更、型変換TODO追加
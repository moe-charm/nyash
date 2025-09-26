# Phase 9.78: 統合BoxFactoryアーキテクチャ実装

## 概要
Nyashの3つの異なるBox生成フロー（ビルトイン、ユーザー定義、プラグイン）を統一的なFactoryパターンで整理し、保守性・拡張性・哲学的一貫性を向上させる。

## 背景と問題点

### 現在の混沌状態
`src/interpreter/objects.rs::execute_new`内で：
- **ビルトインBox**: 600行以上の巨大match文で直接生成
- **ユーザー定義Box**: InstanceBox経由で生成  
- **プラグインBox**: BoxFactoryRegistry経由で生成

この3つが1つの関数内に混在し、800行を超える巨大な関数となっている。

### 影響
- 新しいBox追加時の変更箇所が散在
- コンフリクトの温床
- "Everything is Box"哲学が実装レベルで体現されていない
- 保守性・可読性の著しい低下

## 提案する解決策：統合BoxFactoryアーキテクチャ

### 核心設計
```rust
// 統一インターフェース
trait BoxFactory: Send + Sync {
    fn create_box(&self, name: &str, args: &[Box<dyn NyashBox>]) 
        -> Result<Box<dyn NyashBox>, RuntimeError>;
    fn is_available(&self) -> bool;
    fn box_types(&self) -> Vec<&str>;
    fn supports_birth(&self) -> bool { true }
}

// 統合レジストリ
struct UnifiedBoxRegistry {
    factories: Vec<Box<dyn BoxFactory>>,
}

// 使用時（execute_new内）
let box_instance = registry.create_box(name, args)?;
```

### 期待される効果
1. **コード削減**: 600行 → 30行程度
2. **保守性向上**: Box追加が1行の登録で完了
3. **哲学強化**: "Everything is Box"を実装レベルで体現
4. **WASM対応**: 条件付きコンパイルが簡潔に

## 実装計画

### Phase 9.78a: 基盤構築（1-2日）
- [ ] BoxFactoryトレイト定義
- [ ] UnifiedBoxRegistry実装
- [ ] 基本的なテストケース作成

### Phase 9.78b: ビルトインBox移行（2-3日）
- [ ] BuiltinBoxFactory実装
- [ ] 各Boxに`nyash_create`関数追加
- [ ] マクロによる宣言的登録システム
- [ ] フォールバック付き段階移行

### Phase 9.78c: プラグインBox統合（1日）
- [ ] PluginBoxFactory実装
- [ ] 既存BoxFactoryRegistryのラップ
- [ ] v2プラグインシステムとの整合性確認

### Phase 9.78d: ユーザー定義Box統合（1-2日）
- [ ] UserDefinedBoxFactory実装
- [ ] birth/finiライフサイクル保証
- [ ] InstanceBoxとの連携

### Phase 9.78e: クリーンアップと最適化（1日）
- [ ] 古いコード完全削除
- [ ] パフォーマンステスト
- [ ] ドキュメント更新

### Phase 9.78f: clone_box/share_box統一実装（1-2日）
- [ ] 現在の誤実装を修正（share_boxがclone_boxを呼んでいる箇所）
  - channel_box.rs: 仮実装を修正
  - plugin_box_legacy.rs: 正しいshare実装に変更
- [ ] セマンティクスの明確化
  - clone_box: 新しいインスタンス生成（深いコピー、新しいID）
  - share_box: 同じインスタンスへの参照共有（同じID、Arc参照）
- [ ] 各Box型での実装確認と統一
  - 基本型（String/Integer/Bool）: immutableなので同じ実装でOK
  - コンテナ型（Array/Map）: clone=深いコピー、share=Arc共有
  - ユーザー定義（InstanceBox）: 正しい実装の確認
  - プラグイン（FFI境界）: 特別な考慮が必要
- [ ] 包括的テストケース作成
- [ ] ドキュメント化

## 技術的詳細

### マクロによる登録簡素化
```rust
macro_rules! register_builtins {
    ($registry:expr, $($box_name:literal => $creator_fn:path),*) => {
        $(
            $registry.add_builtin($box_name, Box::new($creator_fn));
        )*
    };
}

register_builtins!(factory,
    "StringBox"  => StringBox::nyash_create,
    "IntegerBox" => IntegerBox::nyash_create,
    // ...
);
```

### birth/finiライフサイクル保証
- **birth**: BoxFactory::create_boxが担当
- **fini**: 生成されたBoxインスタンス自身の責務
- 明確な責務分離により一貫性を保証

## リスクと対策

### リスク
1. **大規模リファクタリング**: 既存コードへの影響大
2. **動的ディスパッチ**: わずかなパフォーマンス影響

### 対策
1. **段階的移行**: フォールバック付きで安全に移行
2. **包括的テスト**: 全Box型の生成テスト必須
3. **パフォーマンス測定**: 実影響を定量的に確認

## 成功基準
- [ ] execute_new関数が100行以下に削減
- [ ] 全Box型が統一インターフェースで生成可能
- [ ] 既存テストが全てパス
- [ ] パフォーマンス劣化が1%未満

## 参考資料
- [Geminiとの設計議論](/docs/archive/2025-01-gemini-unified-box-factory-design.md)
- Phase 9.75g: BID-FFI基盤（プラグインシステムv2）
- Phase 8.4: AST→MIR Lowering（優先度との調整要）

## 優先度と実施時期
- **優先度**: 高（コードベースの健全性に直結）
- **実施時期**: Phase 8.4完了後、Phase 9実装と並行可能
- **見積もり工数**: 8-12日（clone_box/share_box統一を含む）

---

*作成日: 2025年1月19日*  
*次回レビュー: Phase 8.4完了時*
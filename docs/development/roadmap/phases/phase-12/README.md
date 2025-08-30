# Phase 12: Nyashスクリプトプラグインシステム革命

## 🚀 概要

Nyashスクリプト自体でプラグインを作成できる革命的発見！ビルド不要で、既存のネイティブプラグインを組み合わせて新機能を作成可能。

## 💡 発見の経緯

include/export仕様の検討中に、以下の重要な気づきが：

```nyash
# custom_math_plugin.ny
export box CustomMathPlugin {
    init { 
        _math = new MathBox()  # 既存プラグイン活用
        _cache = new MapBox()  # 結果キャッシュ
    }
    
    // カスタム拡張
    cached_sin(x) {
        local key = x.toString()
        if me._cache.has(key) {
            return me._cache.get(key)
        }
        local result = me._math.sin(x)
        me._cache.set(key, result)
        return result
    }
}
```

これにより、Rust/C++のビルドなしでプラグイン開発が可能に！

## 🎯 統一Box ABI設計

### 基本インターフェース

```rust
// Rust側の統一インターフェース
trait BoxInterface {
    fn invoke(&self, method_id: u32, args: NyashValue) -> NyashValue;
    fn get_methods(&self) -> Vec<MethodInfo>;
    fn init(&mut self, ctx: Context);
    fn drop(&mut self);
}
```

### Nyashスクリプトプラグインの要件

```nyash
export box MyPlugin {
    // 必須：初期化
    init { ... }
    
    // 推奨：FFI互換インターフェース
    invoke(method_id, args) {
        // method_idに基づいてディスパッチ
    }
    
    // オプション：メソッド情報
    get_methods() {
        return [
            { name: "method1", id: 1 },
            { name: "method2", id: 2 }
        ]
    }
}
```

## 📊 エコシステムへの影響

### 開発の民主化
- **参入障壁の劇的低下**: Rust/C++環境不要
- **即座の開発**: ビルド待ち時間ゼロ
- **コミュニティ拡大**: より多くの開発者が参加可能

### 新しい開発パターン
1. **プラグインの合成**: 複数のネイティブプラグインを組み合わせ
2. **ラピッドプロトタイピング**: アイデアを即座に実装
3. **ホットリロード**: 実行中の更新が可能

## 🛣️ 実装ロードマップ

### Phase 12.1: 基盤構築
- [ ] Box ABI仕様の最終決定
- [ ] export box構文のパーサー実装
- [ ] 基本的なPluginRegistry実装

### Phase 12.2: 統一インターフェース
- [ ] FFIプラグインのBoxInterface対応
- [ ] NyashスクリプトのBoxInterface実装
- [ ] 相互運用テスト

### Phase 12.3: 動的機能
- [ ] 動的ロード/アンロード機能
- [ ] ホットリロード対応
- [ ] プラグイン間依存関係管理

### Phase 12.4: セキュリティと最適化
- [ ] サンドボックス実装
- [ ] ケイパビリティベース権限
- [ ] パフォーマンス最適化

## 📚 関連ドキュメント
- [Gemini先生の分析](./gemini-analysis-script-plugins.md)
- [Codex先生の技術提案](./codex-technical-proposal.md)
- [統合分析まとめ](./synthesis-script-plugin-revolution.md)

## 🎯 次のアクション
1. Box ABI仕様書の作成
2. export box構文の実装開始
3. 既存FFIプラグイン1つを統一インターフェースに移行

---
*Everything is Box - そしてプラグインもBoxになる！*
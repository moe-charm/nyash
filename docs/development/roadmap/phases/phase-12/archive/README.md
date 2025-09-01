# Phase 12: プラグインシステムの進化 - 既存C ABIとの共存戦略

## 🚀 概要

**重要**: 既存のC ABIプラグインはそのまま使い続けられます！その上で、以下の2つの新機能を追加：

1. **Nyash ABIサポート** - より型安全で拡張性の高い新しいABI（オプション）
2. **Nyashスクリプトプラグイン** - ビルド不要でプラグイン作成可能

### なぜ既存C ABIを残すのか？

- **実績と安定性**: 現在動いているFileBox、NetBox等はそのまま
- **ゼロオーバーヘッド**: 高頻度呼び出しには最速のC ABI
- **段階的移行**: 必要に応じて選択的にNyash ABIへ移行可能

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

## 🎯 ABI共存戦略 - 適材適所の選択

### 三層構造のプラグインエコシステム

```
1. C ABIプラグイン（既存・継続）
   - FileBox, NetBox, MathBox等
   - 高速・安定・実績あり
   - 変更なし！そのまま使える

2. Nyash ABIプラグイン（新規追加）
   - 型安全・拡張性重視の新規開発向け
   - async/await対応などの将来機能

3. Nyashスクリプトプラグイン（革命的）
   - ビルド不要・即座に開発
   - 上記1,2を組み合わせて利用可能
```

### 使い分けの指針

| 用途 | 推奨ABI | 理由 |
|------|---------|------|
| 数値計算（高頻度） | C ABI | ゼロオーバーヘッド |
| ファイルI/O | C ABI | 既存実装が安定 |
| 複雑な型操作 | Nyash ABI | 型安全性重視 |
| プロトタイプ | Nyashスクリプト | 即座に試せる |

### 設定例（nyash.toml v2.1）

```toml
# nyash.toml v2.1
[plugin.math]
path = "plugins/math.so"
abi = "c"  # 高速・安定（デフォルト）

[plugin.advanced_math]
path = "plugins/advanced_math.so"
abi = "nyash"  # 型安全・拡張性
```

### BoxCall拡張による実装

**重要な発見**：MIR層の変更は不要！VM実行時の型判定で十分。
**Phase 12での追加発見**：PluginInvokeも不要！BoxCallに統合可能。

```rust
// MIR層：変更なし → さらにシンプルに（14命令へ）
MirInstruction::BoxCall {
    receiver: Value,
    method: String,
    args: Vec<Value>,
}
// PluginInvoke は廃止（BoxCallに統合）

// VM層：賢い判定
fn execute_boxcall(...) {
    let type_info = get_plugin_info(receiver.type_id);
    match type_info.abi {
        "c" => call_c_abi(...),      // 既存プラグイン
        "nyash" => call_nyash_abi(...), // 新プラグイン
    }
}
```

### Nyash ABI仕様

```c
// 3×u64構造体による型安全な値渡し
typedef struct NyashValue {
    uint64_t type_id;    // 型識別子
    uint64_t box_handle; // Boxハンドル
    uint64_t metadata;   // 拡張用（async flag等）
} NyashValue;

// 統一関数シグネチャ
typedef NyashValue (*NyashFunc)(
    uint32_t argc,
    NyashValue* args,
    void* context
);
```

### 基本インターフェース（内部）

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

## 🛣️ 実装ロードマップ（2025-09-01更新）

**重要**: 既存のC ABIプラグインは一切変更不要！追加機能として実装します。

### Phase 12.1: ABI共存基盤（1週間）
- [ ] nyash.toml v2.1仕様（abi fieldサポート、デフォルト="c"）
- [ ] プラグインローダーのABI判定実装（後方互換性保証）
- [ ] VM execute_boxcallのABI分岐追加（C ABIは現状維持）
- [ ] 同一機能のC ABI / Nyash ABI比較ベンチマーク

### Phase 12.2: Nyash ABI基盤（2週間）
- [ ] NyashValue構造体定義（crates/nyrt/include/nyash_abi.h）
- [ ] pack/unpack関数実装
- [ ] 既存プラグイン1つをNyash ABI移行（実証実験）
- [ ] JIT最適化（型既知時の特化コード生成）

### Phase 12.3: スクリプトプラグイン対応（3週間）
- [ ] export box構文のパーサー実装
- [ ] BoxInterface trait実装
- [ ] NyashスクリプトのVM内実行環境
- [ ] 相互運用テスト（C/Nyash/Script混在）

### Phase 12.4: 動的機能とエコシステム（継続的）
- [ ] ホットリロード対応
- [ ] プラグイン間依存関係管理
- [ ] プラグインマーケットプレイス構想
- [ ] セキュリティサンドボックス

### 実装優先順位（短期）
1. **Week 1**: nyash.tomlのabi field + VM分岐（動作確認）
2. **Week 2**: 性能測定 + 方向性判断
3. **Week 3**: 本格実装 or 方針転換

## 📚 関連ドキュメント

### 初期設計
- [Gemini先生の分析](./gemini-analysis-script-plugins.md)
- [Codex先生の技術提案](./codex-technical-proposal.md)
- [統合分析まとめ](./synthesis-script-plugin-revolution.md)

### ABI戦略議論（2025-09-01）
- [議論まとめ](./abi-strategy-discussion/README.md)
- [Gemini先生のABI分析](./abi-strategy-discussion/gemini-abi-analysis.md)
- [Codex先生のABI実装戦略](./abi-strategy-discussion/codex-abi-implementation.md)
- [Codex先生のBoxCall拡張分析](./abi-strategy-discussion/codex-boxcall-extension-analysis.md)
- [深い分析と結論](./abi-strategy-discussion/deep-analysis-synthesis.md)
- [最終実装決定](./abi-strategy-discussion/final-implementation-decision.md)

## 🎯 次のアクション（優先順位順）
1. nyash.toml v2.1パーサーに`abi` field追加（Day 1-2）
2. VM execute_boxcallでABI判定分岐実装（Day 3-4）
3. SimpleMathプラグインで両ABI比較テスト（Day 5-6）
4. ベンチマーク結果に基づく方向性判断（Day 7）

### 将来的なアクション
- Nyash ABI仕様書（nyash_abi.h）作成
- export box構文の実装
- プラグインSDK（#[nyash_plugin]マクロ）開発

---
*Everything is Box - そしてプラグインもBoxになる！*
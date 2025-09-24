# ChatGPTへの質問：Nyash Phase 15.5 Core Box統一の完全性について

## 背景
Nyash言語のPhase 15.5で「Core Box統一」を実施し、すべてのBoxをプラグイン化することにしました。
しかし、StringBox/IntegerBoxプラグインが正しく実装されているにも関わらず動作しません。

## 調査結果
```
実行フロー：
NewBox命令 → UnifiedBoxRegistry → factories配列を順に試行

factories配列の優先順位：
1. BuiltinBoxFactory（内蔵実装）← ここでStringBoxが捕まる
2. UserBoxFactory（ユーザー定義Box）
3. PluginBoxFactory（プラグインBox）

BuiltinBoxFactoryの実装（src/box_factory/builtin.rs）：
- StringBox → 内蔵StringBoxインスタンスを返す
- IntegerBox → 内蔵IntegerBoxインスタンスを返す
- BoolBox → 内蔵BoolBoxインスタンスを返す
- ArrayBox, MapBox, ConsoleBox → 同様
```

## 問題点
1. **Phase 15.5の思想「すべてがプラグイン」と矛盾**
   - BuiltinBoxFactoryが残存し、Core Boxの内蔵実装がある
   - プラグインより内蔵が優先される設計

2. **回避策が必要**
   ```bash
   NYASH_USE_PLUGIN_BUILTINS=1 \
   NYASH_PLUGIN_OVERRIDE_TYPES="StringBox,IntegerBox" \
   ./target/release/nyash program.nyash
   ```

3. **プラグイン実装の努力が無駄に**
   - StringBoxプラグイン：M_BIRTH/M_FINI/length/toString等を正しく実装
   - IntegerBoxプラグイン：同様に実装
   - しかし内蔵版が優先されるため呼ばれない

## アーキテクチャ的な質問

### Q1: レガシー削除の判断基準
BuiltinBoxFactoryを完全削除すべきでしょうか？それとも段階的移行が必要でしょうか？

考慮点：
- テストコードが内蔵版に依存している可能性
- パフォーマンス（内蔵版の方が高速？）
- ブートストラップ問題（最小限の内蔵Boxが必要？）

### Q2: 優先順位設計の妥当性
現在：builtin > user > plugin

より適切な設計は？
- A) plugin > user > builtin（プラグイン優先）
- B) 完全フラット化（名前衝突時はエラー）
- C) 明示的な名前空間（builtin::StringBox vs plugin::StringBox）
- D) BuiltinBoxFactory削除（プラグインのみ）

### Q3: 移行戦略
安全にBuiltinBoxFactoryを削除するには？

1. **即座に削除**
   - リスク：テスト破壊、パフォーマンス劣化
   - メリット：クリーンな設計

2. **段階的削除**
   - Step 1: StringBox/IntegerBoxのみ削除
   - Step 2: BoolBox削除
   - Step 3: ArrayBox/MapBox削除
   - Step 4: ConsoleBox削除

3. **設定駆動**
   - デフォルトでプラグイン使用
   - --use-builtin フラグで内蔵版使用

### Q4: プラグイン不在時の扱い
もしStringBoxプラグインがロードされなかった場合：
- A) エラーとする（厳格）
- B) 内蔵版にフォールバック（互換性重視）
- C) 最小限の内蔵スタブを提供

## あなたの意見
1. このような「Core機能のプラグイン化」は一般的に良い設計ですか？
2. Rust/Go/Zigなど他言語での類似事例はありますか？
3. 「Everything is Plugin」の理想と現実のギャップをどう埋めるべきですか？

## 技術詳細
- 言語：Rust実装のスクリプト言語
- プラグインシステム：TypeBox v2 FFI（C ABI）
- 目標：80,000行→20,000行のコード削減（Phase 15）
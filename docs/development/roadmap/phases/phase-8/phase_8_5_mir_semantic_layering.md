# Phase 8.5: MIRセマンティック階層化（AI大会議決定版）

## 🎯 Issue概要

**方針転換**: ChatGPT5推奨の「20命令intrinsic戦略」から、**Gemini+Codex両先生一致推奨の「25命令セマンティック階層化」**に変更

**理由**: AI大会議による深い分析の結果、20命令intrinsic戦略は以下の致命的問題が判明：
- JIT/AOT最適化機会の喪失
- Everything is Box哲学の意味情報消失  
- 長期的な実装・保守コスト増大
- パフォーマンス劣化リスク

## 🧠 AI大会議分析結果

### Gemini先生分析（理論面）
- **「賢いコンパイラは、賢いMIRから生まれる」**
- RefNew/WeakLoadのintrinsic化 = 最適化阻害の悪手
- BoxFieldLoad/Store等でEverything is Box明示化
- セマンティック階層化で意味保持

### Codex先生分析（実装面）  
- **二相ロワリング戦略**: 25命令維持パス + 20+intrinsic降格パス
- 実装コスト: 5命令追加で10-20人日（intrinsic戦略より安い）
- マイクロベンチ実測でintrinsicオーバーヘッド検証
- 段階的移行（35→25）で安全な実装

## 📋 決定版: セマンティック階層化MIR（25命令）

### **Tier-0: 普遍的コア（8命令）**
```mir
Const, BinOp, Compare, Branch, Jump, Return, Phi, Call
```
- どんな言語にも共通する基本命令群
- 全バックエンドで必須サポート

### **Tier-1: Nyashセマンティクス（12命令）**
```mir
NewBox, BoxFieldLoad, BoxFieldStore, BoxCall, Safepoint,
RefGet, RefSet, WeakNew, WeakLoad, Send, Recv, 
TypeTest, WeakUpgrade
```
- **Everything is Box哲学の具現化**
- **最適化に不可欠**: JIT/AOTでのエスケープ解析・RC除去の基盤
- **BoxFieldLoad/Store**: `obj.field`専用（Load/Storeより明確）
- **TypeTest**: 動的型検査（分岐最適化の核心）
- **WeakUpgrade**: weak→strong昇格（GC協調で重要）

### **Tier-2: 高度フロー（5命令）**
```mir
Throw, Catch, Pin, Unpin, Barrier
```
- 必須だが頻出度低い高度機能
- WASM等ではランタイム関数呼出しに降格可能

## 🔄 二相ロワリング戦略（Codex提案）

### アーキテクチャ
```
Frontend → New MIR(25命令) → 
  ├─ パスA: VM/JIT/AOT向け（25命令のまま最適化）
  └─ パスB: WASM/最小実装向け（25→20+intrinsic降格）
```

### 利点
- **柔軟性**: バックエンドの能力に応じて最適形式選択
- **互換性**: 既存35命令からの段階移行
- **性能**: 高度バックエンドでセマンティクス活用、最小バックエンドで実装簡素化

## 🧪 検証戦略

### 1. パフォーマンス実測（Codex設計）
**マイクロベンチ3カテゴリ:**
- BoxFieldLoad/Store連鎖（構造体/配列/辞書）
- WeakLoad/Upgrade頻発＋GCセーフポイント
- Send/Recvホットループ＋多待ち

**比較軸:**
- 35現行 vs 25セマンティクス vs 20+intrinsic
- Interpreter/VM/WASM全バックエンド
- 命令数/ランタイムcall回数/最適化効果

### 2. 実装検証
**段階的移行（4フェーズ）:**
1. 仕様固定・ロワリング設計
2. 二相ロワリング導入＋互換Shim
3. バックエンド増分対応
4. 旧命令縮退・削除

### 3. 機能保持確認
- **参照実装**: 単一ソース→両MIR→出力一致検証
- **ゴールデンMIR**: 代表プログラムのスナップショット
- **差分実行**: Interpreter/VM/WASMトライアングル比較

## 🎯 実装優先度

### Phase 8.5A: コア変換（最優先）
- [ ] Tier-0/1命令の詳細仕様策定
- [ ] BoxFieldLoad/Store → RefGet/SetのMIR変換
- [ ] TypeTest/WeakUpgrade命令実装

### Phase 8.5B: 二相ロワリング
- [ ] 25命令維持パス実装
- [ ] 20+intrinsic降格パス実装  
- [ ] バックエンド選択ロジック

### Phase 8.5C: 検証・最適化
- [ ] マイクロベンチ実装・実測
- [ ] Golden MIRテストスイート
- [ ] 性能回帰検出CI

## ✅ 成功基準

### 必須基準
- [ ] 25命令セマンティクス完全実装
- [ ] 全バックエンドで機能保持
- [ ] パフォーマンス劣化なし（ベンチマーク基準）
- [ ] Golden MIRテスト全PASS

### 理想基準
- [ ] JIT/AOTでの最適化効果確認
- [ ] WASM降格パスでも実用性能
- [ ] 開発・デバッグ体験向上

## 🤖 Copilot向け実装ガイド

### 重要なポイント
- **BoxFieldLoad/Store重視**: Everything is Box哲学の核心
- **TypeTest活用**: 動的型検査最適化
- **WeakUpgrade**: GC協調の要
- **二相設計**: 高度バックエンドと最小バックエンドの両立

### デバッグ支援
```bash
# セマンティクス確認
./target/release/nyash --dump-mir-semantic test.nyash

# 降格パス確認  
./target/release/nyash --dump-mir-lowered test.nyash

# 性能比較
./target/release/nyash --benchmark-mir-passes test.nyash
```

## 📊 期待される効果

### 技術的効果
- Everything is Box哲学のMIRレベル実現
- JIT/AOTでの高度最適化基盤確立
- バックエンド実装の柔軟性向上

### 開発効率向上
- MIR可読性・デバッグ性大幅改善
- 最適化パス開発の容易化
- 長期保守コスト削減

---

**優先度**: High（Phase 8.4完了後）
**担当**: Copilot + Claude協調実装
**AI大会議結論**: Gemini+Codex両先生完全一致推奨
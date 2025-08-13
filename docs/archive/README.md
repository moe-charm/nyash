# 📚 Nyash Documentation

**NyashプログラミングLexicalAnalyzer言語の公式ドキュメント** | 最終更新: 2025-08-12

---

## 🚀 すぐ始める

### 👶 **初心者向け**
- **[Getting Started](GETTING_STARTED.md)** - 環境構築から最初のプログラムまで

### 📖 **言語を学ぶ**
- **[Language Guide](LANGUAGE_GUIDE.md)** - 言語仕様・構文・完全ガイド

### 🌐 **P2P通信**
- **[P2P Guide](P2P_GUIDE.md)** - P2P通信システム完全ガイド

---

## 📋 詳細リファレンス

### **[reference/](reference/)**
- **[language-reference.md](reference/language-reference.md)** - 言語仕様完全リファレンス
- **[override-delegation-syntax.md](reference/override-delegation-syntax.md)** - デリゲーション・override構文仕様
- **[design-philosophy.md](reference/design-philosophy.md)** - 明示的デリゲーション革命の設計思想
- **[builtin-boxes.md](reference/builtin-boxes.md)** - ビルトインBox型詳細リファレンス

---

## 🗄️ 過去資料・開発履歴

### **[archive/](archive/)**
- **[development/](archive/development/)** - 過去のドキュメント・開発履歴
- **[p2p/](archive/p2p/)** - P2P詳細設計書・AI相談記録

---

## 🎯 Nyashとは

**「Everything is Box」哲学**に基づく革新的プログラミング言語

```nyash
// シンプルで強力な構文
local greeting = "Hello, Nyash!"
print(greeting)

// すべてがBox - 統一された美しい世界
local numbers = new ArrayBox()
numbers.push(42)
numbers.push(3.14)

// P2P通信もBox！
local node = new P2PBox("alice", transport: "inprocess")
node.send("bob", new IntentBox("chat", { text: "Hello P2P!" }))
```

### ✨ **主な特徴**
- **🔧 Production Ready**: Phase 1完了、実用レベルの言語機能
- **🌐 P2P Native**: P2P通信がビルトイン (Phase 2実装中)
- **🛡️ Memory Safe**: Rust実装による完全メモリ安全性
- **📦 Everything is Box**: 統一されたオブジェクトモデル
- **⚡ Simple & Powerful**: 学習コストが低く、表現力が高い

### 📊 **実装状況 (2025-08-12)**

#### ✅ **Phase 1完了**
- FloatBox, ArrayBox改良, Cross-type演算子
- 包括的テストスイート (188行)
- デリゲーション革命 (`from`構文完成)

#### 🚧 **Phase 2実装中**
- IntentBox (構造化メッセージ)
- P2PBox (P2P通信ノード)  
- MessageBus (プロセス内シングルトン)

#### 🎯 **最終目標**
**NyaMeshP2Pライブラリ実現** - Nyash言語による本格的P2P通信ライブラリ

---

## 🤝 コミュニティ

### 開発方針
- **ドキュメントファースト**: ソースより先にドキュメント確認
- **AI協働開発**: Gemini先生・ChatGPT先生・Copilot連携
- **段階的実装**: Phase 1→2→3の確実な進歩

### 貢献方法
1. **Issue報告**: バグ・要望をGitHub Issuesで報告
2. **ドキュメント改善**: typo修正・内容追加のPull Request歓迎
3. **コード貢献**: 新機能実装・バグ修正のPull Request歓迎

---

**🎉 Welcome to the world of "Everything is Box"!**

*Nyashで新しいプログラミングの世界を体験しよう！*
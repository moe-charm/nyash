# 📚 言語進化ロードマップ - ドキュメント索引

**作成日**: 2025-10-02
**対象**: Phase 16-30（言語機能・標準ライブラリの進化）

---

## 🎯 **メインドキュメント**

### **[README.md](./README.md)** - 言語進化ロードマップ v2.0 🌟
**Phase 16-30の完全な実装計画**

- **「コアは最小・糖衣は最強」** 方針に基づく設計
- **9の柱 + 糖衣5つ** - 全機能デシュガリング
- **実装タイムライン** - 優先順位付き
- **デシュガリング規則一覧** - 全16種

**こんな人向け**:
- Phase 16-30で何を実装するか知りたい
- 言語機能の全体像を把握したい
- 実装優先順位を確認したい

---

## 📖 **設計原則**

### **[desugaring-contract.md](./desugaring-contract.md)** - デシュガリング契約 📜
**Hakoruneの最も重要な設計原則**

- **5つの不変ルール（赤線）** - 絶対に守るべき原則
- **デシュガリング実例集** - 型システム、並行処理、糖衣構文、マクロ
- **全16種のデシュガリング規則表** - すべてMIR命令増加なし
- **契約違反の例** - やってはいけないこと

**こんな人向け**:
- 新機能を実装する開発者
- MIR14凍結の理由を知りたい
- デシュガリングの具体例を見たい

---

## 🔍 **問題分析**

### **[discoverability-analysis.md](./discoverability-analysis.md)** - 発見性問題分析 🔎
**なぜChatGPTが冗長なコードを書いてしまったのか**

- **問題の発見** - if連鎖 vs match式
- **5つの根本原因** - ドキュメント導線、サンプル不足、AI学習データ不足、Linter不足等
- **5つの解決策** - Cookbook/Recipe集、Linter、Quick Reference拡充等
- **Phase 17で即座に着手** - 発見性問題の根本解決

**こんな人向け**:
- なぜ「つよつよ機能」が使われないのか知りたい
- Cookbook/Linter実装の背景を知りたい
- 言語機能の発見性向上に興味がある

---

## 📚 **関連ドキュメント**

### **外部リンク**
- [アーキテクチャ戦略](../architecture-strategy.md) - Rust vs セルフホスト実装戦略
- [Phase 16 Macro Revolution](../phases/phase-16-macro-revolution/README.md) - マクロシステム詳細
- [MIR Instruction Set](../../../reference/mir/INSTRUCTION_SET.md) - MIR14命令セット詳細

### **歴史的資料**
- [v1-original.md](./v1-original.md) - Claude初版（参考資料）

---

## 🗺️ **ドキュメント構造**

```
docs/development/roadmap/language-evolution/
├── INDEX.md                      # このファイル（索引）
├── README.md                     # メインドキュメント（v2.0）⭐
├── desugaring-contract.md        # デシュガリング契約（設計原則）
├── discoverability-analysis.md   # 発見性問題分析
└── v1-original.md               # Claude初版（参考資料）
```

---

## 🎯 **読む順番（推奨）**

### **初めての人**
1. **README.md** - 全体像を把握
2. **desugaring-contract.md** - 設計原則を理解
3. **discoverability-analysis.md** - 発見性問題を知る

### **新機能を実装する開発者**
1. **desugaring-contract.md** - 設計原則を確認
2. **README.md** - 実装計画を確認
3. **Phase 16-30のタイムライン** - 優先順位を確認

### **Cookbook/Linter実装者**
1. **discoverability-analysis.md** - 問題の本質を理解
2. **README.md（Phase 17）** - Cookbook/Linter計画を確認
3. **desugaring-contract.md** - デシュガリング規則を確認

---

## 💡 **重要な原則**

### **De-sugaring Contract（デシュガリング契約）**
> **新構文は既存構文・既存Boxへ有限段で必ず落ちること。IR命令の追加は最後の手段。**

### **「コアは最小・糖衣は最強」**
```
MIR14命令セット（凍結）
    ↓
すべての新機能をデシュガリング
    ↓
Box/マクロ/標準ライブラリで実現
    ↓
VM/LLVM/WASM すべて恩恵！
```

### **Everything is Box の真髄**
- **型システム**: OptionBox, SumBox
- **並行処理**: TaskGroupBox, SelectBox, ChannelBox
- **テスト**: TestRunnerBox, BenchmarkBox
- **プロファイル**: ProfileBox

---

## 🎊 **まとめ**

このフォルダには、**Hakorune言語進化の完全な設計図**が含まれています。

**3つのドキュメント**で、Phase 16-30の全体像を把握できます：
1. **README.md** - 実装計画（9の柱 + 糖衣5つ）
2. **desugaring-contract.md** - 設計原則（5つの不変ルール）
3. **discoverability-analysis.md** - 発見性問題と解決策

**これで、Hakoruneは次世代言語の標準を打ち立てる準備が整っています！** 🚀

---

**作成者**: Claude Sonnet 4.5
**作成日**: 2025-10-02
**関連**: [言語進化ロードマップ v2.0](./README.md)

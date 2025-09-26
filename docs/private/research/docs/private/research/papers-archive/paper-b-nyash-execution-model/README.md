# 論文B: Nyash言語と実行モデル

> Scope (2025-09-19): 本稿の範囲は PyVM と LLVM/llvmlite の2系統に限定。MIR14 は PHI‑off（合流はエッジコピー）、PHI 形成は LLVM ハーネスで行う。JIT/Interpreter は Phase‑15 では補助扱い。詳細は SCOPE.md を参照。

## 📚 概要

**タイトル**: Nyash: A Box-First Programming Language with Symmetric Memory Management and P2P Intent Model

**主題**: Nyash言語そのものの設計と実装

**対象読者**: 言語理論・分散システム・アプリ開発寄り

## 🎯 研究ポイント

### 1. init/fini対称性によるメモリ管理
- コンストラクタ（init/birth/pack）とデストラクタ（fini）の対称設計
- 明示的なリソース管理による安全性
- GCオン/オフ切り替え可能な柔軟性

### 2. P2P Intentモデル
- Box間の意図ベース通信
- 分散アプリケーション向け設計
- NyaMeshライブラリによる実装

### 3. 多層実行アーキテクチャ
- **Interpreter**: 開発・デバッグ用
- **VM**: 高速実行
- **JIT**: 動的最適化
- **AOT**: 配布用バイナリ
- **WASM**: Web展開

## 🚀 実装例

### 1. NyashCoin - P2P暗号通貨
```nyash
box NyashCoin from P2PBox {
    init { balance, transactions }
    
    birth(nodeId, network) {
        from P2PBox.pack(nodeId, network)
        me.balance = new MapBox()
        me.transactions = new ArrayBox()
    }
    
    onIntent(intent, data, sender) {
        switch intent {
            "transfer": me.handleTransfer(data, sender)
            "mine": me.handleMining(data, sender)
            "sync": me.handleSync(data, sender)
        }
    }
}
```

### 2. プラグインストア
- 動的プラグインロード
- TypeBox ABIによる相互運用
- セキュアな実行環境

### 3. GUI/Webアプリケーション
- EguiBoxによるGUI開発
- WebCanvasBoxによるブラウザ対応
- 統一的なBox APIによる開発

## 📊 評価計画

### 言語機能の評価
- 表現力: 他言語との比較
- 学習曲線: 初学者への調査
- 開発効率: LOCとバグ率

### 性能評価
- 各バックエンドのベンチマーク
- メモリ使用量の比較
- 起動時間・応答性

### 実用性評価
- 実アプリケーション開発
- プラグインエコシステム
- クロスプラットフォーム性

## 📁 ディレクトリ構造

```
paper-b-nyash-execution-model/
├── README.md              # このファイル
├── abstract.md           # 論文概要
├── main-paper.md         # 本文
├── chapters/             # 章別ファイル
│   ├── 01-introduction.md
│   ├── 02-language-design.md
│   ├── 03-memory-model.md
│   ├── 04-p2p-intent.md
│   ├── 05-execution-backends.md
│   ├── 06-case-studies.md
│   └── 07-conclusion.md
├── figures/              # 図表
│   ├── box-hierarchy.png
│   ├── execution-flow.svg
│   └── p2p-architecture.png
├── examples/             # コード例
│   ├── nyashcoin/
│   ├── plugin-store/
│   └── gui-apps/
├── data/                 # 実験データ
│   ├── performance/
│   └── usability-study/
└── related-work.md       # 関連研究
```

## 🗓️ スケジュール

- **2025年9月**: 実装例の完成・評価実施
- **2025年10月**: 執筆開始
- **2025年11月**: OOPSLA 2026投稿
- **2026年春**: Onward!投稿（設計哲学編）

## 📝 執筆メモ

### 強調すべき貢献
1. **Everything is Box哲学**: 統一的なオブジェクトモデル
2. **対称的メモリ管理**: init/finiによる明示的制御
3. **P2P Intentモデル**: 分散アプリケーションの新パラダイム
4. **多層実行環境**: 用途に応じた最適な実行方式

### 新規性
- Box中心の言語設計
- 意図ベースのメッセージング
- プラグイン第一級市民
- 実行バックエンドの透過的切り替え

### 実証
- 実動作するアプリケーション群
- プラグインエコシステムの構築
- クロスプラットフォーム展開

## 🔗 関連ドキュメント

- [Language Reference](../../../../reference/language/LANGUAGE_REFERENCE_2025.md)
- [Everything is Box](../../../../reference/boxes-system/everything-is-box.md)
- [P2P Box Guide](../../../../guides/p2p-guide.md)
- [Execution Backends](../../../../reference/architecture/execution-backends.md)

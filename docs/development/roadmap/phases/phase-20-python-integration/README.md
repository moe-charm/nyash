# Phase 20: Python-Hakorune統合 - PythonのAST/木構造をHakoruneに落とす

## 📋 概要

PythonのAST（抽象構文木）をHakoruneの木構造（MIR/AST）に変換し、Python-Hakorune間の相互運用を実現するフェーズです。

**出典**: 旧Phase 10.5（Phase 10.1系）の内容を整理・再編成

## 🎯 目的

1. **PythonパーサーBox**: PythonコードをHakoruneで解析
2. **AST変換**: Python AST → Hakorune MIR/AST
3. **相互運用**: Python ⇄ Hakorune の双方向呼び出し
4. **トランスパイラー**: Python → Hakorune コード変換

## 📁 フォルダ構成

```
phase-20-python-integration/
├── README.md                    # このファイル
├── INDEX.md                     # 全体ナビゲーション ⭐推奨
│
├── planning/                    # 計画・設計ドキュメント
│   ├── README.md
│   ├── milestones.md            # ⭐M0-M6実装計画（2025-10-02追加）
│   ├── integrated-plan.md       # 統合計画
│   ├── expert-feedback.md       # AI専門家フィードバック
│   └── python-parser-box-design.md
│
├── parser-integration/          # パーサー統合
│   ├── README.md
│   ├── implementation-plan.md
│   └── builtin-box-flow.md
│
├── core-implementation/         # コア実装
│   ├── README.md
│   └── implementation-roadmap.md
│
├── transpiler/                  # トランスパイラー
│   ├── README.md
│   └── python-to-hakorune-spec.md
│
├── testing/                     # テスト計画
│   └── README.md
│
├── documentation/               # ドキュメント
│   └── README.md
│
└── design/                      # 設計ドキュメント
    ├── README.md
    ├── enhanced-architecture-v2.md ⭐ChatGPT Pro最新設計（2025-10-02追加）
    ├── meta-config-examples.md     ⭐hako.toml設定例（2025-10-02追加）
    ├── risks-and-mitigations.md    ⭐リスク管理（2025-10-02追加）
    ├── abi-design.md               # ABI設計
    ├── native-build-consolidation.md # ネイティブビルド
    └── handle-first-plugininvoke-plan.md # Handle-First設計
```

## 🎯 主要コンポーネント

### 1. PythonパーサーBox
- PythonコードをHakorune内で解析
- `PyRuntimeBox`: Python実行環境
- `PyObjectBox`: Pythonオブジェクト管理

### 2. AST変換
- Python AST → Hakorune MIR
- 型情報の保持
- スコープ解決

### 3. 相互運用
- Hakorune → Python: BoxCall経由
- Python → Hakorune: FFI経由
- エラーハンドリング

### 4. トランスパイラー
- Python構文 → Hakorune構文
- イディオム変換
- 最適化

## 📊 実装ステータス

| コンポーネント | ステータス | 備考 |
|--------------|----------|------|
| 計画・設計 | ✅ 完了 | 詳細設計済み |
| パーサーBox | 📅 未実装 | Phase 15後に実装予定 |
| AST変換 | 📅 未実装 | - |
| 相互運用 | 📅 未実装 | - |
| トランスパイラー | 📅 未実装 | - |

## 🎯 重要ドキュメント（2025-10-02追加）

### 最新設計（ChatGPT Pro UltraThink）
1. **[強化版アーキテクチャv2](design/enhanced-architecture-v2.md)** ⭐必読 - Effect/Capability/Contract/Policy統合設計
2. **[マイルストーン（M0-M6）](planning/milestones.md)** - 段階的実装計画
3. **[メタ設定例](design/meta-config-examples.md)** - hako.toml設定リファレンス
4. **[リスクと対策](design/risks-and-mitigations.md)** - GILデッドロック・メモリリーク等

### ナビゲーション
- **[INDEX.md](INDEX.md)** - 全体ナビゲーション

## 🔗 関連ドキュメント

- **元資料**: `docs/archive/phases/phase-10.5/phase-10.5/`
- **現在のフェーズ**: [Phase 15 - セルフホスティング](../phase-15/)
- **マスタープラン**: [00_MASTER_ROADMAP.md](../00_MASTER_ROADMAP.md)

## ⚠️ 開発優先度

**現在は保留中** - Phase 15（Hakoruneセルフホスティング）完了後に着手予定

理由：
- まずHakorune自身の基盤（Rust VM + LLVM）を固める
- セルフホスティングの実現が優先
- Python統合は後段で段階的に実装

## 🚀 将来の展望

1. **短期**: Python FFI統合（PyO3等）
2. **中期**: Python AST → Hakorune MIR 変換
3. **長期**: Python-Hakorune シームレス統合

---

**最終更新**: 2025-10-02
**ステータス**: 計画済み・実装待ち
**優先度**: Phase 15完了後

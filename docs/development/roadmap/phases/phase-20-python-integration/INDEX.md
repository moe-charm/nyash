# Phase 20 Python Integration - INDEX

## 📋 全体概要

**Phase 20: Python-Hakorune統合** - PythonのAST/木構造をHakoruneに落とし、Python-Hakorune間の相互運用を実現するフェーズです。

**出典**: 旧Phase 10.5（Phase 10.1系）の内容を整理・再編成

## 🗂️ フォルダ構成

```
phase-20-python-integration/
├── README.md                           # ⭐ メインドキュメント
├── INDEX.md                            # このファイル
│
├── 📁 planning/                        # 計画・設計
│   ├── README.md
│   ├── integrated-plan.md              # ChatGPT5統合計画
│   ├── python-parser-plan-summary.md   # パーサー計画サマリー
│   ├── python-parser-box-design.md     # パーサーBox設計
│   └── expert-feedback.md              # AI専門家フィードバック
│
├── 📁 parser-integration/              # パーサー統合
│   ├── README.md
│   ├── implementation-plan.md          # 実装計画
│   └── builtin-box-flow.md             # ビルトインBox実装フロー
│
├── 📁 core-implementation/             # コア実装
│   ├── README.md
│   └── implementation-roadmap.md       # 実装ロードマップ
│
├── 📁 transpiler/                      # トランスパイラー
│   ├── README.md
│   └── python-to-hakorune-spec.md      # Python→Hakorune変換仕様
│
├── 📁 design/                          # 設計ドキュメント
│   ├── README.md
│   ├── abi-design.md                   # ABI設計
│   ├── native-build-consolidation.md   # ネイティブビルド
│   └── handle-first-plugininvoke-plan.md # Handle-First設計
│
├── 📁 testing/                         # テスト計画
│   └── README.md
│
└── 📁 documentation/                   # ドキュメント計画
    └── README.md
```

## 🎯 クイックナビゲーション

### 🚀 初めての方へ
1. [メインREADME](README.md) - 全体概要を理解
2. **[強化版アーキテクチャv2](design/enhanced-architecture-v2.md)** - ChatGPT Pro最新設計（推奨）
3. [マイルストーン](planning/milestones.md) - M0〜M6実装計画
4. [Planning README](planning/README.md) - 計画の詳細を確認
5. [統合計画](planning/integrated-plan.md) - ChatGPT5による詳細計画

### 🏗️ 設計を知りたい
- **[強化版アーキテクチャv2](design/enhanced-architecture-v2.md)** ⭐ ChatGPT Pro最新設計（2025-10-02）
- [ABI設計](design/abi-design.md) - Python-Hakorune間のABI
- [Handle-First設計](design/handle-first-plugininvoke-plan.md) - プラグイン呼び出し設計
- [ネイティブビルド](design/native-build-consolidation.md) - AOT/EXE生成
- [メタ設定例](design/meta-config-examples.md) - hako.toml設定リファレンス
- [リスクと対策](design/risks-and-mitigations.md) - 既知のリスクと対策

### 💻 実装を見たい
- [パーサー実装計画](parser-integration/implementation-plan.md) - PythonパーサーBox
- [コア実装ロードマップ](core-implementation/implementation-roadmap.md) - PyRuntimeBox/PyObjectBox
- [トランスパイラー仕様](transpiler/python-to-hakorune-spec.md) - Python→Hakorune変換

### 🧪 テストについて
- [テスト計画](testing/README.md) - テスト戦略とチェックリスト

### 📚 ドキュメントについて
- [ドキュメント計画](documentation/README.md) - ドキュメント構成

## 📊 実装ステータス

| カテゴリ | コンポーネント | ステータス |
|---------|--------------|----------|
| **計画** | 設計・計画書 | ✅ 完了 |
| **計画** | 強化版アーキテクチャv2 | ✅ 完了（2025-10-02）|
| **計画** | マイルストーン（M0-M6） | ✅ 完了（2025-10-02）|
| **設計** | ABI設計 | ✅ 完了 |
| **設計** | Handle-First | ✅ 完了 |
| **設計** | ネイティブビルド | ✅ 完了 |
| **設計** | メタ設定例 | ✅ 完了（2025-10-02）|
| **設計** | リスク管理 | ✅ 完了（2025-10-02）|
| **実装** | PyRuntimeBox | 📅 未実装（M0予定）|
| **実装** | PyObjectBox | 📅 未実装（M0予定）|
| **実装** | PythonパーサーBox | 📅 未実装（M4予定）|
| **実装** | トランスパイラー | 📅 未実装 |
| **テスト** | ユニットテスト | 📅 未実装 |
| **テスト** | 統合テスト | 📅 未実装 |
| **ドキュメント** | ユーザーガイド | 📅 未実装 |
| **ドキュメント** | APIリファレンス | 📅 未実装 |

## 🎯 主要コンポーネント

### 1. PyRuntimeBox
**目的**: Python実行環境の提供

```hakorune
box PyRuntimeBox {
    eval(code: StringBox) -> PyObjectBox
    import(name: StringBox) -> PyObjectBox
}
```

### 2. PyObjectBox
**目的**: Pythonオブジェクトの管理

```hakorune
box PyObjectBox {
    getattr(name: StringBox) -> PyObjectBox
    setattr(name: StringBox, value: PyObjectBox)
    call(args: ArrayBox) -> PyObjectBox
    to_string() -> StringBox
    to_int() -> IntegerBox
}
```

### 3. PythonパーサーBox
**目的**: Python AST → Hakorune MIR変換

```hakorune
box PythonParserBox {
    parse(code: StringBox) -> ASTBox
    to_hakorune_ast(ast: ASTBox) -> HakoruneASTBox
    to_mir(ast: ASTBox) -> MIRBox
}
```

### 4. トランスパイラー
**目的**: Python構文 → Hakorune構文変換

```hakorune
box PythonTranspiler {
    transpile(python_code: StringBox) -> StringBox
}
```

## ⚠️ 現在のステータス

**保留中** - Phase 15（Hakoruneセルフホスティング）完了後に着手予定

### 保留理由
1. **優先度**: まずHakorune自身の基盤を固める
2. **依存関係**: セルフホスティング実現が先決
3. **リソース**: 段階的に実装する方が安全

### 再開条件
- ✅ Phase 15完了（Rust VM + LLVM 2本柱体制確立）
- ✅ セルフホスティング実現
- ✅ プラグインシステム安定化

## 🔗 関連Phase

### 前提Phase
- [Phase 15 - セルフホスティング](../phase-15/) - 現在進行中
- [Phase 12 - TypeBox統合](../phase-12/) - 完了

### 後続Phase（予定）
- Phase 21 - 箱データベース
- Phase 22+ - 未定

## 📚 元資料

このPhase 20は以下の旧資料を整理・統合したものです：

- **元の場所**: `docs/archive/phases/phase-10.5/phase-10.5/`
- **旧Phase名**: Phase 10.5（Phase 10.1系を再編成）
- **作成時期**: 2025年8月頃
- **主要貢献者**: ChatGPT5, Gemini, Codex

## 🎯 将来の展望

### 短期目標（Phase 15後）
1. PyRuntimeBox基本実装
2. PyObjectBox基本実装
3. 基本的なPython呼び出し動作

### 中期目標
1. PythonパーサーBox実装
2. Python AST → Hakorune MIR変換
3. トランスパイラー実装

### 長期目標
1. Python-Hakoruneシームレス統合
2. Pythonライブラリの完全活用
3. ハイブリッド開発の実現

## 📞 お問い合わせ

- **メインドキュメント**: [README.md](README.md)
- **マスタープラン**: [00_MASTER_ROADMAP.md](../00_MASTER_ROADMAP.md)
- **現在のフェーズ**: [Phase 15](../phase-15/)

---

**最終更新**: 2025-10-02
**整理者**: Claude Code
**ステータス**: 計画済み・実装待ち

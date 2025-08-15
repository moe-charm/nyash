# Nyashプログラミング言語のinclude/namespace/usingシステム設計相談

## 🎯 現在の状況

### 1. namespace & using設計完了
IDE補完最優先システム設計済み：
```nyash
# 名前空間定義
namespace nyashstd {
    static box string {
        static upper(str) {
            return StringBox.upper(str)  # 既存実装活用
        }
        static lower(str) { ... }
    }
    static box math {
        static sin(x) { ... }
    }
}

# using文での使用
using nyashstd
string.upper("hello")  # 短い＆明確
math.sin(3.14)

# 完全修飾名（常時利用可能）
nyashstd.string.upper("hello")
```

### 2. 既存include実装
単純なファイル読み込み+実行システム：
```nyash
include "myfile.nyash"  # ファイル内容をパース・実行
```

- 重複読み込み防止機能あり
- しかし依存関係管理・名前空間分離なし

### 3. 新たな課題：統合問題
includeとnamespace/usingの統合が必要：
- ファイル間依存関係システムが必要
- 循環依存の検出・防止
- 読み込み順序の決定アルゴリズム

## 🚨 技術的課題

### A. 依存関係解決の複雑性
```nyash
# main.nyash
using nyashstd          # ← nyashstd.nyashの読み込みが必要
using mylib            # ← mylib.nyashの読み込みが必要
string.upper("hello")   # nyashstdから
mylib.custom()         # mylibから
```

### B. include vs using の設計統合
- **include**: 即座にファイル実行（現在の実装）
- **using**: 名前空間のインポートのみ（新設計）
- 両者の統合・共存方法が不明

### C. ファイル探索・解決
- `using nyashstd` → どのファイルを読み込む？
- 標準ライブラリ vs ユーザーライブラリの区別
- パス解決アルゴリズム

## 💡 検討中の解決案：nyash.linkファイル方式

### 基本アイデア
Cargo.toml/package.json類似の依存関係管理ファイル：

```toml
# nyash.link (プロジェクトルート)
[dependencies]
nyashstd = "./stdlib/nyashstd.nyash"
mylib = "./libs/mylib.nyash"

[search_paths]
stdlib = "./stdlib/"
libs = "./libs/"
```

### 動作イメージ
1. `using nyashstd` 実行時
2. nyash.linkを読み取り
3. `"./stdlib/nyashstd.nyash"` を特定
4. ファイル読み込み・名前空間登録
5. `string.upper()` が使用可能に

## 🤔 深く検討してほしい技術的論点

### 1. nyash.linkファイル方式の妥当性
- **実装複雑度**: 依存関係グラフ構築・解決アルゴリズム
- **パフォーマンス**: キャッシュ・遅延読み込みの必要性
- **他言語比較**: Rust Cargo、Node.js、Python等の実装からの学習

### 2. 既存includeとの共存戦略
**選択肢A**: includeを低レベルAPIとして残す
```nyash
include "config.nyash"    # 即座実行（設定ファイル等）
using mylib              # 名前空間インポート（ライブラリ）
```

**選択肢B**: includeを廃止、usingに統一
```nyash
using config             # 設定も名前空間として扱う
using mylib              # ライブラリも名前空間
```

**選択肢C**: includeをusingの内部実装として隠蔽

### 3. 段階的実装戦略
- **最小実装**: 固定パスでのusing実装
- **中級実装**: nyash.link基本機能
- **完全実装**: 循環依存検出・パッケージ管理

### 4. IDE補完・Language Server連携
- nyash.linkによる依存関係情報の活用
- 補完候補の動的生成
- エラー検出・警告システム

### 5. 標準ライブラリ管理
- nyashstdの標準配置場所（相対パス？絶対パス？）
- ユーザーライブラリとの区別方法
- 将来のパッケージ管理システムへの発展性

## 🎯 具体的な質問

1. **nyash.linkファイル方式は技術的に健全で実装可能か？**
   - 依存関係解決アルゴリズムの実装困難度
   - 他言語での類似実装の成功例・失敗例

2. **includeとusingの最適な関係性は？**
   - 両方残すべき？統一すべき？
   - それぞれの用途・使い分け

3. **最小実装からの段階的発展戦略は？**
   - Phase 1で何を実装すべき？
   - 段階的機能追加の優先順位

4. **パフォーマンスへの影響は許容範囲内か？**
   - ファイル読み込みオーバーヘッド
   - 名前解決の計算コスト

5. **他に考慮すべき設計上の課題はあるか？**
   - 見落としている技術的問題
   - より良い代替案の存在

## 🌟 Nyashの設計哲学との整合性

- **Everything is Box**: 名前空間もBoxとして扱うべき？
- **明示性重視**: 依存関係の明示的記述（nyash.link）は哲学と合致
- **初心者フレンドリー**: include廃止は学習コストを下げるか？

## 🔥 期待する回答

プログラミング言語設計・実装の専門的視点から：
- nyash.link方式の実現可能性・妥当性評価
- 実装戦略の具体的提案
- 潜在的課題の指摘・解決策
- 他言語実装例からの学習ポイント
- Nyash哲学との整合性確保方法

---

**深い技術検討をお願いします！🐾**
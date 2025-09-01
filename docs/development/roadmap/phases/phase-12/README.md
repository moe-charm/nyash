# Phase 12: Nyashコード共有エコシステム - Everything is Box の実現

## 🎯 重要な変更 (2025-09-01)

Phase 12の議論とビルトインBox廃止により、プラグインシステムが進化：

**新しい3層プラグインシステムが確立されました！**

```nyash
# Nyashスクリプトプラグイン（ユーザー定義Box）
box DataProcessor {
    init {
        me.file = new FileBox()    # C ABIプラグイン使用
        me.math = new MathBox()    # C ABIプラグイン使用
        me.cache = new MapBox()    # これもC ABIプラグイン（ビルトイン廃止）
    }
    
    process(data) {
        local result = me.math.sin(data)
        me.file.write("log.txt", result.toString())
        return result
    }
}

# 使用例
local processor = new DataProcessor()
processor.process(3.14)  # すべてプラグインで動作！
```

## 📝 なぜ誤解が生まれたのか

「プラグイン」という言葉から、特別な仕組みが必要だと考えてしまいましたが、Nyashの「Everything is Box」哲学により、ユーザー定義Boxこそが最高のプラグインシステムでした。

詳細な分析：[なぜ天才AIたちは間違えたのか](./WHY-AIS-FAILED.md)

## 🚀 Phase 12の真の価値：コード共有エコシステム

### 本当に必要なもの

1. **export/import構文**
   ```nyash
   # math_utils.ny
   export box MathUtils {
       factorial(n) { ... }
       fibonacci(n) { ... }
   }
   
   # main.ny
   import { MathUtils } from "math_utils.ny"
   local utils = new MathUtils()
   ```

2. **パッケージマネージャー**
   ```bash
   nyash install awesome-math-utils
   nyash publish my-cool-box
   ```

3. **ドキュメント生成**
   ```nyash
   # @doc 素晴らしい数学ユーティリティ
   # @param n 計算したい数値
   # @return 階乗の結果
   export box MathUtils { ... }
   ```

## 📊 新しい3層プラグインシステム

```
Nyashエコシステム（ビルトインBox廃止後）：
├── Nyashスクリプトプラグイン（ユーザー定義Box）← .nyashファイル
├── C ABIプラグイン（既存のまま使用）← シンプル・高速・安定
└── Nyash ABIプラグイン（必要時のみ）← 言語間相互運用・将来拡張
    └── MIR命令は増やさない（BoxCallにabi_hint追加のみ）
```

### プラグイン選択の指針
- **C ABIで済むなら、C ABIを使う**（シンプルイズベスト）
- Nyash ABIは以下の場合のみ：
  - 他言語（Python/Go等）からの呼び出し
  - 複雑な型の相互運用が必要
  - 将来の拡張性を重視する場合

## 🛣️ 実装ロードマップ（修正版）

### Phase 12.1: export/import構文（2週間）
- [ ] exportキーワードのパーサー実装
- [ ] importステートメントの実装
- [ ] モジュール解決システム
- 📄 **[詳細仕様書](./export-import-spec.md)**

### Phase 12.2: パッケージ管理（3週間）
- [ ] nyash.tomlのdependencies対応
- [ ] 中央リポジトリ設計
- [ ] CLIツール（install/publish）
- 📄 **[パッケージマネージャー設計書](./package-manager-design.md)**

### Phase 12.3: 開発者体験向上（継続的）
- [ ] ドキュメント生成ツール
- [ ] VSCode拡張（補完・定義ジャンプ）
- [ ] サンプルパッケージ作成

---

### 🗄️ 議論の過程

AIたちがなぜ複雑な解決策を提案したのか、その議論の過程は `archive/` ディレクトリに保存されています。良い教訓として残しておきます。

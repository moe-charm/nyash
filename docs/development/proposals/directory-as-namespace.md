# Directory-as-Namespace: 究極のシンプルモジュールシステム

**Status**: Proposal
**Date**: 2025-10-07
**Phase**: Phase 2.2 候補

---

## 🎯 コンセプト

**「ディレクトリ名 = ネームスペース」で設定ファイルほぼ不要に！**

```
apps/
├── calculator/
│   ├── Math.hako         → calculator.Math
│   └── Advanced.hako     → calculator.Advanced
└── network/
    ├── Http.hako         → network.Http
    └── WebSocket.hako    → network.WebSocket
```

**たったこれだけ！設定ファイルゼロでモジュール化完了！**

---

## 🚀 基本ルール

### **ルール1: ディレクトリ構造 = ネームスペース階層**

```
selfhost/vm/boxes/
└── Entry.hako  → selfhost.vm.boxes.Entry
```

- ディレクトリパス → ドット区切りネームスペース
- 自動的に階層構造を反映

### **ルール2: ファイル名 = Box名（推奨）**

```
apps/mylib/
├── Utils.hako          → mylib.Utils
├── Helper.hako         → mylib.Helper
└── Formatter.hako      → mylib.Formatter
```

- **1ファイル1Box推奨**（PascalCase.hako）
- ファイル名とBox名を一致させる
- 複数Boxも許可（後方互換）

### **ルール3: 検出ルート指定**

```toml
# hako.toml
[modules.options]
roots = ["apps", "lib"]
enable_discovery = true
```

- `roots`ディレクトリ配下のみ自動検出
- 明確なスコープ（全ファイルスキャンしない）

---

## 📊 他言語との比較

| 言語 | ディレクトリ=NS | ファイル名=型名 | 設定不要 | 評価 |
|------|---------------|---------------|---------|------|
| **Go** | ✅ | ❌ | ✅ | ⭐⭐⭐⭐ |
| **Python** | ✅ | ❌ | ⚠️ `__init__.py` | ⭐⭐⭐⭐ |
| **Java** | ✅ | ✅ | ❌ package宣言 | ⭐⭐⭐ |
| **Rust** | ⚠️ mod宣言 | ❌ | ❌ Cargo.toml | ⭐⭐⭐ |
| **Hakorune (提案)** | ✅ | ✅ | ✅ | ⭐⭐⭐⭐⭐ |

**Hakoruneが最もシンプル！Go + Javaのいいとこ取り**

---

## 💡 実装例

### **Before（現在の複雑さ）**

```toml
# hako.toml（100行以上の手動定義）
[modules]
"selfhost.vm.entry" = "selfhost/vm/boxes/mini_vm_entry.hako"
"selfhost.vm.mir_min" = "selfhost/vm/boxes/mir_vm_min.hako"
"selfhost.vm.core" = "selfhost/vm/boxes/mini_vm_core.hako"
# ... 97行続く
```

### **After（提案のシンプルさ）**

```toml
# hako.toml（たった3行！）
[modules.options]
roots = ["apps"]
enable_discovery = true
```

```
apps/
└── selfhost/
    └── vm/
        └── boxes/
            ├── Entry.hako      → selfhost.vm.boxes.Entry
            ├── MirMin.hako     → selfhost.vm.boxes.MirMin
            └── Core.hako       → selfhost.vm.boxes.Core
```

---

## 🎨 使用例

### **モジュール作成（3ステップ）**

```bash
# 1. ディレクトリ作成
mkdir -p apps/mylib

# 2. Boxファイル作成
cat > apps/mylib/Utils.hako <<'EOF'
static box Utils {
    add(a, b) {
        return a + b
    }
}
EOF

# 3. 完了！（設定不要）
```

### **使用例**

```nyash
// main.hako
using mylib.Utils

static box Main {
    main() {
        local result = Utils.add(10, 20)
        print(result)  // 30
    }
}
```

---

## 🔧 詳細設計

### **命名規則の自動変換**

| ディレクトリ名 | ネームスペース | 備考 |
|--------------|--------------|------|
| `my-lib` | `my_lib` | ケバブ→スネーク変換 |
| `my_lib` | `my_lib` | そのまま |
| `MyLib` | `MyLib` | PascalCaseそのまま |
| `my.lib` | ❌ エラー | ドット禁止 |
| `my lib` | ❌ エラー | スペース禁止 |

### **同一ネームスペース内の参照**

```nyash
// apps/mylib/Utils.hako
static box Utils {
    help() {
        // 同一ネームスペース内は直接参照OK
        Helper.do_something()  // mylib.Helper
    }
}

// apps/mylib/Helper.hako
static box Helper {
    do_something() {
        Utils.help()  // mylib.Utils
    }
}
```

**同一ディレクトリ内は`using`不要！**

### **プライベートBox**

```
apps/mylib/
├── Utils.hako          → mylib.Utils（public）
└── _Internal.hako      → mylib._Internal（private推奨）
```

```toml
# apps/mylib/hako_module.toml（オプション）
[private]
patterns = ["_*"]  # アンダースコア始まりは非公開
```

---

## 🎯 柔軟性：オーバーライド可能

### **デフォルト（設定不要）**

```
apps/stringutils/
└── Helper.hako  → stringutils.Helper
```

### **明示的オーバーライド（必要時のみ）**

```toml
# apps/stringutils/hako_module.toml
[module]
name = "str-utils"  # オーバーライド
version = "1.0.0"

[exports]
Helper = "Helper.hako::Helper"
```

**95%のケースは設定不要、5%は柔軟に対応！**

---

## 🚨 問題と解決策

### **問題1: ディレクトリリネーム = API破壊**

**解決策A: エイリアス**
```toml
[modules.aliases]
old-name = "new-name"  # 後方互換性
```

**解決策B: バージョン管理**
```
apps/
├── mylib/           → mylib.* (v2.0)
└── mylib-v1/        → mylib_v1.* (v1.0 レガシー)
```

### **問題2: 長いネームスペース**

```
apps/very/deep/nested/structure/
└── Utils.hako  → very.deep.nested.structure.Utils
```

**解決策: hako_module.toml でショートカット**
```toml
[module]
name = "shortname"
```

### **問題3: 複数Boxが同一ファイル**

```nyash
// utils.hako
static box Utils { ... }
static box UtilsHelper { ... }
```

**解決策: 1ファイル1Box推奨（複数も許可）**
```toml
# hako_module.toml で明示
[exports]
Utils = "utils.hako::Utils"
UtilsHelper = "utils.hako::UtilsHelper"
```

---

## 📈 段階的導入戦略

### **Phase 1: オプトイン（実験的）**

```toml
[modules.options]
enable_discovery = true
directory_as_namespace = true  # 明示的有効化
roots = ["apps"]
```

- 既存システムと共存
- 実験的機能として提供

### **Phase 2: デフォルト化**

```toml
[modules.options]
# directory_as_namespace = true（デフォルト）
roots = ["apps"]
```

- 新規プロジェクトはデフォルト有効
- 既存プロジェクトは明示的無効化可能

### **Phase 3: レガシー削除**

- 手動`[modules]`エントリ削除
- 完全自動化

---

## 🎊 メリット

### **開発者体験**

- ✅ **学習コストゼロ**: ディレクトリ作るだけ
- ✅ **設定ファイル最小**: 99%のケースで不要
- ✅ **リファクタリング簡単**: ファイル移動しても同一NS内ならOK
- ✅ **一貫性**: ディレクトリ構造 = ネームスペース構造

### **プロジェクト管理**

- ✅ **スケーラビリティ**: 大規模プロジェクト対応
- ✅ **可視性**: ディレクトリ見ればNS分かる
- ✅ **モジュール境界明確**: 1ディレクトリ = 1モジュール

### **AIとの親和性**

- ✅ **予測可能**: ディレクトリ構造から自動推論
- ✅ **説明不要**: Claude/ChatGPTが直感的に理解
- ✅ **コンテキスト削減**: 設定ファイル読む必要なし

---

## 🔗 関連提案

### **Phase 2.1との統合**

本提案はPhase 2.1（hako_module.toml導入）と完全互換：

```toml
# hako_module.toml（オプション）
[module]
# 書かなければディレクトリ名を使用
# 書けばオーバーライド可能
name = "custom-name"

[exports]
# 書かなければすべて自動エクスポート
# 書けば個別制御
Helper = "Helper.hako::Helper"
```

### **既存システムとの関係**

| システム | 関係 | 備考 |
|---------|------|------|
| **Phase 2.1 (hako_module.toml)** | 統合 | オプションとして共存 |
| **Auto-discovery** | 拡張 | ディレクトリベースで強化 |
| **using システム** | 変更なし | 既存構文そのまま |

---

## 📝 実装チェックリスト

### **必須機能**

- [ ] ディレクトリスキャン（`roots`配下）
- [ ] パス→ネームスペース変換
- [ ] ファイル名→Box名マッピング
- [ ] 同一NS内の自動解決
- [ ] hako_module.toml オーバーライド対応

### **推奨機能**

- [ ] 命名規則lint（ドット/スペース禁止）
- [ ] `_*` パターンでプライベート化
- [ ] 循環依存検出（同一NS内は許可）
- [ ] `--list-modules` でプレビュー

### **オプション機能**

- [ ] エイリアス自動生成
- [ ] バージョン管理統合
- [ ] IDE統合（補完・ジャンプ）

---

## 🎯 期待される効果

### **定量的**

- 設定ファイル行数: **100行 → 3行**（97%削減）
- モジュール追加時間: **5分 → 30秒**（90%短縮）
- 学習コスト: **1時間 → 5分**（95%削減）

### **定性的**

- **直感性**: ディレクトリ見ればすべて分かる
- **保守性**: ファイル移動でもNS維持
- **拡張性**: 大規模プロジェクトでも破綻しない

---

## 💎 究極の例

```
apps/
├── calculator/
│   ├── Math.hako           → calculator.Math
│   ├── Scientific.hako     → calculator.Scientific
│   └── _Internal.hako      → calculator._Internal (private)
├── network/
│   ├── Http.hako           → network.Http
│   ├── WebSocket.hako      → network.WebSocket
│   └── internal/
│       └── Parser.hako     → network.internal.Parser
└── database/
    ├── Sql.hako            → database.Sql
    └── NoSql.hako          → database.NoSql
```

```nyash
// main.hako
using calculator.Math
using network.Http
using database.Sql

static box Main {
    main() {
        local sum = Math.add(10, 20)
        Http.get("https://api.example.com")
        Sql.query("SELECT * FROM users")
    }
}
```

**設定ファイルゼロ！ディレクトリ作るだけで完結！** 🚀

---

## 🎊 まとめ

### **コアアイデア**

1. **ディレクトリパス = ネームスペース** → 自動マッピング
2. **ファイル名 = Box名** → 1ファイル1Box推奨
3. **roots指定のみ** → 最小限の設定（3行）

### **利点**

- ⭐⭐⭐⭐⭐ **シンプルさ**: 他言語最高レベル
- ⭐⭐⭐⭐⭐ **直感性**: 誰でもすぐ理解
- ⭐⭐⭐⭐⭐ **スケーラビリティ**: 大規模対応

### **推奨実装順序**

1. **Phase 2.2-A**: 基本実装（auto-discovery拡張）
2. **Phase 2.2-B**: オーバーライド対応
3. **Phase 2.2-C**: lintツール統合

---

**これがHakoruneの究極のモジュールシステム！** 😺✨

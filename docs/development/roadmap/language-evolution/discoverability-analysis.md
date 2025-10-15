# 🔍 Hakorune 機能発見性（Discoverability）問題の深層分析

**問題**: ChatGPTですら、こんな冗長コードを書いてしまう：
```nyash
hex_digit(ch) {
    if ch == "0" { return 0 }
    if ch == "1" { return 1 }
    if ch == "2" { return 2 }
    // ... 続く16行
}
```

**本来あるべき姿**（既に実装済み機能で書ける）：
```nyash
// 方法1: match式（Phase 12.7実装済み）
hex_digit(ch) {
    return match ch {
        "0" => 0, "1" => 1, "2" => 2, "3" => 3,
        "4" => 4, "5" => 5, "6" => 6, "7" => 7,
        "8" => 8, "9" => 9, "a" => 10, "b" => 11,
        "c" => 12, "d" => 13, "e" => 14, "f" => 15,
        _ => 0
    }
}

// 方法2: Mapリテラル（実装済み）
static box HexUtils {
    hex_map: MapBox
    
    birth() {
        me.hex_map = new MapBox()
        me.hex_map.set("0", 0)
        me.hex_map.set("1", 1)
        // ...
    }
    
    hex_digit(ch) {
        return me.hex_map.get(ch)
    }
}

// 方法3: indexOf（StringBox実装済み）
hex_digit(ch) {
    local hex_chars = "0123456789abcdef"
    return hex_chars.indexOf(ch)
}
```

---

## 📊 根本原因分析（5つの問題）

### **問題1: ドキュメント導線の弱さ**

**現状**:
```
docs/reference/language/LANGUAGE_REFERENCE_2025.md  (1,500行)
  ↓
機能は書いてある
  ↓
でも「いつ使うか」が分からない
```

**不足している情報**:
- ❌ 「こういう場合はmatch式を使う」
- ❌ 「配列ルックアップ vs Map vs match の使い分け」
- ❌ 「Anti-Pattern（やってはいけない書き方）」

### **問題2: サンプルコード不足**

**現状**:
```
apps/examples/  ← 基本的なサンプルのみ
apps/tests/     ← テストケースはあるが学習用ではない
```

**不足**:
- ❌ Cookbook/Recipe集（「～したい時は？」）
- ❌ Best Practice集
- ❌ イディオム集（慣用句）

### **問題3: AI学習データの不足**

**現状**:
```
ChatGPT/Claudeの学習データ:
- JavaScript: 大量
- Python: 大量
- Rust: 中程度
- Nyash/Hakorune: ほぼゼロ
```

**結果**: JavaScriptパターンで書いてしまう
```javascript
// JavaScript風（冗長）
if (x == "a") return 1
if (x == "b") return 2
```

### **問題4: 糖衣構文の発見性**

**実装済みだが知られていない機能**:
- ✅ match式（Phase 12.7）
- ✅ Lambda式
- ✅ ?演算子（Result伝播）
- ✅ Mapリテラル
- ✅ 配列メソッド（indexOf/find等）

**問題**: 「機能がある」ことは分かっても、「いつ使うべきか」が分からない

### **問題5: Linter/Static Analysis不足**

**現状**: コード品質チェックがない
```nyash
// これを書いても警告が出ない（出るべき）
if ch == "0" { return 0 }
if ch == "1" { return 1 }
// ... 続く
```

**理想**: Linterが検出
```
Warning: Consider using match expression instead of if-else chain
  → Suggested refactoring:
    match ch { "0" => 0, "1" => 1, ... }
```

---

## 🎯 解決策（5つの柱）

### **解決策1: Cookbook/Recipe集の作成** 🔴 最優先

**構成案**:
```markdown
docs/cookbook/
├── README.md
├── patterns/
│   ├── lookup-table.md     # ルックアップテーブル実装
│   ├── string-parsing.md   # 文字列解析
│   ├── error-handling.md   # エラー処理パターン
│   └── collections.md      # コレクション操作
├── anti-patterns/
│   ├── if-chain.md         # if連鎖のアンチパターン
│   ├── manual-loop.md      # 手動ループのアンチパターン
│   └── null-check.md       # null チェックのアンチパターン
└── refactoring/
    ├── if-to-match.md      # if → match変換
    ├── loop-to-map.md      # ループ → map/filter変換
    └── manual-to-builtin.md # 手動実装 → ビルトイン変換
```

**例**: `docs/cookbook/patterns/lookup-table.md`
```markdown
# ルックアップテーブルパターン

## ❌ Anti-Pattern（やってはいけない）
```nyash
hex_digit(ch) {
    if ch == "0" { return 0 }
    if ch == "1" { return 1 }
    // ... 冗長
}
```

## ✅ Pattern 1: match式（推奨、シンプルなルックアップ）
```nyash
hex_digit(ch) {
    return match ch {
        "0" => 0, "1" => 1, "2" => 2, ...
        _ => 0
    }
}
```

## ✅ Pattern 2: indexOf（文字列ベース）
```nyash
hex_digit(ch) {
    return "0123456789abcdef".indexOf(ch)
}
```

## ✅ Pattern 3: MapBox（複雑な値）
```nyash
static box HexMap {
    map: MapBox
    birth() {
        me.map = new MapBox()
        me.map.set("zero", 0)
        // ...
    }
}
```

## 使い分け
- シンプルなルックアップ → match式
- 連続文字 → indexOf
- 複雑な値/初期化 → MapBox
```

---

### **解決策2: Quick Reference拡充** 🟡

**現状**: 文法中心
**改善**: Use Case中心に再構成

```markdown
# Use Case別 Quick Reference

## 文字列解析
- hex変換 → indexOf/match
- split → string.split()
- 正規表現 → RegexBox

## ルックアップ
- 固定値 → match
- 動的 → MapBox
- 連続 → indexOf

## 繰り返し処理
- 変換 → array.map()
- フィルタ → array.filter()
- 集約 → array.reduce()
```

---

### **解決策3: サンプルコード集の充実** 🟡

**追加すべきサンプル**:
```
apps/examples/
├── cookbook/
│   ├── hex_parser.hkr      # hex解析の正しい書き方
│   ├── json_parser.hkr     # JSON解析
│   ├── state_machine.hkr   # ステートマシン
│   └── validator.hkr       # バリデーション
├── best-practices/
│   ├── match_vs_if.hkr     # match vs if使い分け
│   ├── map_vs_array.hkr    # Map vs Array選択
│   └── error_handling.hkr  # エラー処理パターン
└── anti-patterns/
    ├── if_chain_bad.hkr    # ❌ 悪い例
    └── if_chain_good.hkr   # ✅ 良い例
```

---

### **解決策4: Linter/フォーマッター導入** 🟢

**Phase 18-20で実装検討**:

```bash
# Linter実行
$ hakorune lint my_code.hkr

Warning: Inefficient if-else chain detected
  --> my_code.hkr:10:5
   |
10 |     if ch == "0" { return 0 }
11 |     if ch == "1" { return 1 }
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^ Consider using match expression
   |
   = help: match ch { "0" => 0, "1" => 1, ... }
```

**検出すべきAnti-Pattern**:
1. If-else連鎖（match式推奨）
2. 手動ループ（map/filter推奨）
3. 未使用変数
4. 型不一致（opt-in型チェック時）

---

### **解決策5: AI学習用データ整備** 🟢

**コンテキスト注入用ファイル**:
```
docs/for-ai/
├── PATTERNS.md          # 推奨パターン集
├── ANTI_PATTERNS.md     # アンチパターン集
├── IDIOMS.md            # 慣用句集
└── COMMON_MISTAKES.md   # よくある間違い
```

**CLAUDE.mdに追加**:
```markdown
## 🤖 AI開発者への注意事項

### ❌ やってはいけないパターン
1. If-else連鎖 → match式を使う
2. 手動for文 → map/filter/reduce
3. null手動チェック → ?演算子

### ✅ 推奨パターン
1. ルックアップ → match/indexOf/MapBox
2. 変換 → array.map()
3. エラー処理 → Result + ?演算子
```

---

## 📅 実装優先順位

### **Phase 17（即座）** 🔴
1. **Cookbook/Recipe集作成**
   - `docs/cookbook/patterns/lookup-table.md`
   - `docs/cookbook/anti-patterns/if-chain.md`
   - `docs/cookbook/refactoring/if-to-match.md`

2. **CLAUDE.md拡充**
   - AI開発者への注意事項セクション追加
   - よくあるアンチパターン明記

### **Phase 18-19** 🟡
3. **サンプルコード集**
   - `apps/examples/cookbook/` 作成
   - ベストプラクティス実装例

4. **Quick Reference再構成**
   - Use Case中心に再編成

### **Phase 20-22** 🟢
5. **Linter/フォーマッター**
   - Anti-Pattern検出
   - 自動リファクタリング提案

---

## 🎯 期待される効果

### **短期効果（Phase 17）**
- ✅ AI（ChatGPT/Claude）がアンチパターンを避ける
- ✅ 新規開発者が正しい書き方を学べる
- ✅ コードレビュー時の指摘減少

### **中期効果（Phase 18-19）**
- ✅ サンプルコードからコピペで正しいコード
- ✅ 言語機能の利用率向上
- ✅ コード品質の一貫性向上

### **長期効果（Phase 20-22）**
- ✅ Linterが自動で品質保証
- ✅ リファクタリングが容易
- ✅ 技術的負債の削減

---

## 💡 名言化

> **「機能があっても、使われなければ意味がない」**
>
> Hakoruneは強力な機能を持つ。
> しかし、その使い方が伝わらなければ宝の持ち腐れ。
> 
> **Cookbook/Recipe集で「発見性」を高める**ことが、
> 次のPhaseの最重要課題。

---

## 🎊 まとめ

ChatGPTが冗長なif連鎖を書いてしまった問題は、
**言語機能の不足ではなく、発見性（Discoverability）の問題**。

**解決策5本柱**:
1. 🔴 Cookbook/Recipe集作成（Phase 17、最優先）
2. 🟡 Quick Reference拡充（Phase 17-18）
3. 🟡 サンプルコード集（Phase 18-19）
4. 🟢 Linter/フォーマッター（Phase 20-22）
5. 🟢 AI学習用データ整備（Phase 17-19）

**Phase 17で即座に着手すべき**:
- `docs/cookbook/` ディレクトリ作成
- Anti-Pattern集の整備
- CLAUDE.mdへのAI開発者向け注意事項追加

これにより、「つよつよ機能」が「実際に使われる」ようになりますにゃ！

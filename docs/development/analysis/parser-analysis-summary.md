# Parser系分析サマリー（1分で読める版）

**作成日**: 2025-10-12
**完全版**: [parser-boxification-optimization-analysis.md](./parser-boxification-optimization-analysis.md)

---

## 🎯 結論

セルフホストコンパイラーのParser（1,401行）は**既に高品質**。
**2つの簡単な改善**で **50-80行削減**（4-6%）可能。

---

## 📊 発見した改善機会

### ✅ 優先度A: 重複ヘルパー統一（必須！）
**問題**: 4つのBoxで同じ関数を重複実装
- `_i2s` (4箇所), `_is_digit` (2箇所), `_is_alpha` (2箇所)
- `_starts_with`, `_index_of`, `_trim`, `_esc_json` (UsingCollectorBox)

**解決策**: `ParserCommonUtilsBox` 作成
```hako
static box ParserCommonUtilsBox {
  i2s(v) { return "" + v }
  is_digit(ch) { return ch >= "0" && ch <= "9" }
  is_alpha(ch) { ... }
  starts_with(src, i, pat) { ... }
  // ... 他の共通関数
}
```

**効果**: **30-50行削減** / 工数: 2-3時間 / リスク: 低

---

### ✅ 優先度B: 位置解析パターン統一
**問題**: 21箇所で同じ「`@`区切り解析」コード
```hako
// これが21回出現
local at = idp.lastIndexOf("@")
local name = idp.substring(0, at)
local pos = ctx.to_int(idp.substring(at+1, idp.size()))
```

**解決策**: `ParserCommonUtilsBox` に追加
```hako
// 返り値: "content|position"
split_at_mark(str) {
  local at = str.lastIndexOf("@")
  if at < 0 { return str + "|0" }
  return str.substring(0, at) + "|" + str.substring(at+1, str.size())
}
```

**効果**: **20-30行削減** / 工数: 3-4時間 / リスク: 中

---

### 🔍 優先度C: その他（将来検討）
- **JSON Builder**: 可読性向上（削減なし）
- **ループ最適化**: 5-10%高速化見込み（要計測）

---

## 🚀 推奨アクション

### 今すぐ実施
```bash
# 1. ParserCommonUtilsBox 作成（2時間）
# 2. 4ファイル修正（1時間）
#    - ParserIdentScanBox
#    - ParserNumberScanBox
#    - ParserStringScanBox
#    - UsingCollectorBox
# 3. テスト実行
tools/smokes/v2/run.sh --profile quick
```

**期待効果**: 30-50行削減、保守性向上

### 1週間以内
- Phase 2（位置解析統一）: さらに20-30行削減

---

## 📈 インパクト

| 改善 | 削減 | 工数 | ROI |
|-----|------|------|-----|
| Phase 1 | 30-50行 | 2-3h | ⭐⭐⭐⭐⭐ |
| Phase 2 | 20-30行 | 3-4h | ⭐⭐⭐⭐ |
| **合計** | **50-80行** | **5-7h** | - |

---

## 💡 その他の観察

### 優秀な点
- ✅ Box分離が明確（Scanner/Parser/Control/Exception）
- ✅ Progress Guard による無限ループ防止
- ✅ 委譲パターンの活用

### 改善余地
- ⚠️ エラーハンドリング（degradation方式）
- ⚠️ ドキュメント不足

---

**詳細**: [完全分析レポート](./parser-boxification-optimization-analysis.md)

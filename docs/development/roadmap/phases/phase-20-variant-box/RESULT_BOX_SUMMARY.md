# Result<T,E> Box 設計サマリー

**作成日**: 2025-10-08
**完全設計**: [RESULT_BOX_COMPLETE_DESIGN.md](./RESULT_BOX_COMPLETE_DESIGN.md)

---

## 🎯 1分でわかるResult<T,E> Box

### 現状（34行）
```hakorune
box ResultBox {
  _val: Box
  _err: StringBox
  _ok: IntegerBox

  is_ok() { return me._ok }
  value() { return me._val }
  error() { return me._err }
  unwrap_or(def) { if me._ok == 1 { return me._val } return def }
}

static box Result {
  ok(v) { ... }
  err(msg) { ... }
}
```

**問題点**:
- ❌ unwrap()なし（panicする基本メソッド不在）
- ❌ is_err()なし（`!r.is_ok()` で代用）
- ❌ expect()なし（カスタムメッセージ付きpanic不在）

### Phase 1: MVP版（84行）

**追加メソッド**:
```hakorune
is_err()              // エラー判定
unwrap()              // 値取得、エラー時panic
expect(msg)           // カスタムメッセージ付きpanic
unwrap_err()          // エラー取得、成功時panic
debug()               // デバッグ文字列表現
```

**既存互換性**: ✅ 100%（4メソッド完全互換）

**工数**: 2-3時間

---

## 📊 API比較表

| メソッド | 旧版 | Phase 1 | Rust std::result |
|---------|------|---------|------------------|
| `is_ok()` | ✅ | ✅ | ✅ |
| `is_err()` | ❌ | ✅ | ✅ |
| `unwrap()` | ❌ | ✅ | ✅ |
| `expect(msg)` | ❌ | ✅ | ✅ |
| `unwrap_or(def)` | ✅ | ✅ | ✅ |
| `unwrap_err()` | ❌ | ✅ | ✅ |
| `map(fn)` | ❌ | ❌ | ✅ (Phase 2) |
| `and_then(fn)` | ❌ | ❌ | ✅ (Phase 2) |

---

## 🚀 使用例

### Before（旧版）
```hakorune
local r = Result.ok(42)
local v = r.value()
if v == null {
  print("Error!")
  return 1
}
print("Value: " + v)
```

### After Phase 1（推奨）
```hakorune
local r = Result.ok(42)
local v = r.unwrap()  // エラー時は自動パニック
print("Value: " + v)
```

### After Phase 1（カスタムメッセージ）
```hakorune
local r = decode_phi(seg)
local v = r.expect("Failed to decode PHI")  // より詳細なエラー
```

---

## 📋 実装チェックリスト（Phase 1）

### コア実装
- [ ] `is_err()` 追加（4行）
- [ ] `unwrap()` 追加（8行）
- [ ] `expect(msg)` 追加（8行）
- [ ] `unwrap_err()` 追加（8行）
- [ ] `debug()` 追加（6行）

### テスト
- [ ] `apps/tests/result_box_extended.hako` 作成（100-150行）
- [ ] スモークテスト追加

### ドキュメント
- [ ] `docs/reference/boxes-system/result-box.md` 作成（150-200行）
- [ ] エラーハンドリングガイド更新

### 互換性確認
- [ ] 既存5箇所の動作確認（変更不要）
- [ ] 全スモークテスト通過

---

## 🎯 段階導入戦略

### Phase 1: MVP版（即座実装）
- **優先度**: P0
- **工数**: 2-3時間
- **価値**: 80%の機能（unwrap/expectが最重要）

### Phase 2: 関数型拡張（Phase 21後）
- **優先度**: P2
- **工数**: 1-2時間
- **価値**: 20%（map/and_then、任意実装）

### Phase 3: VariantBox統合（Phase 20.6後）
- **優先度**: P1
- **工数**: 3-5時間
- **価値**: 型安全性向上、パターンマッチング対応

---

## 🔑 重要設計判断

### 1. panic実装（疑似panic採用）

**問題**: Hakoruneに `panic()` がない

**解決**: `print("[PANIC] ...") + return null`

```hakorune
unwrap() {
  if me._ok == 0 {
    print("[PANIC] Result.unwrap() called on Err: " + me._err)
    return null
  }
  return me._val
}
```

**将来**: Phase 25でPanicBox統合

### 2. 後方互換性100%維持

**原則**: 既存4メソッドは一切変更しない

**保証**:
- `is_ok()` - 完全互換
- `value()` - 完全互換
- `error()` - 完全互換
- `unwrap_or(def)` - 完全互換

**実証**: 既存5箇所が無変更で動作

### 3. 関数型拡張は後回し

**理由**:
- Hakoruneに第一級関数がない（2025-10-08時点）
- Phase 1で80%の価値を提供可能
- Phase 21完了まで保留

---

## 📖 関連ドキュメント

### 設計書
- **[RESULT_BOX_COMPLETE_DESIGN.md](./RESULT_BOX_COMPLETE_DESIGN.md)** - 完全設計（本文書）
- **[result_box_v2_reference.hako](./result_box_v2_reference.hako)** - 参照実装（350行）
- **[RESULT_BOX_MIGRATION_PLAN.md](./RESULT_BOX_MIGRATION_PLAN.md)** - 移行計画

### 参考資料
- **[DESIGN.md](./DESIGN.md)** - VariantBox設計（Phase 20.6）
- **[apps/selfhost/vm/boxes/result_box.hako](../../../../apps/selfhost/vm/boxes/result_box.hako)** - 現在の実装（34行）

---

## ✅ 次のステップ

### 即座実行（2-3時間）

1. ✅ 設計完了
2. ⬜ `apps/selfhost/vm/boxes/result_box.hako` 編集
   - 5つのメソッド追加（34行 → 84行）
3. ⬜ テストファイル作成
4. ⬜ スモークテスト実行
5. ⬜ ドキュメント作成
6. ⬜ コミット

---

## 🎓 成功の鍵

✅ **80/20ルール**: Phase 1で80%の価値（unwrap/expectが最重要）
✅ **後方互換性**: 既存5箇所は一切変更不要
✅ **段階導入**: MVP → 拡張 → VariantBox統合
✅ **テスト駆動**: 既存テスト + 新規テスト
✅ **ドキュメント先行**: 使い方を先に明確化

---

**承認**: Phase 1実装開始可能（2-3時間見込み）

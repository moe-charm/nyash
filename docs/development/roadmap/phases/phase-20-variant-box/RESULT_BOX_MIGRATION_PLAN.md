# Result<T,E> Box 移行計画

**作成日**: 2025-10-08
**基準設計**: [RESULT_BOX_COMPLETE_DESIGN.md](./RESULT_BOX_COMPLETE_DESIGN.md)
**参照実装**: [result_box_v2_reference.hako](./result_box_v2_reference.hako)

---

## 📋 移行全体計画

### フェーズ概要

```
Phase 1 (MVP版)        → 即座実装可能（2-3時間）
  ├─ 既存互換100%
  ├─ 基本拡張（is_err/unwrap/expect）
  └─ テスト追加

Phase 2 (関数型拡張)    → Phase 21後（1-2時間）
  ├─ map/and_then/or_else
  └─ 第一級関数前提

Phase 3 (VariantBox統合) → Phase 20.6後（3-5時間）
  ├─ @enum Result
  ├─ パターンマッチング
  └─ 旧版廃止
```

---

## Phase 1: MVP版実装チェックリスト

### 🎯 目標
- 既存互換性100%維持
- 基本拡張メソッド追加（is_err/unwrap/expect/unwrap_err/debug）
- 工数: 2-3時間

### タスクリスト

#### 1. コア実装（selfhost/vm/boxes/result_box.hako）

- [ ] **1.1 is_err() 追加**
  - コード: 4行
  - テスト: `test_is_err()`
  - 既存互換: ✅ 影響なし

- [ ] **1.2 unwrap() 追加**
  - コード: 8行
  - テスト: `test_unwrap_ok()`, `test_unwrap_err()`
  - 疑似panic実装（print + return null）
  - 既存互換: ✅ 影響なし

- [ ] **1.3 expect(msg) 追加**
  - コード: 8行
  - テスト: `test_expect_ok()`, `test_expect_err()`
  - カスタムメッセージ対応
  - 既存互換: ✅ 影響なし

- [ ] **1.4 unwrap_err() 追加**
  - コード: 8行
  - テスト: `test_unwrap_err_ok()`, `test_unwrap_err_err()`
  - 既存互換: ✅ 影響なし

- [ ] **1.5 debug() 追加**
  - コード: 6行
  - テスト: `test_debug()`
  - 既存互換: ✅ 影響なし

#### 2. テストファイル作成

- [ ] **2.1 apps/tests/result_box_extended.hako 作成**
  - 基本テスト（is_ok/is_err）
  - unwrap系テスト（unwrap/expect/unwrap_err）
  - unwrap_or テスト（既存互換確認）
  - debug テスト
  - 想定行数: 100-150行

#### 3. スモークテスト追加

- [ ] **3.1 tools/smokes/v2/profiles/quick/selfhost/result_box_extended_vm.sh 作成**
  - 基本動作確認
  - unwrap() 正常系
  - expect() エラー系
  - 既存テストとの互換性確認

#### 4. ドキュメント作成

- [ ] **4.1 docs/reference/boxes-system/result-box.md 作成**
  - API完全リファレンス
  - 使用例
  - 移行ガイド
  - 想定行数: 150-200行

- [ ] **4.2 docs/guides/error-handling.md 更新**
  - Resultの基本的な使い方
  - unwrap vs unwrap_or の使い分け
  - expect のベストプラクティス

#### 5. 既存コード互換性確認

- [ ] **5.1 既存5箇所の動作確認**
  - selfhost/vm/boxes/phi_decode_box.hako (2箇所)
  - tools/smokes/.../selfhost_utils_result_box_vm.sh (1箇所)
  - 他2箇所
  - 変更不要であることを確認

#### 6. 最終確認

- [ ] **6.1 コードレビュー**
  - 既存APIの互換性100%
  - 新APIの動作確認
  - エラーメッセージの適切性

- [ ] **6.2 テスト実行**
  - 全スモークテスト通過
  - 新規テスト通過
  - 既存テスト通過

- [ ] **6.3 コミット**
  - コミットメッセージ: "feat(box): Result<T,E> MVP版実装 - unwrap/expect追加"
  - 変更ファイル:
    - selfhost/vm/boxes/result_box.hako (34→84行)
    - apps/tests/result_box_extended.hako (新規)
    - tools/smokes/v2/profiles/quick/selfhost/result_box_extended_vm.sh (新規)
    - docs/reference/boxes-system/result-box.md (新規)

---

## Phase 2: 関数型拡張チェックリスト

**前提条件**: Phase 21（関数型機能）完了

### タスクリスト

- [ ] **2.1 map(fn) 実装**
  - 関数オブジェクト対応
  - テスト追加

- [ ] **2.2 map_err(fn) 実装**
  - テスト追加

- [ ] **2.3 and_then(fn) 実装**
  - テスト追加

- [ ] **2.4 or_else(fn) 実装**
  - テスト追加

- [ ] **2.5 ドキュメント更新**
  - 関数型メソッドの使用例追加

---

## Phase 3: VariantBox統合チェックリスト

**前提条件**: Phase 20.6（VariantBox）完了

### タスクリスト

#### 3.1 並行運用開始

- [ ] **3.1.1 @enum Result 実装**
  - マクロ脱糖確認
  - Result.Ok(v) / Result.Err(e) 動作確認

- [ ] **3.1.2 旧版を result_box.hako → result_box_legacy.hako にリネーム**
  - using文の更新（legacy版を明示）

#### 3.2 段階移行（5箇所）

- [ ] **3.2.1 phi_decode_box.hako 移行**
  - Result.ok() → Result.Ok()
  - Result.err() → Result.Err()
  - テスト確認

- [ ] **3.2.2 残り4箇所の移行**
  - 各箇所でテスト確認

#### 3.3 旧版廃止

- [ ] **3.3.1 result_box_legacy.hako 削除**
  - すべてのusingが新版を使用していることを確認

- [ ] **3.3.2 ドキュメント更新**
  - VariantBoxベースの使用例追加
  - @match パターンマッチング例追加

---

## 互換性マトリックス

### API互換性

| メソッド | 旧版 | Phase 1 | Phase 2 | Phase 3 |
|---------|------|---------|---------|---------|
| `is_ok()` | ✅ | ✅ | ✅ | ✅ |
| `is_err()` | ❌ | ✅ | ✅ | ✅ |
| `value()` | ✅ | ✅ | ✅ | ⚠️ 非推奨 |
| `error()` | ✅ | ✅ | ✅ | ⚠️ 非推奨 |
| `unwrap()` | ❌ | ✅ | ✅ | ✅ |
| `expect(msg)` | ❌ | ✅ | ✅ | ✅ |
| `unwrap_err()` | ❌ | ✅ | ✅ | ✅ |
| `unwrap_or(def)` | ✅ | ✅ | ✅ | ✅ |
| `map(fn)` | ❌ | ❌ | ✅ | ✅ |
| `and_then(fn)` | ❌ | ❌ | ✅ | ✅ |
| `@match` | ❌ | ❌ | ❌ | ✅ |

### コンストラクタ互換性

| 構文 | 旧版 | Phase 1 | Phase 2 | Phase 3 |
|------|------|---------|---------|---------|
| `Result.ok(v)` | ✅ | ✅ | ✅ | ⚠️ 非推奨 |
| `Result.err(msg)` | ✅ | ✅ | ✅ | ⚠️ 非推奨 |
| `Result.Ok(v)` | ❌ | ❌ | ❌ | ✅ |
| `Result.Err(e)` | ❌ | ❌ | ❌ | ✅ |

---

## 既存コード移行例

### 移行パターン1: 基本的な使い方

#### Before（旧版）
```hakorune
using "selfhost/vm/boxes/result_box.hako" as Result

local r = Result.ok(42)
local v = r.value()
if v == null {
  print("Error")
}
```

#### After Phase 1（MVP版）
```hakorune
using "selfhost/vm/boxes/result_box.hako" as Result

local r = Result.ok(42)
local v = r.unwrap()  // エラー時は自動パニック
```

#### After Phase 3（VariantBox版）
```hakorune
@enum Result {
  Ok(value)
  Err(error)
}

local r = Result.Ok(42)
@match r {
  Ok(v) => print("Value: " + v)
  Err(e) => print("Error: " + e)
}
```

### 移行パターン2: エラーハンドリング

#### Before（旧版）
```hakorune
local r = decode_phi(seg)
if r.is_ok() == 1 {
  return r.value()
} else {
  print("Error: " + r.error())
  return null
}
```

#### After Phase 1（MVP版）
```hakorune
local r = decode_phi(seg)
return r.expect("Failed to decode PHI")  // カスタムメッセージ
```

#### After Phase 3（VariantBox版）
```hakorune
local r = decode_phi(seg)
@match r {
  Ok(v) => return v
  Err(e) => {
    print("Error: " + e)
    return null
  }
}
```

---

## テスト戦略

### 1. 既存テスト継続実行

**目的**: 後方互換性確認

```bash
# 既存スモークテスト
bash tools/smokes/v2/profiles/quick/selfhost/selfhost_utils_result_box_vm.sh
```

**期待結果**: 100%成功（変更なし）

### 2. 新規テスト追加

**目的**: 新機能の動作確認

```bash
# 新規拡張メソッドテスト
bash tools/smokes/v2/profiles/quick/selfhost/result_box_extended_vm.sh
```

**カバレッジ**:
- is_err() 動作確認
- unwrap() 正常系/エラー系
- expect() カスタムメッセージ
- unwrap_err() 正常系/エラー系
- debug() 文字列表現

### 3. 統合テスト

**目的**: 実際のユースケース確認

```bash
# PHI decodeでの使用確認
bash tools/smokes/v2/profiles/quick/selfhost/phi_decode_integration_vm.sh
```

---

## リスク管理

### リスク1: 既存コードの破壊

**確率**: 低
**影響**: 高
**対策**:
- ✅ 既存メソッドは一切変更しない
- ✅ 新規メソッドのみ追加
- ✅ 既存テストを全実行

### リスク2: panic実装の不完全性

**確率**: 高（Hakoruneにpanic()がない）
**影響**: 中
**対策**:
- ✅ 疑似panic（print + return null）で代替
- ⬜ Phase 25でPanicBox統合
- ⬜ ドキュメントで制約を明記

### リスク3: 関数型拡張の遅延

**確率**: 中
**影響**: 低
**対策**:
- ✅ Phase 1で80%の価値を提供（map不要）
- ✅ Phase 2は任意実装（必須ではない）

---

## 成功基準

### Phase 1 成功基準

✅ **機能**:
- is_err/unwrap/expect/unwrap_err/debug が動作
- 既存5箇所が無変更で動作

✅ **品質**:
- 全スモークテスト通過
- ドキュメント完備

✅ **パフォーマンス**:
- 既存コードと同等（劣化なし）

### Phase 2 成功基準

✅ **機能**:
- map/and_then/or_else が動作
- 関数型プログラミングスタイル対応

### Phase 3 成功基準

✅ **機能**:
- @enum Result 動作
- @match パターンマッチング対応
- 旧版完全廃止

---

## タイムライン

```
Week 1: Phase 1 実装
  Day 1-2: is_err/unwrap/expect 実装
  Day 3:   テスト作成
  Day 4:   ドキュメント作成
  Day 5:   レビュー・コミット

Week 2-N: Phase 21（関数型機能）待機

Week N+1: Phase 2 実装
  Day 1:   map/and_then 実装
  Day 2:   テスト・ドキュメント
  Day 3:   レビュー・コミット

Week N+2-M: Phase 20.6（VariantBox）待機

Week M+1: Phase 3 実装
  Day 1-2: @enum Result 実装
  Day 3-4: 既存5箇所移行
  Day 5:   旧版廃止・ドキュメント更新
```

---

## Next Steps

### 即座実行タスク（Phase 1）

1. ✅ 設計完了（本ドキュメント）
2. ⬜ result_box.hako 編集開始
   - is_err() 追加
   - unwrap() 追加
   - expect() 追加
   - unwrap_err() 追加
   - debug() 追加
3. ⬜ テストファイル作成
4. ⬜ スモークテスト実行
5. ⬜ ドキュメント作成
6. ⬜ コミット

**見積もり**: 2-3時間

---

**承認**: 移行計画完了、Phase 1実装開始可能

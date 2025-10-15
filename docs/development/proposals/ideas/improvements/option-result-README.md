# Option/Result 実装プロジェクト - 総合ガイド

**作成日**: 2025-10-08
**ステータス**: 設計完了、実装準備完了
**目的**: HakoruneにRust/Swift風のOption<T>とResult<T,E>を導入

---

## 📚 ドキュメント構成

### 🚀 すぐ始めたい人向け
- **[クイックスタートガイド](./option-result-quick-start.md)** ⭐最優先
  - 30分で動作確認する方法
  - 最小実装コード（コピペ可）
  - よくある失敗と対策

### 📋 完全な計画を知りたい人向け
- **[テスト戦略＋スモークテスト統合](./option-result-test-strategy.md)**
  - 25テストケース詳細
  - スモークテスト統合手順
  - Phase 15.11成功事例の適用
  - 失敗事例からの学び

---

## 🎯 プロジェクト概要

### 目的
Hakorune言語にモダンなエラーハンドリング機能を追加:
- **Option<T>**: 値の有無を表現（nullチェックの型安全版）
- **Result<T,E>**: 成功/失敗を表現（例外の型安全版）

### 既存実装との関係
- **既存**: `selfhost/vm/boxes/result_box.hako` （基本的なResultBox）
- **新規**: `apps/lib/boxes/option_std.hako` （Option実装）
- **拡張**: `apps/lib/boxes/result_std.hako` （Resultの高階関数追加）

### 想定工数
- **最小実装（MVP）**: 3.5時間（楽観的）～ 7時間（現実的）
- **完全実装**: 7時間（楽観的）～ 11時間（現実的）

---

## 🏗️ 実装計画

### Phase 1: 最小実装（MVP）【優先度: P0】
**目標**: 基本的なOption/Resultが動作する

**成果物**:
```
apps/lib/boxes/option_std.hako          # Option実装
apps/tests/test_option_minimal.hako     # 最小テスト
```

**機能**:
- OptionBox: some/none/is_some/is_none/value/unwrap_or
- Opt static box: some/none ファクトリ

**完了条件**:
- テスト実行で "ALL_TESTS_PASSED" 表示
- MIR出力正常

**所要時間**: 1-2時間

### Phase 2: 完全実装【優先度: P1】
**目標**: 高階関数・相互変換を含む完全な実装

**成果物**:
```
apps/lib/boxes/option_std.hako          # Option完全版
apps/lib/boxes/result_std.hako          # Result完全版
apps/tests/test_option_basic.hako       # Option 10パターンテスト
apps/tests/test_result_basic.hako       # Result 10パターンテスト
apps/tests/test_option_result_combined.hako  # 組み合わせ5パターン
```

**機能**:
- OptionBox: map/and_then/filter/ok_or
- ResultBox: map/map_err/and_then/or_else/ok

**完了条件**:
- 25テストケースすべてPASS

**所要時間**: 4-6時間

### Phase 3: スモークテスト統合【優先度: P1】
**目標**: CI/CDパイプラインに統合

**成果物**:
```
tools/smokes/v2/profiles/quick/core/option_basic_vm.sh
tools/smokes/v2/profiles/quick/core/result_basic_vm.sh
```

**完了条件**:
- `tools/smokes/v2/run.sh --profile quick` でPASS

**所要時間**: 1-2時間

---

## 🧪 テストケース概要

### Option<T> テスト（10パターン）
1. Some作成と値取得
2. None作成と確認
3. unwrap_or（Some時）
4. unwrap_or（None時）
5. map（Some時）
6. map（None時）
7. and_then（Some時）
8. and_then（None時）
9. filter（条件一致）
10. filter（条件不一致）

### Result<T,E> テスト（10パターン）
1. Ok作成と値取得
2. Err作成と確認
3. unwrap_or（Ok時）
4. unwrap_or（Err時）
5. map（Ok時）
6. map（Err時）
7. map_err（Err時）
8. and_then（Ok時）
9. or_else（Err時）
10. 境界条件（空文字列エラー）

### 組み合わせテスト（5パターン）
1. Option → Result変換（Some）
2. Option → Result変換（None）
3. Result → Option変換（Ok）
4. Result → Option変換（Err）
5. チェーニング（Option → Result → Option）

詳細: [テスト戦略](./option-result-test-strategy.md#1-テストケース設計)

---

## 📊 成功事例と失敗事例

### ✅ Phase 15.11 成功事例（参考にすべき点）
**StringHelpers共通ライブラリ箱化**

**成功要因**:
1. ✅ テスト先行作成（`test_string_helpers.hako`）
2. ✅ 包括的カバレッジ（7種類すべてテスト）
3. ✅ 即座確認（各メソッド実装後にテスト）
4. ✅ 期待出力明確化

**成果**: 14ファイル統合、335行純削減

**適用方法**:
- Option/Resultも同じパターンで進める
- テスト→実装→確認のサイクルを徹底

### ❌ Phase 2.1 失敗事例（避けるべき点）
**dep_tree統合**

**失敗要因**:
1. ❌ テスト実行0回成功
2. ❌ 構文エラー4回連続
3. ❌ 見積もり精度18%（108-150行→20行削減）
4. ❌ 根本原因調査なし

**教訓**:
- 中間テスト必須
- 構文制約の事前確認
- 調査優先（試行錯誤禁止）

詳細: [テスト戦略](./option-result-test-strategy.md#4-失敗事例からの学び)

---

## 🔧 実装パターン

### 推奨パターン（Phase 15.11準拠）

```nyash
// ✅ パターン1: Box + Static Boxの分離
box OptionBox {
  _val: Box
  _some: IntegerBox

  birth() {
    me._val = null
    me._some = 0
  }

  is_some() { return me._some }
  // ... その他のメソッド
}

static box Opt {
  some(v) {
    local opt = new OptionBox()
    opt._val = v
    opt._some = 1
    return opt
  }

  none() {
    local opt = new OptionBox()
    return opt
  }
}
```

### 避けるべきパターン

```nyash
// ❌ NG: セミコロン区切り
me._val = null  me._some = 0

// ❌ NG: continue使用（Hakorune未サポート）
loop(i < n) {
  if skip { continue }
  process(i)
}

// ❌ NG: using文で相対パス
using "./option_std.hako"  // パースエラー

// ✅ OK: hako.tomlに定義
[modules]
"std.option" = "apps/lib/boxes/option_std.hako"
```

---

## 🚀 実装手順（推奨）

### ステップ1: 準備（10分）
```bash
# ドキュメント確認
cat docs/development/proposals/ideas/improvements/option-result-quick-start.md

# 既存実装確認
cat selfhost/vm/boxes/result_box.hako
cat apps/lib/boxes/string_std.hako

# Phase 15.11成功事例確認
cat apps/selfhost/test_string_helpers.hako
```

### ステップ2: 最小実装（1-2時間）
```bash
# テスト作成
# → apps/tests/test_option_minimal.hako

# 実装作成
# → apps/lib/boxes/option_std.hako

# 動作確認
./target/release/hako apps/tests/test_option_minimal.hako
# 期待: "ALL_TESTS_PASSED"
```

### ステップ3: 完全実装（4-6時間）
```bash
# テスト拡張
# → apps/tests/test_option_basic.hako (10パターン)
# → apps/tests/test_result_basic.hako (10パターン)

# 実装拡張
# → apps/lib/boxes/option_std.hako (map/and_then/filter等)
# → apps/lib/boxes/result_std.hako (map/map_err/and_then等)

# 中間確認（各メソッド実装後）
./target/release/hako apps/tests/test_option_basic.hako
./target/release/hako apps/tests/test_result_basic.hako
```

### ステップ4: スモークテスト統合（1-2時間）
```bash
# スモークテスト作成
# → tools/smokes/v2/profiles/quick/core/option_basic_vm.sh
# → tools/smokes/v2/profiles/quick/core/result_basic_vm.sh

# 実行確認
bash tools/smokes/v2/profiles/quick/core/option_basic_vm.sh
bash tools/smokes/v2/profiles/quick/core/result_basic_vm.sh

# 全体確認
tools/smokes/v2/run.sh --profile quick
```

---

## 🐛 デバッグ方法

### 基本デバッグ
```bash
# MIR出力確認
./target/release/hako --dump-mir apps/tests/test_option_minimal.hako

# トレースログ確認
export HAKO_VM_TRACE="op=call,boxcall;regs=1"
export NYASH_DISABLE_PLUGINS=1
./target/release/hakorune apps/tests/test_option_minimal.hako 2>&1
```

### よくあるエラーと対策
| エラー | 原因 | 対策 |
|--------|------|------|
| `Expected identifier` | using文の構文エラー | hako.tomlに追加 |
| `Unexpected token ';'` | セミコロン区切り使用 | 複数行に分割 |
| `Unexpected token 'fn'` | lambda構文未実装 | 通常の関数に変更 |

詳細: [クイックスタート](./option-result-quick-start.md#よくある失敗と対策)

---

## 📈 進捗管理

### チェックリスト

#### 準備フェーズ
- [ ] クイックスタートガイド読了
- [ ] テスト戦略読了
- [ ] Phase 15.11成功事例確認
- [ ] 既存ResultBox確認

#### 実装フェーズ
- [ ] 最小テスト作成完了
- [ ] 最小実装作成完了
- [ ] 最小実装動作確認（"ALL_TESTS_PASSED"）
- [ ] 完全テスト作成完了（25パターン）
- [ ] 完全実装作成完了
- [ ] 全テストPASS確認

#### 統合フェーズ
- [ ] スモークテスト2本作成完了
- [ ] スモークテスト個別実行PASS
- [ ] `tools/smokes/v2/run.sh --profile quick` PASS
- [ ] ドキュメント更新完了

### 見積もり精度管理

**Phase 2.1の失敗を繰り返さない**:
- 見積もり: 現実的（7時間）
- 実際: ____時間
- 精度: ____%（目標: 80%以上）

**中間確認ポイント**:
- [ ] 1時間後: 最小実装動作確認
- [ ] 3時間後: Option完全実装50%完了
- [ ] 5時間後: Result完全実装50%完了
- [ ] 7時間後: スモークテスト統合完了

---

## 🎯 完了条件

### 最小成功（MVP）
- [ ] Option基本操作（some/none/is_some/is_none/unwrap_or）が動作
- [ ] Result基本操作（ok/err/is_ok/value/error/unwrap_or）が動作
- [ ] テスト実行で "ALL_TESTS_PASSED" 表示

### 完全成功
- [ ] 25テストケースすべてPASS
- [ ] スモークテスト2本PASS
- [ ] MIR出力正常
- [ ] トレースログで内部動作確認
- [ ] ドキュメント更新完了

---

## 📖 関連リソース

### プロジェクトドキュメント
- [クイックスタートガイド](./option-result-quick-start.md) - 今すぐ始める
- [テスト戦略](./option-result-test-strategy.md) - 完全な計画

### 参考実装
- `selfhost/vm/boxes/result_box.hako` - 既存ResultBox
- `apps/lib/boxes/string_std.hako` - StringStd（パターン参考）
- `apps/selfhost/test_string_helpers.hako` - Phase 15.11テスト

### 関連ドキュメント
- [CLAUDE.md](../../../../../CLAUDE.md) - 開発方針
- [Phase 15.11 README](../../../../roadmap/phases/phase-15.11/README.md) - 成功事例（※存在しない場合はCLAUDE.md参照）
- [スモークテストガイド](../../../../../tools/smokes/README.md) - テスト体系

---

## 🚀 次のアクション

**今すぐ開始する手順**:
1. [クイックスタートガイド](./option-result-quick-start.md)のステップ1を実行（5分）
2. ステップ2を実行（10分）
3. ステップ3で動作確認（5分）

**所要時間**: 20分で最初の動作確認完了

---

**作成者**: Claude (2025-10-08)
**レビュー**: 未実施
**ステータス**: 設計完了、実装準備完了

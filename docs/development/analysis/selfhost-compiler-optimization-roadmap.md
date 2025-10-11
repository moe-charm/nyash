# セルフホストコンパイラー 最適化ロードマップ

**エグゼクティブサマリー** - 2025-10-12

---

## 🎯 1分でわかる核心

### 現状
- **総行数**: 5,733行 (71ファイル)
- **重複コード**: 1,091行 (19.0%)
- **Everything is Box準拠度**: 65/100

### 問題
- 文字列操作が30+ファイルに散在
- JsonProgramBox が巨大化 (531行、全体の9.3%)
- 型安全アクセサ (MapHelpersBox) の採用率30%

### 解決策
**StringUtilsBox統合** で **220-370行削減** (最優先)

### 効果
- 削減率: 5.7-9.4%
- Everything is Box準拠度: 65% → 85-90%
- 保守性: 単一ソース原則確立

---

## 📊 優先度付き推奨事項

| 優先度 | 施策 | 削減行数 | 工数 | ROI |
|--------|------|---------|------|-----|
| **P0 超優先** | StringUtilsBox統合 | 220-370行 | 20-30h | ★★★★★ |
| **P1 優先** | JsonUtilsBox抽出 | 50-80行 | 10-15h | ★★★★☆ |
| **P2 中** | MapHelpersBox拡張 | 50-80行 | 10-15h | ★★★☆☆ |
| **P3 低** | DebugBox改善 | 5-10行 | 5-10h | ★☆☆☆☆ |

**合計**: 325-540行削減 (5.7-9.4%)、45-70時間

---

## 🚀 即座の推奨アクション (今日から)

### アクション1: クイックウィン実施 (5ファイル、11-16時間)

**対象ファイル** (影響小・効果大):
1. ParserStringUtilsBox (83行) → **63行削減** (76%!)
2. MirEmitterBox (230行) → 30行削減
3. regex_flow.hako (103行) → 15-20行削減
4. builder/ssa/local.hako (122行) → 10-15行削減
5. builder/ssa/cond_inserter.hako (118行) → 10-15行削減

**合計削減**: 128-143行 (2.2-2.5%)

**手順**:
```bash
# Step 1: StringHelpers.hako を拡張 (既存ファイル)
apps/selfhost/common/string_helpers.hako
  ↓ 追加機能:
  - index_of, last_index_of, starts_with
  - trim, skip_ws
  - is_digit, is_alpha, is_space

# Step 2: 5ファイルを段階的移行
1. ParserStringUtilsBox.hako を更新
2. スモークテスト実行
3. 次のファイルへ (5回繰り返し)

# Step 3: 回帰テスト
tools/smokes/v2/run.sh --profile quick
```

**期待効果**:
- ✅ 即座の成果実感 (2-3日で完了)
- ✅ リスク低い (局所的変更)
- ✅ 次フェーズへの足がかり

---

### アクション2: Phase 1全体の着手判断 (Week 1-2、30-45時間)

**Week 1: StringUtilsBox統合** (20-30時間)
- 対象: 30+ファイル
- 削減: 220-370行

**Week 2: JsonUtilsBox抽出** (10-15時間)
- 対象: JsonProgramBox (531行 → 330行)
- 削減: 50-80行

**判断基準**:
- ✅ クイックウィン成功
- ✅ スモークテスト安定
- ✅ 工数確保可能

---

## 📈 段階的削減計画

### 現在 → 最終形態

```
【現在】 5,733行、Everything is Box準拠度 65%
  ↓
【Phase 1-Week1】 StringUtilsBox統合
  ├─ 削減: 220-370行
  └─ 5,363-5,513行、準拠度 70%
  ↓
【Phase 1-Week2】 JsonUtilsBox抽出
  ├─ 削減: 50-80行
  └─ 5,283-5,463行、準拠度 75%
  ↓
【Phase 2】 MapHelpersBox拡張
  ├─ 削減: 50-80行
  └─ 5,203-5,413行、準拠度 85%
  ↓
【Phase 3】 DebugBox改善
  ├─ 削減: 5-10行
  └─ 5,193-5,408行、準拠度 90%
  ↓
【最終】 5,193-5,408行 (▼5.7-9.4%)
        Everything is Box準拠度 85-90%
```

---

## 🎓 重要な学び: Everything is Box の成功法則

### 成功パターン

#### パターン1: 小さく、責務明確なBox
**実例**: MirEmitBox, CallEmitBox (14-38行)

**成功要因**:
- ✅ 1つのBoxが1つのMIR命令カテゴリを担当
- ✅ 薄いファサード (複雑なロジックなし)
- ✅ 統一的なインターフェース (make_XXX命名規則)

**教訓**:
> 小さく始めて、巨大化させない

---

#### パターン2: 状態を持つ段階的構築Box
**実例**: UsingResolverBox (249行)

**成功要因**:
- ✅ birth() でマップ初期化
- ✅ 段階的構築 (load → load → upgrade)
- ✅ パイプラインで一貫使用 (25+箇所)

**教訓**:
> 状態管理 + 段階的構築 + パイプライン統合

---

### 失敗パターン

#### アンチパターン: 作っただけで採用されないBox
**実例**: DebugBox (39行、使用3箇所のみ)

**失敗要因**:
- ❌ 採用率低い (3箇所)
- ❌ 非効率 (ConsoleBox毎回生成)
- ❌ 一貫性なし (他は直接print使用)

**教訓**:
> Boxを作っただけでは不十分。採用促進と標準化が必須。

---

## 🔍 技術詳細: 最大のボトルネック

### JsonProgramBox (531行) の責務過多

**現状の責務** (4つ):
1. JSON v0 正規化 (200行)
2. JSON読み取りユーティリティ (150行)
3. 文字列操作 (100行)
4. メタデータ注入 (50行)

**問題点**:
- 単一責任原則違反
- 最大ファイル (全体の9.3%)
- 再利用性低い

**リファクタ案**:
```
JsonProgramBox (531行)
  ↓
JsonProgramBox (330行)  - 正規化 + メタデータ
+ JsonUtilsBox (150行)  - JSON操作 (新規)
+ StringUtilsBox (統合) - 文字列操作
= 純削減: 50-80行
```

---

## 📋 チェックリスト

### Phase 1-Week1 開始前
- [ ] StringHelpers.hako 拡張計画レビュー
- [ ] クイックウィン対象5ファイル確認
- [ ] スモークテスト実行環境確認
- [ ] バックアップ/git branch作成

### Phase 1-Week1 実施中
- [ ] 5ファイル × 段階的移行 (1ファイルごとにテスト)
- [ ] スモークテスト (各ファイル後)
- [ ] MIR出力ダンプ比較 (回帰確認)
- [ ] 削減行数記録

### Phase 1-Week1 完了判定
- [ ] 全スモークテスト PASS
- [ ] 削減目標達成 (128-143行)
- [ ] Everything is Box準拠度向上確認
- [ ] Phase 1-Week2 着手判断

---

## 🚨 リスクマトリックス

| リスク | 発生確率 | 影響度 | 軽減策 |
|--------|---------|--------|--------|
| 影響範囲広い (30files) | 高 | 高 | 段階的移行 (5files単位) |
| テスト不足 | 中 | 高 | スモークテスト拡充 |
| パフォーマンス劣化 | 低 | 中 | ベンチマーク実施 |
| 回帰バグ | 中 | 高 | MIR出力比較 |

---

## 📚 関連資料

### 詳細分析レポート
- **[横断的重複コード分析](./selfhost-compiler-cross-cutting-analysis.md)** - 完全版
- **[重複コードヒートマップ](./selfhost-compiler-duplication-heatmap.md)** - 視覚的分析

### 既存良好実装 (参考)
- StringHelpers: `/home/tomoaki/git/hakorune-selfhost/apps/selfhost/common/string_helpers.hako`
- MapHelpersBox: `/home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/pipeline_v2/map_helpers_box.hako`
- UsingResolverBox: `/home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/pipeline_v2/using_resolver_box.hako`

### プロジェクトドキュメント
- [開発マスタープラン](../roadmap/phases/00_MASTER_ROADMAP.md)
- [Phase 15 INDEX](../roadmap/phases/phase-15/INDEX.md)
- [Box理論](../../reference/language/LANGUAGE_REFERENCE_2025.md)

---

## 🎯 次のステップ

### 今日のアクション
1. ✅ **このレポートをレビュー** (10分)
2. ✅ **クイックウィン着手判断** (5分)
3. ✅ **StringHelpers.hako 拡張計画** (30分)

### 今週のアクション
1. ✅ **クイックウィン実施** (5ファイル、11-16時間)
2. ✅ **効果検証** (削減行数・テスト結果)
3. ✅ **Phase 1-Week1 着手判断** (金曜日)

### 今月のアクション
1. ✅ **Phase 1完了** (Week 1-2、30-45時間)
2. ✅ **Phase 2着手判断** (月末)
3. ✅ **Everything is Box準拠度75%達成**

---

**生成日時**: 2025-10-12
**分析対象**: apps/selfhost-compiler/ (71ファイル、5,733行)
**推奨優先度**: P0 (StringUtilsBox統合を今日から)
**期待ROI**: ★★★★★ (高削減・低工数・低リスク)

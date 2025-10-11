# Phase 15.12: index_of_from → CfgNavigatorBox統合

## 概要
セルフホストコードベース全体に散在する `index_of_from` 実装を `CfgNavigatorBox` に統合し、さらなる重複削減を行う。

## 背景
Phase 15.11で `_str_to_int`, `read_digits`, `json_quote` 等を `StringHelpers` に統合したが、`index_of_from` の重複が残っている。

## 現状分析

### 既存実装箇所
1. ✅ **統合済み** (2ファイル):
   - `mini_vm_scan.hako` → CfgNavigatorBox委譲
   - `instruction_scanner.hako` → CfgNavigatorBox委譲

2. ❌ **未統合** (5-6ファイル):
   - `JsonFragBox.index_of_from` (超長1行実装)
   - `FlowDebugBox._index_of_from`
   - `flow_runner.hako._index_of_from`
   - `collect_mixed_smoke.hako.index_of_from`
   - `collect_empty_args_smoke.nyash.index_of_from`
   - `collect_literal_eval.nyash.index_of_from`

3. 🚨 **本体の重複**:
   - `CfgNavigatorBox._int_to_str` → StringHelpers委譲すべき

## CfgNavigatorBox.index_of_from 仕様

**Location**: `apps/hakorune/vm/boxes/cfg_navigator.hako:11`

```hako
index_of_from(hay,needle,pos){
  if pos<0 {pos=0}                    // 負数→0正規化
  local n=hay.length()
  if pos>=n {return -1}               // 範囲外→-1
  local m=needle.length()
  if m<=0 {return pos}                // 空needle→pos返す
  local i=pos
  local limit=n-m
  loop(i<=limit){
    if hay.substring(i,i+m)==needle {return i}
    i=i+1
  }
  return -1
}
```

### 仕様特性
| 項目 | 仕様 |
|------|------|
| エスケープ処理 | ❌ なし（プレーンテキスト検索） |
| パフォーマンス | ⚠️ O(n×m) 総当たり |
| 境界条件 | ✅ 負数pos, 空needle, 範囲外pos 全対応 |
| 部分一致 | ✅ substring完全一致 |

## 実装計画

### Step 1: CfgNavigatorBox自身の改善
```hako
// apps/hakorune/vm/boxes/cfg_navigator.hako
using "selfhost/shared/common/string_helpers.hako" as StringHelpers

static box CfgNavigatorBox {
  _int_to_str(n) { return StringHelpers.int_to_str(n) }
  // ... 既存のindex_of_fromはそのまま
}
```

### Step 2: apps/selfhost/ の統合
各ファイルで `index_of_from` → `CfgNavigatorBox.index_of_from` 委譲

### Step 3: テスト
- 既存のスモークテスト全実行
- 特に JSON parsing 系テスト重点確認

## 見込み効果
- **削減行数**: 60-100行
- **影響ファイル**: 7-8ファイル
- **リスク**: 中（本体 apps/hakorune/ も触る）

## リスク要因（ChatGPT分析）

1. **触る範囲が広い**: 5-6 selfhost + 1 hakorune = 7-8ファイル
2. **仕様確定必要**: エスケープ、負荷、境界条件の確認
3. **回帰テスト重要**: JSON parsing は多くの機能に影響
4. **段階導入ポリシー**: 1フェーズに詰め込みすぎない

## 優先度
**20% category** (Phase 15.11で十分な成果 335行削減達成済み)

## 実施タイミング
- Phase 15.11完了後の独立フェーズとして実施
- 十分なテスト時間を確保
- 必要なら複数のサブフェーズに分割

## 関連
- Phase 15.11: StringHelpers統合 (319行削減)
- Phase 15.11.1: json_scan/json_frag統合 (15行削減)
- 合計: 335行削減 → さらに+60-100行見込み

## Created
Phase 15.11完了時 (2025-10-05)

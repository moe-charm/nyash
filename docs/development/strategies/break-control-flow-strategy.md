# Break制御フロー問題 - 根本解決戦略

**問題**: break文がvoid値を返し、メソッドの正常なreturn値を阻害する
**影響**: collect_printsメソッドでnull値が返される問題

## 🎯 ChatGPT Pro最強モード分析結果

### **TL;DR（戦略概要）**

* **短期（フェーズS）**: ブロック終端の厳密検出 + PHI生成時の実到達ブロック捕捉（案A徹底）
* **中期（フェーズM）**: MIRをPHI前提で一本化、no_phi_mode撤廃でコード大幅削減
* **長期（フェーズL）**: build_statementの返り値をBuildOutcomeに根本修正（案B強化版）

## 📊 現在の実装分析

### 1. break/continue処理の問題点
- **現状**: Jump発行後、新規ブロックに切替え、`Const(Void)`を合成して返す
- **問題**: PHI incomingで「入口ブロック」vs「実到達ブロック」のズレ
- **結果**: 早すぎるコピーが走り、古い値が選ばれる

### 2. PHI実装の現状
- **良い点**: if/else、loopは既にPHI前提の設計
- **問題点**: no_phi_mode分岐が残存し、複雑性を増している
- **方向性**: PHI一本化が現実的

## 🚀 段階的実装プラン

### **フェーズS: 即効の止血（1〜2コミット）**

**優先度**: 🔥 最高（Phase 15ブロック解除）

#### 1. PHI incoming predecessor修正
```rust
// 修正前（問題あり）
let then_bb = self.new_block();
self.emit_branch(cond_val, then_bb, else_bb)?;
// ... then処理 ...
self.emit_jump(merge_bb)?;
incomings.push((then_bb, then_val)); // ← 入口ブロック（間違い）

// 修正後（正しい）
let then_bb = self.new_block();
self.emit_branch(cond_val, then_bb, else_bb)?;
// ... then処理 ...
let cur_id = self.current_block()?; // ← 実到達ブロック捕捉
if need_jump {
    self.emit_jump(merge_bb)?;
    incomings.push((cur_id, then_val)); // ← 実到達ブロック（正しい）
}
```

#### 2. 終端ガード徹底
```rust
for statement in statements {
    last_value = Some(self.build_expression(statement)?);
    if self.is_current_block_terminated() {
        break; // ← これを全箇所で徹底
    }
}
```

#### 3. break/continue後のincoming除外
```rust
// break/continue後は到達不能なのでincomingに含めない
if terminated_by_break_or_continue {
    // incoming作成をスキップ
}
```

### **フェーズM: PHI一本化（中期・数週間）**

**優先度**: ⭐⭐⭐ 高（80k→20k圧縮貢献）

#### 1. no_phi_mode分岐撤廃
- `if self.no_phi_mode`分岐を全削除
- edge_copy関連コードを削除
- **期待削減**: 数百行規模

#### 2. Builder API軽ダイエット
- build_blockの終端処理統一
- build_statement呼び出しに寄せる
- 「式と文の下ろし先混在」を減らす

### **フェーズL: 根本解決（後期・Phase 15後半）**

**優先度**: ⭐⭐ 中（設計完成）

#### BuildOutcome導入
```rust
struct BuildOutcome {
    value: Option<ValueId>,
    terminated: bool,
    term: Option<TerminatorKind>, // Return/Jump/Branch等
}

// 段階的移行
impl MirBuilder {
    fn build_statement_new(&mut self, stmt: ASTNode) -> Result<BuildOutcome, String> {
        // 新実装
    }
    
    fn build_statement(&mut self, stmt: ASTNode) -> Result<ValueId, String> {
        // 既存API互換（アダプタ）
        let outcome = self.build_statement_new(stmt)?;
        outcome.value.unwrap_or_else(|| {
            let void_id = self.new_value();
            self.emit_const(void_id, ConstValue::Void).unwrap();
            void_id
        })
    }
}
```

## 📈 効果試算

### コード削減効果
- **no_phi_mode撤廃**: 数百行削除
- **if/loop PHI正規化**: 条件分岐20-30%削減
- **Builder API統一**: 重複処理削除

### Phase 15への貢献
- **80k→20k圧縮**: 大きく貢献（数%単位）
- **安定性向上**: 分岐地獄解消でバグ減少
- **保守性向上**: 設計がクリーンに

## 🛡️ リスク対策

### 短期リスク
- **既存コード互換性**: フェーズSは挙動変更なし
- **テスト回帰**: 最小再現ケースでユニットテスト追加

### 長期リスク
- **API変更波及**: 段階的移行でコンパイル時制御
- **variable_mapスナップショット**: continue順序問題への対策

## 🧪 検証計画

### フェーズS検証
```bash
# 最小再現テスト
echo 'loop(true) { if(cond1) { if(cond2) { x=1 } else { x=2 } break } }' > test.nyash
NYASH_PLUGIN_POLICY=off ./target/release/nyash test.nyash  # compat: NYASH_DISABLE_PLUGINS=1

# collect_prints修正確認
NYASH_PLUGIN_POLICY=off NYASH_RESOLVE_FIX_BRACES=1 ./target/release/nyash apps/selfhost/vm/collect_empty_args_using_smoke.nyash  # compat: NYASH_DISABLE_PLUGINS=1
```

### PHI検証
- predecessor⇔CFG一致チェック
- break/continue後の未定義値検出

## 🎯 次のアクション

1. **即座実行**: フェーズS修正（loop_builder.rs重点）
2. **ユニットテスト**: 最小再現ケース追加
3. **ベンチマーク**: 修正効果の定量評価

## 📚 関連する解決策

### AI各種のアプローチ比較
- **task先生**: 根本原因分析（完璧）
- **Gemini**: 短期案A + 長期案B戦略
- **codex**: 実装重視の型推論強化（高度だがビルド失敗）
- **ChatGPT Pro**: 段階的戦略（最も現実的）

### 推奨採用方針
**フェーズS（ChatGPT Pro戦略）を最優先で実行**
理由: Phase 15セルフホスティング完了への最短経路

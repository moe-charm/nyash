# Selfhost Compiler 超リファクタリング計画 (Master Plan)

**作成日**: 2025-10-04
**対象**: apps/selfhost-compiler/
**目標**: 箱化率100%・モジュール性向上・保守性強化

---

## 📊 現状分析サマリー

### 統計データ
- **総ファイル数**: 38ファイル (.hako + .nyash)
  - .hako: 24ファイル (4,445行)
  - .nyash: 14ファイル (943行)
- **重複ファイル**: 14組 (.nyash/.hako両方存在)
- **箱化率**: 約82% (14/24 .hakoファイルが箱化済み)
- **最大ファイル**:
  - parser_box.hako (921行) - 大きすぎる
  - json_program_box.hako (520行)
  - local.nyash (547行) - .hako版は130行

### 問題点の整理

#### 🔴 P0: 破壊的問題（即座に対処必要）
1. **重複ファイル問題**: 14組の.nyash/.hakoが並存
   - 影響: ビルド混乱・メンテナンス二重負担
   - リスク: どちらが正しいか不明・同期ズレの可能性

2. **巨大ファイル問題**: parser_box.hako (921行)
   - 影響: 責務不明確・テスト困難・変更影響範囲大
   - リスク: バグ混入リスク高・並行開発困難

#### 🟡 P1: 品質問題（早期対処推奨）
1. **箱化未完了**: 14ファイルが.nyashのまま残存
   - 影響: Everything is Box哲学からの乖離
   - リスク: 型安全性低下・将来的な技術負債

2. **モジュール構造不明確**:
   - parser/builder/emitter/mir の責務境界が曖昧
   - pipeline_v2/ 配下が17箱に分散

#### 🔵 P2: 改善（計画的対処）
1. **インターフェース不統一**:
   - 箱間の依存関係が暗黙的
   - INTERFACES.mdとの整合性確認必要

2. **テストカバレッジ不足**:
   - 個別箱の単体テストが不明確
   - smokes/テストが統合レベルのみ

---

## 🎯 リファクタリング戦略

### 基本方針
1. **段階的移行**: 一度に全部やらず、Phase毎に動作確認
2. **Fail-Fast原則**: 各Phase完了時に必ずテスト実行
3. **後方互換性**: 既存の動作を壊さない（フラグで切替可能に）
4. **箱理論4原則**:
   - 「箱にする」: すべて.hakoに統一
   - 「境界を作る」: モジュール間の責務を明確化
   - 「戻せる」: 各Phaseで安全にロールバック可能
   - 「見える化」: INTERFACES.md完全同期

---

## 📅 Phase別実行計画

### Phase 0: 事前準備（1-2時間）

#### 目的
- 現状の完全把握
- ベースライン確立
- バックアップ確保

#### タスク
1. **全スモークテスト実行** (30分)
   ```bash
   # 既存テストの動作確認
   tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"
   ```
   - 成功基準: 全テストPASS
   - 記録: /tmp/baseline_test_results.txt

2. **依存関係マップ作成** (30分)
   ```bash
   # 各.hakoファイルの"using"/"new"を抽出
   find apps/selfhost-compiler -name "*.hako" -exec grep -H "using\|new " {} \;
   ```
   - 成果物: /tmp/dependency_map.md
   - 内容: 箱間の依存グラフ

3. **重複ファイル差分確認** (30分)
   - 14組の.nyash/.hakoの差分を比較
   - 成果物: /tmp/duplicate_analysis.txt
   - 判定: どちらを残すか決定

4. **ブランチ作成** (5分)
   ```bash
   git checkout -b refactor/selfhost-super-cleanup
   git push -u origin refactor/selfhost-super-cleanup
   ```

---

### Phase 1: 緊急修正（3-4時間）

#### 目的
- 重複ファイル解消
- ビルド混乱排除
- 安定基盤確立

#### タスク

##### 1.1 重複ファイル統一 (2時間)
**優先順位**: P0

**戦略**: .nyash → .hako へ段階的移行

**手順**:
```bash
# 各ファイルペアごとに:
# 1. 差分確認
diff -u file.nyash file.hako

# 2. .hakoに統合（.nyashの最新内容を反映）
# 3. .nyashを削除
git rm file.nyash

# 4. テスト実行（個別確認）
tools/smokes/v2/run.sh --profile quick --filter "関連テスト"
```

**対象ファイル** (優先順):
1. **interfaces.nyash** → interfaces.hako (最優先)
2. **parser/lexer.nyash** → parser/lexer.hako
3. **parser/parser.nyash** → parser/parser.hako
4. **parser/ast.nyash** → parser/ast.hako
5. **emitter/json_v0.nyash** → emitter/json_v0.hako
6. **boxes/debug_box.nyash** → boxes/debug_box.hako
7. **boxes/mir_emitter_box.nyash** → boxes/mir_emitter_box.hako
8. **builder/ssa/loopssa.nyash** → builder/ssa/loopssa.hako
9. **builder/ssa/local.nyash** → builder/ssa/local.hako (547行→130行に精査)
10. **builder/rewrite/special.nyash** → builder/rewrite/special.hako
11. **builder/rewrite/known.nyash** → builder/rewrite/known.hako
12. **builder/mod.nyash** → builder/mod.hako
13. **mir/builder.nyash** → mir/builder.hako
14. **mir/optimizer.nyash** → mir/optimizer.hako

**成功基準**:
- ✅ .nyashファイル 0個
- ✅ 全スモークテスト PASS
- ✅ INTERFACES.md更新完了

**リスク**:
- ⚠️ .nyash/.hakoの差分が大きい場合、統合ミスの可能性
- **軽減策**: 各ファイル統合後に個別テスト実行

##### 1.2 巨大ファイル分割計画策定 (1時間)
**対象**: parser_box.hako (921行)

**分析ポイント**:
- 責務ごとの行数分布
- メソッド単位の凝集度
- 分割候補の抽出

**成果物**: /tmp/parser_box_split_plan.md
- 分割後の箱候補リスト
- 各箱の責務定義
- インターフェース設計

**実装**: Phase 2に送る

##### 1.3 テスト＆コミット (1時間)
```bash
# 全スモークテスト実行
tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"

# 成功したらコミット
git add -A
git commit -m "refactor(selfhost): Phase 1完了 - 重複ファイル統一化 (14組 .nyash削除)

- ✅ .nyash→.hako統一: 14ファイル
- ✅ 箱化率: 82%→100%
- ✅ ビルド混乱解消
- ✅ 全スモークテストPASS

Phase 1 完了。Phase 2（箱化推進）へ。
"
git push
```

---

### Phase 2: 箱化推進（5-6時間）

#### 目的
- Everything is Box完全実現
- 巨大ファイル分割
- モジュール責務明確化

#### タスク

##### 2.1 parser_box.hako 分割 (3時間)
**現状**: 921行の巨大箱

**分割戦略**:
```
parser_box.hako (921行)
  ↓ 分割
├─ lexer_box.hako (~300行)      // 字句解析専用
├─ parser_core_box.hako (~400行) // 構文解析コア
└─ ast_builder_box.hako (~250行) // AST構築
```

**手順**:
1. **責務分析** (30分)
   - メソッド単位で責務を分類
   - 依存関係を可視化

2. **インターフェース設計** (30分)
   - 各箱のpublicメソッド定義
   - INTERFACES.md更新

3. **分割実装** (90分)
   - lexer_box.hako作成
   - parser_core_box.hako作成
   - ast_builder_box.hako作成
   - 元のparser_box.hakoは削除

4. **テスト** (30分)
   ```bash
   tools/smokes/v2/run.sh --profile quick --filter "selfhost_*parser*"
   ```

**成功基準**:
- ✅ 各箱が300行以下
- ✅ 責務が明確（1箱1責務）
- ✅ パーサーテスト全PASS

##### 2.2 json_program_box.hako 精査 (1時間)
**現状**: 520行

**戦略**:
- ヘルパーメソッドをutilsに分離
- JSONシリアライズ専用箱を作成

**分割候補**:
```
json_program_box.hako (520行)
  ↓ 精査
├─ json_program_box.hako (~300行)  // コアロジックのみ
└─ json_utils_box.hako (~220行)    // ヘルパー関数
```

##### 2.3 local.hako 精査 (1時間)
**現状**: local.nyash (547行) vs local.hako (130行)

**問題**: .nyashが4倍の行数

**手順**:
1. 差分分析 (20分)
2. .nyashの追加機能を.hakoに統合 (30分)
3. テスト (10分)

##### 2.4 pipeline_v2/ 箱構造整理 (1時間)
**現状**: 17箱が並列

**整理戦略**:
```
pipeline_v2/
├─ core/                    // コア箱（実行パイプライン）
│  ├─ execution_pipeline_box.hako
│  ├─ backend_box.hako
│  └─ mir_builder_box.hako
├─ extractors/              // 抽出系
│  ├─ compare_extract_box.hako
│  ├─ call_extract_box.hako
│  ├─ method_extract_box.hako
│  └─ new_extract_box.hako
├─ emitters/                // 発行系
│  ├─ emit_binop_box.hako
│  ├─ emit_call_box.hako
│  ├─ emit_compare_box.hako
│  ├─ emit_method_box.hako
│  ├─ emit_newbox_box.hako
│  └─ emit_return_box.hako
├─ flows/                   // フロー系
│  ├─ flow_entry.hako
│  ├─ emit_mir_flow.hako
│  ├─ emit_mir_flow_map.hako
│  ├─ regex_flow.hako
│  └─ stage1_extract_flow.hako
├─ utils/                   // ユーティリティ
│  ├─ map_helpers_box.hako
│  ├─ normalizer_box.hako
│  ├─ readonly_map_view.hako
│  └─ mir_call_box.hako
└─ ssa/                     // SSA関連
   └─ local_ssa_box.hako
```

**手順**:
1. ディレクトリ作成
2. ファイル移動
3. using文更新
4. テスト実行

##### 2.5 テスト＆コミット (30分)
```bash
# 全スモークテスト
tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"

git add -A
git commit -m "refactor(selfhost): Phase 2完了 - 箱化推進＆モジュール整理

- ✅ parser_box.hako分割: 921行→3箱(~300行/箱)
- ✅ json_program_box精査: 520行→2箱
- ✅ pipeline_v2/構造化: 5ディレクトリ編成
- ✅ 箱化率: 100%達成
- ✅ 全スモークテストPASS

Phase 2 完了。Phase 3（インターフェース統一）へ。
"
git push
```

---

### Phase 3: インターフェース統一（3-4時間）

#### 目的
- INTERFACES.md完全同期
- 箱間依存を明示的に
- Fail-Fast原則の徹底

#### タスク

##### 3.1 INTERFACES.md完全更新 (2時間)
**現状**: 最小限の記述のみ

**追加内容**:
1. **全箱のインターフェース定義**
   - birth()シグネチャ
   - publicメソッド一覧
   - 戻り値の型（i64/String/ArrayBox等）
   - エラーケース（Fail-Fast箇所）

2. **依存関係マトリックス**
   ```
   | 箱名 | 依存先 | 責務 |
   |------|--------|------|
   | ExecutionPipelineBox | ParserBox, EmitterBox | オーケストレーション |
   | ParserBox | LexerBox, AstBuilderBox | 構文解析 |
   ...
   ```

3. **契約（Contracts）強化**
   - 入力検証ルール
   - 出力形式保証
   - エラーハンドリング方針

**成果物**: INTERFACES.md (v2.0)

##### 3.2 箱間インターフェース検証 (1時間)
**方法**:
- 各箱のusing文とINTERFACES.mdを突合
- 未定義の依存を発見
- 暗黙的な結合を明示化

**ツール作成**: /tmp/verify_interfaces.sh
```bash
#!/bin/bash
# INTERFACES.mdに記載された箱と実際のusing文を比較
# 不一致があれば警告
```

##### 3.3 エラーハンドリング統一 (1時間)
**現状**: エラー処理が不統一

**統一方針**:
```hakorune
// ✅ 統一形式
birth(arg: String) {
    if arg == "" {
        print("[ERROR] ParserBox: empty argument")
        return -1  // Fail-Fast
    }
    // ... 正常処理
    return 0
}
```

**適用箇所**:
- すべての箱のbirthメソッド
- publicメソッドの入力検証
- 中間状態の不正検出

##### 3.4 テスト＆コミット (30分)
```bash
tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"

git add -A
git commit -m "refactor(selfhost): Phase 3完了 - インターフェース統一

- ✅ INTERFACES.md v2.0: 全箱定義完了
- ✅ 依存関係マトリックス追加
- ✅ エラーハンドリング統一（Fail-Fast）
- ✅ 検証スクリプト作成
- ✅ 全スモークテストPASS

Phase 3 完了。Phase 4（最適化）へ。
"
git push
```

---

### Phase 4: 最適化・クリーンアップ（4-5時間）

#### 目的
- デッドコード削除
- パフォーマンス改善
- ドキュメント完備

#### タスク

##### 4.1 デッドコード削除 (2時間)
**方法**:
1. **未使用メソッド検出**
   ```bash
   # すべてのメソッド定義を抽出
   grep -r "^\s*[a-z_]*(" apps/selfhost-compiler/*.hako

   # 呼び出し箇所を検索
   # 呼び出しがないメソッドをリスト化
   ```

2. **未使用箱検出**
   - usingされていない箱を探す
   - テスト専用箱かどうか確認

3. **削除実施**
   - 慎重に削除
   - テスト実行で確認

**成功基準**:
- ✅ 未使用コード 0行
- ✅ 全テストPASS

##### 4.2 パフォーマンス改善 (1時間)
**対象**:
- 巨大MapBoxの最適化
- 不要なコピー削減
- ループ内のnew削減

**手法**:
- プロファイリング（可能なら）
- コードレビュー
- ベンチマーク比較

##### 4.3 ドキュメント完備 (1時間)
**追加ドキュメント**:
1. **apps/selfhost-compiler/README.md更新**
   - アーキテクチャ図
   - 箱の責務一覧
   - 開発ガイド

2. **各箱にコメント追加**
   ```hakorune
   // ParserBox: ソーステキストをStage-1 JSON (AST)に変換
   // 依存: LexerBox, AstBuilderBox
   // 責務: 字句解析→構文解析→AST生成
   static box ParserBox {
       // ...
   }
   ```

3. **pipeline_v2/README.md作成**
   - パイプライン構造図
   - データフロー
   - 拡張方法

##### 4.4 最終テスト＆コミット (30分)
```bash
# すべてのスモークテスト
tools/smokes/v2/run.sh --profile quick
tools/smokes/v2/run.sh --profile integration

git add -A
git commit -m "refactor(selfhost): Phase 4完了 - 最適化＆クリーンアップ

- ✅ デッドコード削除: XX行削減
- ✅ パフォーマンス改善: YY%向上
- ✅ ドキュメント完備: README×3更新
- ✅ コメント追加: 全箱
- ✅ 全スモークテストPASS

Phase 4 完了。超リファクタリング計画完遂！
"
git push
```

---

## 📊 成果指標（KPI）

### 定量指標
| 指標 | 現状 | 目標 | Phase 4完了後 |
|-----|------|------|--------------|
| 箱化率 | 82% | 100% | 100% ✅ |
| .nyashファイル数 | 14 | 0 | 0 ✅ |
| 重複ファイル | 14組 | 0組 | 0組 ✅ |
| 最大ファイル行数 | 921 | <300 | <300 ✅ |
| 未使用コード | ? | 0 | 0 ✅ |
| INTERFACES.md網羅率 | 30% | 100% | 100% ✅ |

### 定性指標
- ✅ モジュール結合度: Low（箱間依存が明示的）
- ✅ コードの凝集度: High（1箱1責務）
- ✅ 保守性: 高（ドキュメント完備）
- ✅ テストカバレッジ: 十分（全スモークテストPASS）

---

## ⏱️ 推定工数

| Phase | 内容 | 推定時間 | 累積時間 |
|-------|------|----------|----------|
| Phase 0 | 事前準備 | 1-2時間 | 2時間 |
| Phase 1 | 緊急修正 | 3-4時間 | 6時間 |
| Phase 2 | 箱化推進 | 5-6時間 | 12時間 |
| Phase 3 | インターフェース統一 | 3-4時間 | 16時間 |
| Phase 4 | 最適化・クリーンアップ | 4-5時間 | 21時間 |
| **合計** | | **16-21時間** | **21時間** |

**推奨実行スケジュール**:
- **Day 1**: Phase 0 + Phase 1（4-6時間）
- **Day 2**: Phase 2（5-6時間）
- **Day 3**: Phase 3（3-4時間）
- **Day 4**: Phase 4（4-5時間）
- **総期間**: 3-4日（1日5-6時間作業想定）

---

## 🚨 リスク評価と軽減策

### リスク1: 既存機能の破壊
**確率**: 中
**影響**: 高
**軽減策**:
- ✅ 各Phase完了時に全スモークテスト実行
- ✅ 段階的コミット（Phase毎）
- ✅ ブランチ作成でmainを守る
- ✅ CI/CDがあればフル活用

### リスク2: 重複ファイル統合ミス
**確率**: 中
**影響**: 中
**軽減策**:
- ✅ 差分確認を徹底（diff -u）
- ✅ 各ファイル統合後に個別テスト
- ✅ .nyash削除前にバックアップ（git管理下なので安全）

### リスク3: 巨大ファイル分割の複雑化
**確率**: 中
**影響**: 中
**軽減策**:
- ✅ 責務分析を慎重に実施
- ✅ インターフェース設計を先に完成
- ✅ 小さく分割（3箱程度）
- ✅ 各分割後にテスト実行

### リスク4: Phase 2以降の工数超過
**確率**: 高
**影響**: 低
**軽減策**:
- ✅ Phase毎に動作確認→完了判定
- ✅ 優先度低いタスクはスキップ可
- ✅ 80/20ルール適用（完璧より進捗）

### リスク5: ドキュメント更新漏れ
**確率**: 中
**影響**: 中
**軽減策**:
- ✅ INTERFACES.md更新をチェックリスト化
- ✅ コミットメッセージに更新箇所明記
- ✅ Phase 3でドキュメント集中対応

---

## 🔧 ツール・スクリプト

### 作成予定ツール
1. **/tmp/verify_interfaces.sh**
   - INTERFACES.mdと実装の整合性検証
   - 未定義依存の検出

2. **/tmp/find_dead_code.sh**
   - 未使用メソッド検出
   - 未使用箱検出

3. **/tmp/check_boxification.sh**
   - .nyashファイル残存チェック
   - 箱化率計算

---

## 📝 チェックリスト（実行時に使用）

### Phase 0
- [ ] 全スモークテスト実行（ベースライン）
- [ ] 依存関係マップ作成
- [ ] 重複ファイル差分確認
- [ ] ブランチ作成

### Phase 1
- [ ] 14組の重複ファイル統一
  - [ ] interfaces.nyash → interfaces.hako
  - [ ] parser/lexer.nyash → parser/lexer.hako
  - [ ] parser/parser.nyash → parser/parser.hako
  - [ ] parser/ast.nyash → parser/ast.hako
  - [ ] emitter/json_v0.nyash → emitter/json_v0.hako
  - [ ] boxes/debug_box.nyash → boxes/debug_box.hako
  - [ ] boxes/mir_emitter_box.nyash → boxes/mir_emitter_box.hako
  - [ ] builder/ssa/loopssa.nyash → builder/ssa/loopssa.hako
  - [ ] builder/ssa/local.nyash → builder/ssa/local.hako
  - [ ] builder/rewrite/special.nyash → builder/rewrite/special.hako
  - [ ] builder/rewrite/known.nyash → builder/rewrite/known.hako
  - [ ] builder/mod.nyash → builder/mod.hako
  - [ ] mir/builder.nyash → mir/builder.hako
  - [ ] mir/optimizer.nyash → mir/optimizer.hako
- [ ] 巨大ファイル分割計画策定
- [ ] 全スモークテスト（Phase 1完了確認）
- [ ] Phase 1コミット

### Phase 2
- [ ] parser_box.hako分割（921行→3箱）
- [ ] json_program_box.hako精査（520行→2箱）
- [ ] local.hako統合（547行→統一版）
- [ ] pipeline_v2/構造整理（5ディレクトリ）
- [ ] 全スモークテスト（Phase 2完了確認）
- [ ] Phase 2コミット

### Phase 3
- [ ] INTERFACES.md v2.0作成
- [ ] 依存関係マトリックス追加
- [ ] 箱間インターフェース検証
- [ ] エラーハンドリング統一
- [ ] 全スモークテスト（Phase 3完了確認）
- [ ] Phase 3コミット

### Phase 4
- [ ] デッドコード削除
- [ ] パフォーマンス改善
- [ ] ドキュメント完備
- [ ] 全スモークテスト（Phase 4完了確認）
- [ ] Phase 4コミット
- [ ] **超リファクタリング計画完遂！🎉**

---

## 🎯 成功の定義

### 必須条件（Must Have）
- ✅ .nyashファイル 0個（箱化率100%）
- ✅ 重複ファイル 0組
- ✅ 全スモークテストPASS
- ✅ INTERFACES.md完全同期

### 推奨条件（Should Have）
- ✅ 最大ファイル行数 <300行
- ✅ pipeline_v2/構造化（5ディレクトリ）
- ✅ ドキュメント完備

### 理想条件（Nice to Have）
- ✅ デッドコード 0行
- ✅ パフォーマンス改善 10%以上
- ✅ 単体テスト追加

---

## 📖 参考資料

### 重要ドキュメント
- [CLAUDE.md](file:///home/tomoaki/git/hakorune-selfhost/CLAUDE.md) - 箱理論・開発方針
- [INTERFACES.md](file:///home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/INTERFACES.md) - 現行インターフェース定義
- [pipeline_v2.md](file:///home/tomoaki/git/hakorune-selfhost/docs/development/selfhosting/pipeline_v2.md) - パイプライン設計

### 開発原則
- **箱理論4原則**: 箱にする・境界を作る・戻せる・見える化
- **Fail-Fast原則**: フォールバック禁止・早期エラー
- **80/20ルール**: 完璧より進捗

---

## 🔄 ガントチャート（テキスト版）

```
Week 1 (Day 1-4):
Day 1: [==Phase 0==][========Phase 1========]
Day 2: [====================Phase 2====================]
Day 3: [==============Phase 3==============]
Day 4: [================Phase 4================]

詳細タイムライン:
Day 1 (6h):
  00:00-02:00  Phase 0: 事前準備
  02:00-06:00  Phase 1: 緊急修正

Day 2 (6h):
  00:00-06:00  Phase 2: 箱化推進

Day 3 (4h):
  00:00-04:00  Phase 3: インターフェース統一

Day 4 (5h):
  00:00-05:00  Phase 4: 最適化・クリーンアップ

合計: 21時間（3-4日間）
```

---

## 🎊 完了後の展望

### 即座に得られる効果
1. **開発速度向上**: 責務明確化→変更箇所が即座に特定
2. **バグ減少**: Fail-Fast原則→エラーの早期発見
3. **新規参入容易化**: ドキュメント完備→学習コスト低減

### 中長期的効果
1. **Phase 15.7加速**: クリーンな基盤でMIR生成実装が容易
2. **セルフホスティング実現**: 箱化100%でHakorune自体でコンパイル可能
3. **WASMバックエンド対応**: モジュール化により新バックエンド追加が容易

### 技術的資産
- **再利用可能な箱群**: 他プロジェクトでも利用可能
- **設計パターン確立**: 箱理論の実践例として参照可能
- **テストインフラ**: スモークテストが充実

---

## 📞 サポート・相談

### 迷ったら
- CLAUDE.mdの箱理論4原則を再確認
- INTERFACES.mdの契約を確認
- スモークテストを実行（動作確認）

### エラー時
- Fail-Fast原則に従い、エラーメッセージを精査
- git logで前回動作時のコミットを確認
- 必要ならgit revertでロールバック

### 改善提案
- アイデアは docs/development/proposals/ideas/ に記録
- 80/20ルール適用（すべてを今やらない）

---

**🎯 この計画は段階的・実行可能・後戻り可能に設計されています。**
**各Phaseを確実に完了させ、箱理論4原則を守りながら進めましょう！**

**🚀 Let's refactor with Box-First philosophy!**

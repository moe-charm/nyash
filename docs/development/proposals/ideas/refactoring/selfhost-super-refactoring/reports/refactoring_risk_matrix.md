# 超リファクタリング計画 - リスク分析マトリックス

## 🎯 リスク評価基準

### 影響度（Impact）
- **高（High）**: プロジェクト全体に影響、復旧に1日以上
- **中（Medium）**: 一部機能に影響、復旧に数時間
- **低（Low）**: 限定的影響、すぐ復旧可能

### 発生確率（Probability）
- **高（High）**: 70-100%の確率で発生
- **中（Medium）**: 30-70%の確率で発生
- **低（Low）**: 0-30%の確率で発生

### リスクレベル（Risk Level）
- **Critical**: 影響度:高 × 確率:高 → 即座対応必須
- **High**: 影響度:高 × 確率:中 OR 影響度:中 × 確率:高
- **Medium**: 影響度:中 × 確率:中
- **Low**: その他

---

## 📊 リスク一覧表

| ID | リスク | Phase | 影響度 | 確率 | レベル | 対策優先度 |
|----|-------|-------|--------|------|--------|----------|
| R01 | 既存機能の破壊 | All | 高 | 中 | **High** | P0 |
| R02 | 重複ファイル統合ミス | 1 | 中 | 中 | **Medium** | P1 |
| R03 | 巨大ファイル分割の複雑化 | 2 | 中 | 中 | **Medium** | P1 |
| R04 | Phase 2以降の工数超過 | 2-4 | 低 | 高 | **Medium** | P2 |
| R05 | ドキュメント更新漏れ | 3 | 中 | 中 | **Medium** | P1 |
| R06 | インターフェース定義の不整合 | 3 | 中 | 低 | **Low** | P2 |
| R07 | デッドコード判定ミス | 4 | 低 | 中 | **Low** | P3 |
| R08 | テスト環境の不安定 | All | 高 | 低 | **Medium** | P1 |
| R09 | Gitブランチの衝突 | All | 中 | 低 | **Low** | P2 |
| R10 | パフォーマンス劣化 | 4 | 中 | 低 | **Low** | P2 |

---

## 🔴 Critical / High リスク詳細分析

### R01: 既存機能の破壊 ⚠️ **High Risk**
**影響度**: 高 | **確率**: 中

#### 発生シナリオ
1. 重複ファイル統合時に古い版を選択
2. parser_box分割時に依存関係を壊す
3. using文更新漏れ
4. インターフェース変更の影響波及

#### 影響範囲
- 全スモークテストが失敗
- セルフホスティングが動作しなくなる
- Phase 15.7進捗が停止

#### 軽減策（5層防御）
1. **Phase毎の全スモークテスト実行**
   ```bash
   tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"
   ```
   - 成功基準: 全テストPASS
   - 失敗時: 即座にロールバック

2. **段階的コミット**
   - Phase毎にコミット
   - 問題発生時はgit revert可能

3. **ブランチ作成**
   ```bash
   git checkout -b refactor/selfhost-super-cleanup
   ```
   - mainブランチを守る
   - 実験的変更も安全

4. **差分確認の徹底**
   ```bash
   diff -u file.nyash file.hako
   ```
   - 統合前に必ず確認
   - 不明な差分は調査

5. **個別テストの実施**
   - ファイル統合毎にテスト
   - 早期発見・早期修正

#### 復旧手順（もし発生したら）
```bash
# 1. 失敗したPhaseを特定
git log --oneline

# 2. 直前のコミットにリバート
git revert HEAD

# 3. スモークテスト再実行
tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"

# 4. 原因分析
git diff HEAD~1 HEAD

# 5. 慎重に再実装
```

#### 成功確率向上策
- ✅ 各Phase開始前にベースライン確立
- ✅ テスト実行時間を惜しまない（安全第一）
- ✅ 不明点は保守的に判断（疑わしいものは残す）

---

## 🟡 Medium リスク詳細分析

### R02: 重複ファイル統合ミス ⚠️ **Medium Risk**
**影響度**: 中 | **確率**: 中

#### 発生シナリオ
- .nyash/.hakoの差分が大きい
- どちらが正しいか判断できない
- 統合後に機能が失われる

#### 軽減策
1. **差分分析の徹底**
   ```bash
   diff -u file.nyash file.hako > /tmp/diff_analysis.txt
   # 行数を確認
   wc -l /tmp/diff_analysis.txt
   ```

2. **3段階判定法**
   ```
   差分が小さい（<10行）:
     → 新しい方を採用

   差分が中程度（10-50行）:
     → 手動でマージ（重要部分のみ）

   差分が大きい（>50行）:
     → 個別に機能確認→慎重にマージ
   ```

3. **個別テスト実施**
   - 統合後、該当箱のテストを実行
   - 動作確認後に次へ進む

#### チェックリスト
- [ ] diff -u で差分確認
- [ ] 差分が10行以下 → そのまま統合
- [ ] 差分が10行以上 → 手動マージ検討
- [ ] 統合後にテスト実行
- [ ] .nyash削除前に最終確認

---

### R03: 巨大ファイル分割の複雑化 ⚠️ **Medium Risk**
**影響度**: 中 | **確率**: 中

#### 発生シナリオ
- parser_box.hako (921行) の責務分析が不十分
- 分割境界が曖昧
- 循環依存が発生

#### 軽減策
1. **責務分析（30分投資）**
   ```bash
   # メソッド一覧抽出
   grep "^\s*[a-z_]*(" parser_box.hako

   # 各メソッドの責務分類
   # - 字句解析: lexer_box.hako
   # - 構文解析: parser_core_box.hako
   # - AST構築: ast_builder_box.hako
   ```

2. **インターフェース先行設計**
   - 分割後の箱のpublicメソッドを先に定義
   - INTERFACES.mdに仮記述
   - 実装前にレビュー

3. **段階的実装**
   ```
   Step 1: lexer_box.hako作成（最も独立）
   Step 2: ast_builder_box.hako作成（次に独立）
   Step 3: parser_core_box.hako作成（残り）
   Step 4: 元のparser_box.hako削除
   ```

4. **依存関係の可視化**
   ```bash
   # 各箱のusing文を確認
   grep "using\|new " lexer_box.hako
   grep "using\|new " parser_core_box.hako
   grep "using\|new " ast_builder_box.hako
   ```

#### 成功基準
- ✅ 各箱が300行以下
- ✅ 循環依存なし
- ✅ パーサーテスト全PASS

---

### R04: Phase 2以降の工数超過 ⚠️ **Medium Risk**
**影響度**: 低 | **確率**: 高

#### 発生シナリオ
- 想定外の複雑さ発見
- テスト失敗→原因調査に時間
- ドキュメント作成に予想以上の時間

#### 軽減策
1. **80/20ルール適用**
   - 優先度低いタスクはスキップ
   - 必須条件（Must Have）に集中
   - 理想条件（Nice to Have）は後回し

2. **タイムボックス設定**
   ```
   Phase 2.1: parser_box分割 → 3時間厳守
     3時間経過時点で80%完了→そのまま進む
     50%未満→アプローチ変更

   Phase 4.2: パフォーマンス改善 → 1時間厳守
     目標未達でも次へ（後回し可）
   ```

3. **バッファ時間の活用**
   - Phase 1-2: 各1時間バッファ
   - Phase 3-4: 各30分バッファ
   - 合計3時間の余裕

#### 工数超過時の対応
```
超過1時間以内:
  → バッファ時間で吸収

超過1-2時間:
  → Nice to Haveタスクをスキップ

超過2時間以上:
  → Phase分割（一部を次回に延期）
```

---

### R05: ドキュメント更新漏れ ⚠️ **Medium Risk**
**影響度**: 中 | **確率**: 中

#### 発生シナリオ
- INTERFACES.md更新忘れ
- README更新忘れ
- コメント追加忘れ

#### 軽減策
1. **Phase 3でドキュメント集中対応**
   - Phase 1-2は実装に集中
   - Phase 3で一気にドキュメント更新

2. **チェックリスト活用**
   ```
   Phase 3完了チェックリスト:
   □ INTERFACES.md v2.0完成
   □ 全箱のインターフェース定義済み
   □ 依存関係マトリックス作成
   □ apps/selfhost-compiler/README.md更新
   □ pipeline_v2/README.md作成
   □ 各箱にコメント追加
   ```

3. **verify_interfaces.sh作成**
   ```bash
   #!/bin/bash
   # INTERFACES.mdと実装の突合
   # 未定義の箱を検出
   ```

---

### R08: テスト環境の不安定 ⚠️ **Medium Risk**
**影響度**: 高 | **確率**: 低

#### 発生シナリオ
- スモークテストが環境依存で失敗
- テストデータの破損
- ビルド環境の問題

#### 軽減策
1. **Phase 0でベースライン確立**
   ```bash
   # ベースラインテスト（Phase 0）
   tools/smokes/v2/run.sh --profile quick --filter "selfhost_*" \
     | tee /tmp/baseline_test_results.txt
   ```
   - すべてPASSを確認
   - 失敗があれば先に修正

2. **テスト環境の固定**
   ```bash
   # ビルド確認
   cargo build --release

   # バージョン確認
   ./target/release/hakorune --version
   ```

3. **テスト失敗時の対応**
   ```
   失敗が1-2個:
     → 該当テストのみ調査

   失敗が3個以上:
     → 環境問題の可能性（ベースライン比較）
   ```

---

## 🟢 Low リスク（監視のみ）

### R06: インターフェース定義の不整合
**軽減策**: verify_interfaces.sh で検出

### R07: デッドコード判定ミス
**軽減策**: 保守的に判定（疑わしいものは残す）

### R09: Gitブランチの衝突
**軽減策**: 専用ブランチ作成・他ブランチと隔離

### R10: パフォーマンス劣化
**軽減策**: ベンチマーク比較・劣化時は最適化延期

---

## 📋 リスク監視チェックリスト

### Phase 0完了時
- [ ] ベースラインテスト全PASS（R08対策）
- [ ] ブランチ作成完了（R01,R09対策）
- [ ] 依存関係マップ作成（R03対策）

### Phase 1完了時
- [ ] .nyashファイル 0個（R02確認）
- [ ] 全スモークテストPASS（R01確認）
- [ ] parser_box分割計画完成（R03準備）

### Phase 2完了時
- [ ] parser_box分割成功（R03確認）
- [ ] 全ファイル<300行（品質確認）
- [ ] 全スモークテストPASS（R01確認）

### Phase 3完了時
- [ ] INTERFACES.md v2.0完成（R05対策）
- [ ] verify_interfaces.sh動作（R06対策）
- [ ] 全スモークテストPASS（R01確認）

### Phase 4完了時
- [ ] デッドコード削除慎重実施（R07対策）
- [ ] ドキュメント完備（R05確認）
- [ ] 全スモークテストPASS（R01最終確認）

---

## 🚨 緊急時対応フロー

### テスト失敗時（R01発生）
```bash
# 1. 失敗内容確認
tools/smokes/v2/run.sh --profile quick --filter "selfhost_*" 2>&1 | tee /tmp/test_failure.log

# 2. ベースラインと比較
diff /tmp/baseline_test_results.txt /tmp/test_failure.log

# 3. 直前のコミットにリバート
git revert HEAD

# 4. 再テスト
tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"

# 5. 成功したら原因分析・再実装
git diff HEAD~1 HEAD
```

### 工数超過時（R04発生）
```bash
# 1. 現在時刻確認
date

# 2. 進捗確認（Phase完了率）
# Phase 2想定6時間、現在4時間経過、進捗50%
# → 残り2時間で50%は困難

# 3. Nice to Haveタスクをスキップ
# Phase 2.4 pipeline_v2/構造整理 → Phase 5に延期

# 4. 必須タスクに集中
# Phase 2.1-2.3 のみ完了させる
```

### ドキュメント更新漏れ時（R05発生）
```bash
# 1. verify_interfaces.sh実行
bash /tmp/verify_interfaces.sh

# 2. 未定義箇所をリスト化
# 出力例: "ParserCoreBox: undefined in INTERFACES.md"

# 3. 集中更新（30分タイムボックス）
# INTERFACES.mdに追加記述

# 4. 再検証
bash /tmp/verify_interfaces.sh
```

---

## 📊 リスクダッシュボード（監視用）

### 実行中のリスクレベル可視化
```
Phase 0実行中:
[R01] 既存機能破壊:     🟡 Medium (監視中)
[R08] テスト環境不安定: 🟡 Medium (ベースライン確立中)

Phase 1実行中:
[R01] 既存機能破壊:     🟡 Medium (テスト実施中)
[R02] 重複統合ミス:     🟡 Medium (差分確認中)

Phase 2実行中:
[R01] 既存機能破壊:     🟡 Medium (テスト実施中)
[R03] 分割複雑化:       🟡 Medium (責務分析実施中)
[R04] 工数超過:         🟡 Medium (タイムボックス監視中)

Phase 3実行中:
[R01] 既存機能破壊:     🟡 Medium (テスト実施中)
[R05] ドキュメント漏れ: 🟡 Medium (集中対応中)

Phase 4実行中:
[R01] 既存機能破壊:     🟡 Medium (最終テスト中)
[R07] デッドコード判定: 🟢 Low (保守的判定)
[R10] パフォーマンス劣化:🟢 Low (ベンチマーク比較中)

全Phase完了:
すべてのリスク: 🟢 Low (クリア！)
```

---

## 🎯 リスク管理の成功基準

### Phase毎の成功基準
- Phase 0: ベースライン確立 → R08解消
- Phase 1: 重複統一完了 → R02解消
- Phase 2: 分割成功 → R03,R04解消
- Phase 3: ドキュメント完備 → R05,R06解消
- Phase 4: 最終確認完了 → R01,R07,R10解消

### 総合成功基準
- ✅ すべてのHigh Riskが解消
- ✅ 全スモークテストPASS（R01対策完了）
- ✅ ドキュメント完全同期（R05対策完了）
- ✅ 工数21時間以内（R04対策完了）

---

**このリスク分析マトリックスで、安全に超リファクタリングを完遂できます！**
**リスクを恐れず、適切に管理しながら進めましょう！**

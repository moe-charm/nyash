# Phase 15.75 — 完全脱Rust大作戦 📚 エントリーポイント

**最終更新**: 2025-10-14
**ステータス**: 計画段階 → 実装開始準備中
**目的**: Rust 99,406行 → 10,400行（89.5%削減）への完全ロードマップ

---

## ⚡ **超急ぎの人向け（3分で全部わかる）**

### 🎯 **Phase 15.75を1行で説明**

```
Rust 99,406行 → 10,400行（89.5%削減）を1ヶ月で実現
MIR疎結合により失敗時Rollback可能なゼロリスク計画
```

### 🚀 **今すぐやる3ステップ**

#### **ステップ1: スモークテスト確認（2分）**
```bash
bash tools/smokes/v2/run.sh --profile quick-selfhost
# 期待: 170/185 PASS（現状維持）
```

#### **ステップ2: TODO読む（5分）**
```bash
cat "docs/development/roadmap/phases/phase 15.75/TODO.md"

# 確認内容:
# - Phase 1（MirCall）準備タスク
# - plugin-on スモーク拡充
# - HostHandleRouter 段階移設
```

#### **ステップ3: 1つ選んで実装開始**
```bash
# Option A: 安全メソッドの whitelist を箱化
#   - normalize.rs に is_safe_core_method 追加

# Option B: Callable(argv 再構成) のパリティ確認
#   - arity>0 の methodRef.call(argv) 正常系スモーク追加

# Option C: HostHandleRouter 境界スモーク
#   - 型不一致を明示検出するスモーク追加
```

### 🚨 **失敗したらどうなる？**

```
✅ いつでもRollbackできます！

Level 1: バックエンド切替（0秒）
  ./hakorune-stage0.exe problem.hako

Level 2: Parser Rollback（5-10分）
  vim selfhost/compiler/parser.hako → 修正

Level 3: Full Rollback（10-30分）
  git checkout stage0-preserved
```

詳細: [STRATEGY.md](./STRATEGY.md)

---

## 🚀 **今すぐ読むべきドキュメント（優先順）**

### 1️⃣ **[TODO.md](./TODO.md)** ← 👈 **ここから始める！**
**読むべき人**: 今すぐ作業開始したい人
**内容**: 次の1-2週間でやるべきこと（P1スコープ）
**所要時間**: 5分

```
✅ 完了済み: Map.call P1, Fallback箱化, HostHandleRouter stub
📋 次のアクション:
  - Phase 1（MirCall）小粒・箱化方針で進行
  - plugin-on スモーク拡充
  - HostHandleRouter 段階移設
```

**結論**: 具体的な作業リストが欲しい → TODO.md

---

### 2️⃣ **[ROADMAP.md](./ROADMAP.md)** ← 全体像を知りたい人
**読むべき人**: 「なぜRust削減するのか」「どうやって実現するのか」を知りたい人
**内容**: Phase 15.75の全体戦略と詳細タスクリスト
**所要時間**: 15-20分

```
📊 削減計画:
  - Option A: 99,406行 → 10,400行（89.5%削減）
  - Phase 1-5: 淡々とタスクを消化

🎯 核心戦略:
  - Rust VM → Hakorune VM（自己実装）
  - Parser → Selfhost Parser（.hako実装）
  - Boxes → Plugins（プラグインシステム）
```

**結論**: 全体像と背景理論を理解したい → ROADMAP.md

---

### 3️⃣ **[STRATEGY.md](./STRATEGY.md)** ← 失敗が怖い人
**読むべき人**: 「本当に安全なのか」「失敗したらどうなるのか」が不安な人
**内容**: MIR疎結合によるゼロリスク戦略 + ツールチェーン要件
**所要時間**: 15分

```
🔄 Rollbackレベル:
  - Level 1: バックエンド切替（0秒）
  - Level 2: Parser Rollback（5-10分）
  - Level 3: Full Rollback（10-30分）

✅ 核心原則:
  - Stage 0/1/2並行運用（いつでも戻れる）
  - MIR = 疎結合ポイント（バックエンド交換可能）

🔧 ツールチェーン:
  - HakoBuildBox（500-800行）
  - HakoLspBox（1,200-2,000行）
```

**結論**: リスク管理とツールチェーンを知りたい → STRATEGY.md

---

### 4️⃣ **[ANALYSIS.md](./ANALYSIS.md)** ← 「1ヶ月で可能？」と思った人
**読むべき人**: 実績ベースの見積もり根拠を知りたい人
**内容**: Git履歴に基づく速度分析 + 3者見積もり比較
**所要時間**: 10分

```
📊 実績データ:
  - 21.6コミット/日（66日間）
  - 115行/日削減（直近13日間）
  - M2/M3達成: 63日（業界標準の29倍速）

🎯 結論:
  - ユーザー「1ヶ月」✅ 正しい（Phase 1-2コア完了）
  - Gemini「1年以上」❌ 業界標準ベース
  - Claude「6ヶ月」❌ 実績を無視した理論値
```

**結論**: 1ヶ月の根拠を知りたい → ANALYSIS.md

---

## 🎯 **あなたの状況別ガイド**

### 🚀 **「今すぐ作業したい」**
```
1. TODO.md 読む（5分）
2. 作業開始！
```

### 📚 **「全体を理解してから作業したい」**
```
1. ROADMAP.md 読む（15分）
   → 全体像理解
2. STRATEGY.md 読む（15分）
   → 安全性確認
3. TODO.md 読む（5分）
   → 作業開始
```

### 🔍 **「リスクが心配」**
```
1. STRATEGY.md 読む（15分）
   → Rollback手順確認
2. TODO.md 読む（5分）
   → 段階的作業開始
```

### 📊 **「1ヶ月で本当に可能？」**
```
1. ANALYSIS.md 読む（10分）
   → 実績ベースの根拠確認
2. ROADMAP.md 読む（15分）
   → 詳細計画確認
```

---

## 📊 **4つのドキュメントの関係図**

```
┌─────────────────────────────────────────┐
│   ROADMAP.md                             │
│   （全体戦略 + Phase別タスクリスト）      │
│   - なぜRust削減するのか                  │
│   - Phase 1-5 詳細スケジュール            │
│   - 週次TODO展開                         │
└────────────┬────────────────────────────┘
             │
    ┌────────┴────────┐
    │                 │
┌───▼──────────┐  ┌──▼─────────────────────┐
│ STRATEGY.md  │  │ ANALYSIS.md            │
│ （戦略・安全）│  │ （実績分析）            │
│ - Rollback   │  │ - Git履歴分析           │
│ - Stage並行  │  │ - 3者見積もり比較       │
│ - ツール要件  │  │ - 1ヶ月根拠             │
└───┬──────────┘  └────────────────────────┘
    │
    │ 実践↓
    │
┌───▼──────────────────────────────────────┐
│   TODO.md                                 │
│   （今すぐやること）                       │
│   - Phase 1（MirCall）小粒タスク           │
│   - plugin-on スモーク拡充                 │
│   - HostHandleRouter 段階移設             │
└───────────────────────────────────────────┘
```

---

## 🗺️ **Phase 15.75 完全手順マップ**

### **Phase 0（今）: 準備・計画確認**
```
✅ ドキュメント読了
  1. INDEX.md（このファイル）
  2. TODO.md
  3. ROADMAP.md

✅ 環境確認
  - スモークテスト実行: bash tools/smokes/v2/run.sh --profile quick-selfhost
  - 現状: 170/185 PASS（15 FAIL）
```

### **Phase 1: Hakorune VM完成（MirCall実装）**
```
📋 Phase 1タスク:
  - TODO.md の「Phase 1（MirCall）着手準備」参照
  - 小粒・箱化方針で段階実装
  - 15/16命令 → 16/16命令（100%完成）
```

### **Phase 2: Selfhost Parser実装**
```
📋 Phase 2タスク:
  - Parser.hako 実装
  - MIR出力確認（Rust Parser と一致）
  - 詳細は ROADMAP.md 参照
```

### **Phase 3-5: Boxes プラグイン化 + Runtime置き換え**
```
🔄 Phase 3: Box移行（src/boxes → plugins）
🔄 Phase 4: Runtime置き換え
🔄 Phase 5: AOT化

※ ROADMAP.md 参照
```

---

## 📝 **各ドキュメントの詳細情報**

| ドキュメント | サイズ | 読了時間 | 優先度 | 役割 |
|-------------|--------|---------|--------|------|
| **INDEX.md** | 11KB | 3分 | ★★★★★ | エントリーポイント |
| **TODO.md** | 7.4KB | 5分 | ★★★★★ | 今すぐやること |
| **ROADMAP.md** | 41KB | 15-20分 | ★★★★ | 全体戦略＋タスクリスト |
| **STRATEGY.md** | 30KB | 15分 | ★★★★ | 安全性保証＋ツール |
| **ANALYSIS.md** | 18.2KB | 10分 | ★★★ | 実績分析＋見積もり |

---

## 🚨 **よくある質問（FAQ）**

### Q1: どこから読めばいいの？
**A**: 👉 **[TODO.md](./TODO.md)** から始めてください（5分で読める）

### Q2: Phase 15.75って何？
**A**: Rust 99,406行 → 10,400行（89.5%削減）する計画です。詳細は [ROADMAP.md](./ROADMAP.md) 参照

### Q3: 失敗したらどうなるの？
**A**: MIR疎結合により、いつでもRollback可能です。詳細は [STRATEGY.md](./STRATEGY.md) 参照

### Q4: いつから始めるの？
**A**: **今すぐ**始められます。[TODO.md](./TODO.md) の「次アクション」参照

### Q5: どのくらい時間がかかるの？
**A**: **1ヶ月計画**（Phase 1-2コア完了）。詳細は [ANALYSIS.md](./ANALYSIS.md) 参照

### Q6: 1ヶ月で本当に可能？
**A**: Git履歴の実績ベースで可能です。詳細は [ANALYSIS.md](./ANALYSIS.md) 参照

---

## 🔗 **関連ドキュメント（外部リンク）**

### **上位ドキュメント**
- [00_MASTER_ROADMAP.md](../00_MASTER_ROADMAP.md) - Hakorune全体のマスタープラン
- [CURRENT_TASK.md](../../../../CURRENT_TASK.md) - 現在進行状況

### **参考ドキュメント**
- [INSTRUCTION_SET.md](../../../../reference/mir/INSTRUCTION_SET.md) - MIR命令セット
- [実行モード完全ガイド](../../../../guides/execution-modes-guide.md) - VM/LLVM/WASM実行

---

## 📋 **次のアクション（明確な手順）**

### **ステップ1: 理解する（30分）**
```bash
# 1. このファイル（INDEX.md）を読む（3分）✅ 完了！
# 2. TODO.md を読む（5分）
cat TODO.md

# 3. ROADMAP を流し読み（15分）
less ROADMAP.md

# 4. STRATEGY を流し読み（10分）
less STRATEGY.md
```

### **ステップ2: 現状確認（10分）**
```bash
# スモークテスト実行
bash tools/smokes/v2/run.sh --profile quick-selfhost

# 期待: 170/185 PASS（現状維持）
```

### **ステップ3: 作業開始（TODO.mdベース）**
```bash
# TODO.md の「次アクション」を1つずつ実行
# Phase 1（MirCall）小粒タスクから開始
```

---

## 🎯 **まとめ: Phase 15.75 を1行で説明**

```
Rust 99,406行 → 10,400行（89.5%削減）を1ヶ月で実現する、
MIR疎結合により失敗時Rollback可能なゼロリスク計画
```

---

## 📝 **変更履歴**

- 2025-10-14: ファイル整理（9→5ファイルに削減）
  - QUICKSTART.md を INDEX.md に統合
  - ROADMAP.md, STRATEGY.md, ANALYSIS.md 作成
  - リンク修正完了

- 2025-10-13: 初版作成（Claude Code）
  - 4つのドキュメントの整理・導線設計
  - 読む順番と優先度の明確化
  - FAQ・手順マップ追加

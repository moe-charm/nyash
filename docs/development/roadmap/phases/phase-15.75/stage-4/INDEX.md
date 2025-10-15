# Stage 4 — Dual Parser Harness（C ABI + Hako ABI）

**Phase 15.75 Stage 4**: Rust層を極限まで薄く（C ABI化）し、Hakoruneから呼び出す形にする

> Start Here（最短導線）
> - Minimal plan（2日）: ../stage-4-chatgpt/INDEX.md
> - Minimal C‑ABI spec: ../stage-4-chatgpt/C_ABI_MIN_SPEC.md
> - Integration (build.rs/Rust extern/runner): ../stage-4-chatgpt/INTEGRATION.md
> - extern_c Self‑Host strategy（次フェーズ方針）: ../stage-4-chatgpt/EXTERN_C_SELFHOST_STRATEGY.md

DoD（Stage‑4 完了条件）
- feature `parser-c-abi` で C ハーネスがビルドされる
- Rust 側 extern（rust/hakoヘッダ）がリンク可能（hakoは当面stubでも可）
- both モードの最小スモーク（ヘッダ比較：version/kind/stmts）が PASS
- 既存ランナー/スモークは既定設定で影響なし（opt‑in のみ有効）

Status（現状）
- C ハーネス/feature/スモークの導線は配置済み（最小ヘッダ）。hakoヘッダは stub（次で実装）。

---

## 🎯 **Stage 4の核心**

### **目標**: Rust Parser層を100-200行のC ABI層に縮退

```
┌─────────────────────────────────────────┐
│ Hakorune実装（.hako）                   │
│ - ParserHarnessBox                      │
│ - ParseResultBox/Comparator             │
│ - JSON v0ヘッダ生成・比較              │
│ (200-300行、Hakorune言語)              │
└─────────────────────────────────────────┘
           ↓ 呼び出し（C ABI経由）
┌─────────────────────────────────────────┐
│ C ABI層（極薄、100-200行）              │
│ - parse_source_dual()                   │
│ - free_parse_result()                   │
│ - 言語非依存インターフェース            │
│ (C言語)                                 │
└─────────────────────────────────────────┘
           ↓ 内部で呼び出し
┌─────────────────────────────────────────┐
│ Rust Parser（既存、縮小対象）           │
│ - Facade経由で呼び出し                  │
│ - JSON v0ヘッダ生成（追加実装）        │
│ (既存コード + 30行追加)                │
└─────────────────────────────────────────┘
```

**これがPhase 15.75の本質！**
- Rust層を極限まで薄く（C ABI、100-200行）
- 今後の標準: すべてC ABI経由でHakoruneから呼び出す
- 言語非依存（Rust/Hakorune/Python/他から呼び出し可能）

---

詳細ドキュメント（参照用）
- フル設計（Claude）: ./TECHNICAL_REQUIREMENTS.md, ./C_ABI_DESIGN_SPEC.md, ./SCHEDULE.md, ./RISK_ANALYSIS.md
- 最短導線（ChatGPT最小）: ../stage-4-chatgpt/INDEX.md

---

## 📚 **ドキュメント構成**

### **1. QUICKSTART.md** ⭐最優先
**3行要約 + 実装すべき3つのこと + 2日間スケジュール**
- 📄 [QUICKSTART.md](./QUICKSTART.md)
- 文字数: 6,840文字（3,500文字 + 拡張）

### **2. TECHNICAL_REQUIREMENTS.md** ⭐技術要件
**現状分析 + 技術要件 + 境界設計 + 受け入れ基準**
- 📄 [TECHNICAL_REQUIREMENTS.md](./TECHNICAL_REQUIREMENTS.md)
- 文字数: 35,898文字（完全版）
- 内容:
  - Rust Parser vs Hakorune Parser の現状分析
  - SMOKES_PARSER_MODE 実装要件
  - JSON v0ヘッダ仕様
  - Phase-A（MVP）vs Phase-B（完全版）

### **3. C_ABI_DESIGN_SPEC.md** ⭐C ABI設計
**データ構造 + 関数API + メモリ管理 + Cargo統合**
- 📄 [C_ABI_DESIGN_SPEC.md](./C_ABI_DESIGN_SPEC.md)
- 文字数: 29,892文字
- 内容:
  - ParserMode enum, ParseResult struct 設計
  - parse_source_dual(), free_parse_result() 実装
  - メモリ管理戦略（確保/解放ルール）
  - 完全なサンプルコード（150行）

### **4. HAKO_ABI_DESIGN_SPEC.md** ⭐Hako ABI設計
**Box設計 + C ABI連携 + Hakorune Parser統合**
- 📄 [HAKO_ABI_DESIGN_SPEC.md](./HAKO_ABI_DESIGN_SPEC.md) ✅ 完了
- 文字数: 27,162文字
- 内容:
  - ParserHarnessBox/ParseResultBox/Comparator 設計
  - C ABI連携（C ↔ Hako 呼び出しフロー）
  - Hakorune Parser統合（Selfhostコンパイラ呼び出し）
  - 完全実装例（300行）

### **5. SCHEDULE.md** ⭐詳細スケジュール
**2-3日間のタスク分解 + 各タスクの詳細 + Rollback方法**
- 📄 [SCHEDULE.md](./SCHEDULE.md)
- 文字数: 28,374文字
- 内容:
  - Day 1: C ABI層実装（8時間、4タスク）
  - Day 2: Hako ABI層実装（8時間、3タスク）
  - Day 3: テスト・検証（6時間、3タスク）
  - 各タスクの詳細（作業内容、成果物、検証方法、Rollback）

### **6. RISK_ANALYSIS.md** ⭐リスク分析
**18個のリスク + 3レベルRollback戦略 + 完了判定**
- 📄 [RISK_ANALYSIS.md](./RISK_ANALYSIS.md)
- 文字数: 26,933文字
- 内容:
  - 18個のリスク詳細分析（C言語、Hako ABI、統合、運用）
  - リスクマトリックス（スコア順）
  - 3レベルRollback戦略（5分/2分/30分）
  - Rollback判断基準
  - Phase 4完了判定（9項目チェックリスト）

---

## 🚀 **読む順序（推奨）**

### **超急ぎの人（5分）**
1. **QUICKSTART.md** の「3行要約」を読む
2. **QUICKSTART.md** の「実装すべき3つのこと」を確認

### **実装開始したい人（30分）**
1. **QUICKSTART.md** 全体を読む（6,840文字）
2. **C_ABI_DESIGN_SPEC.md** のサンプルコードを確認
3. **SCHEDULE.md** のDay 1タスクを確認

### **完全に理解したい人（2時間）**
1. **TECHNICAL_REQUIREMENTS.md** で現状分析・技術要件を理解
2. **C_ABI_DESIGN_SPEC.md** でC ABI層を完全理解
3. **HAKO_ABI_DESIGN_SPEC.md** でHako ABI層を完全理解
4. **RISK_ANALYSIS.md** でリスクとRollback戦略を確認

### **実装中の人（都度参照）**
- **SCHEDULE.md** で進捗確認（各タスクの検証方法）
- **RISK_ANALYSIS.md** で問題発生時のRollback方法確認

---

## 🎯 **Phase 15.75全体との関係**

### **Stage構成**
```
Phase 15.75 — 完全脱Rust大作戦（1ヶ月）
├── Stage 0-3: Boxes Migration（進行中、2025-10-16完了見込み）
│   └── src/boxes/ の参照を plugin/HostHandleRouter に移行
├── Stage 4: Dual Parser Harness（このフォルダ、2-3日）⭐
│   └── Rust Parser層をC ABI化（100-200行）
├── Stage 5: Parser完全移行（1週間）
│   └── Parser/Tokenizer (7,637行) を Hakorune実装に置き換え
└── Stage 6-N: 他のコンポーネントも同様にC ABI化
```

### **Stage 4の位置づけ**
- **直前**: Stage 3（Boxes Migration）完了
- **直後**: Stage 5（Parser完全移行）開始
- **依存**: Stage 3完了が前提（plugin-only build 緑）
- **影響**: Parser以外のコンポーネントにも適用される標準パターン

---

## 🔥 **Stage 4の重要性**

### **1. C ABI層の確立**
Stage 4で確立するC ABI層は、今後のすべてのコンポーネントで再利用されます：
- Stage 5: Parser C ABI
- Stage 6: Resolver C ABI
- Stage 7: MIR Builder C ABI
- ...

### **2. Rust層の極限縮退**
Rust 99,406行 → 10,400行の削減において、Stage 4は重要なマイルストーン：
- Parser/Tokenizer: 7,637行 → 100-200行（C ABI層のみ）
- 削減率: 97.4%

### **3. 言語非依存の実現**
C ABI経由で呼び出すことで：
- Rust から呼び出し可能
- Hakorune から呼び出し可能
- Python から呼び出し可能（将来）
- 他の言語からも呼び出し可能（将来）

---

## ✅ **受け入れ基準（DoD）**

Stage 4完了条件：
- [ ] C ABI層実装完了（100-200行）
- [ ] Hako ABI層実装完了（200-300行）
- [ ] SMOKES_PARSER_MODE=rust 成功
- [ ] SMOKES_PARSER_MODE=hako 成功
- [ ] SMOKES_PARSER_MODE=both 成功（比較一致）
- [ ] quick-selfhost 170/185 PASS 維持（最重要！）
- [ ] ビルド時間増加 10% 以内
- [ ] メモリリーク 0KB
- [ ] valgrind エラー 0件
- [ ] ドキュメント更新完了

---

## 🚨 **Rollback戦略**

### **Level 1: 最速Rollback（5分）**
```bash
rm -rf src/parser_harness/
git checkout build.rs
cargo build --release
```

### **Level 2: feature flag無効化（2分）**
```bash
cargo build --release  # C ABI無効（デフォルト）
cargo build --release --features parser-c-abi  # 有効
```

### **Level 3: Full Rollback（30分）**
```bash
git revert --no-commit HEAD~3..HEAD
git commit -m "Rollback: Stage 4全削除"
cargo build --release
bash tools/smokes/v2/run.sh --profile quick-selfhost
# 170/185 PASS 復帰確認
```

---

## 📈 **期待される成果**

### **短期（Stage 4完了後）**
- ✅ Rust Parser層をC ABI化（7,637行 → 100-200行）
- ✅ Hakorune側で統一ハーネス実装（ParserHarnessBox）
- ✅ SMOKES_PARSER_MODE環境変数による切り替え
- ✅ Stage 5（Parser完全移行）への道筋確保

### **中期（Stage 5完了後）**
- ✅ Parser/Tokenizer (7,637行) の完全削除
- ✅ Rust実装 → Hakorune実装への完全移行
- ✅ 脱Rust化の大きな前進

### **長期（Phase 15.75完了後）**
- ✅ Rust 99,406行 → 10,400行（89.5%削減）
- ✅ C ABI標準パターンの確立
- ✅ 言語非依存アーキテクチャの実現

---

## 🔍 **関連ドキュメント**

### **Phase 15.75全体**
- [INDEX.md](../INDEX.md) - Phase 15.75エントリポイント
- [TODO.md](../TODO.md) - 現在のタスクリスト
- [ROADMAP.md](../ROADMAP.md) - 全体ロードマップ
- [STRATEGY.md](../STRATEGY.md) - MIR疎結合戦略
- [ANALYSIS.md](../ANALYSIS.md) - 1ヶ月完了見積もりの根拠

### **他のStage**
- Stage 3: [PHASE_3_BOXES_MIGRATION.md](../PHASE_3_BOXES_MIGRATION.md)
- Stage 5: （作成予定）

---

## 💡 **次のアクション**

1. **QUICKSTART.md を読む**（5分）
2. **TECHNICAL_REQUIREMENTS.md で現状分析**（30分）
3. **C_ABI_DESIGN_SPEC.md でC ABI設計を理解**（30分）
4. **HAKO_ABI_DESIGN_SPEC.md を作成**（1時間）⭐未実施
5. **実装開始**（SCHEDULE.md に従う）

---

**作成者**: Claude (Sonnet 4.5) + Task Agent 5個
**作成日**: 2025-10-14
**最終更新**: 2025-10-14
**ステータス**: ドキュメント完成（7ファイル、165KB）、実装待ち

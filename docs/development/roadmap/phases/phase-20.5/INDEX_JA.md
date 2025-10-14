# Phase 20.5 — 索引（日本語版）

**脱Rust大作戦: Gate方式による段階的実装**

状態: 実行中（Gate A実装中）
期間: 6-8週間 (2025-12-21 - 2026-02-中旬)

---

## 📚 ドキュメント構造

### 核心ドキュメント

1. **[PLAN.md](PLAN.md)** ⭐ **実行中の計画書（英語）**
   - 5行サマリー / 最小命令セット / DoD
   - Gate A〜E（Parser→MIR→VM PoC→op_eq→統合）
   - テスト/CI/リスク/次ステップ

2. **[README.md](README.md)** ⭐ **全体概要（日本語）**
   - フェーズ概要とゴール
   - 週次計画（Week 1-6）
   - 成功基準とDoD
   - **Hakorune VM発見による更新版**

3. **[HAKORUNE_VM_DISCOVERY.md](HAKORUNE_VM_DISCOVERY.md)** ⭐ **重大発見レポート（英語）**
   - Hakorune VM完全実装の発見（3,413行、22ハンドラー）
   - アーキテクチャ解析
   - Phase 20.5戦略の変更（36週間→6週間）

4. **[STRATEGY_RECONCILIATION.md](STRATEGY_RECONCILIATION.md)** - 戦略比較
   - C Code Generator vs Pure Hakorune Strategy
   - リスク評価、信頼度評価
   - Pure Hakorune戦略採用の理由

5. **[PURE_HAKORUNE_ROADMAP.md](PURE_HAKORUNE_ROADMAP.md)** - 全体ロードマップ
   - Phase 20.5-20.8の計画（36週間）
   - 6フェーズ構成（Phase A-F）
   - Golden Testing戦略

---

## 🎯 クイックリファレンス（5行サマリー）

1. **Goal（目標）**: 脱Rust。凍結EXEを土台に自己ホストへ前進。
2. **Strategy（戦略）**: Gate方式（Parser→MIR→VM PoC→op_eq→統合）。
3. **Boundary（境界）**: C-ABI/HostBridgeのみ外部境界。中はEverything is Box。
4. **Proof（証明）**: 決定性（JSON正規化）、Golden＆固定点で検証。
5. **Policy（方針）**: 小さく、順序よく、SKIPはWARN、回帰のみFAIL。

---

## 📊 Phase 20.5のGate構成

```
Gate A（実装中）: Parser v1（Hakorune製）→ Canonical AST JSON
  ├─ DoD: ~10ケース PASS、非決定性ゼロ、キーソート
  ├─ CLI: --dump-ast-json, --emit-ast-json
  └─ Smoke: 9件追加済み（quick-selfhost）

Gate B（次）: MIR Builder v1（最小16命令）
  ├─ DoD: 16命令到達可能、ゴールデンJSON一致
  └─ 4本スモーク（const+ret, add+ret, eq+branch, lt+branch）

Gate C（その次）: VM Foundations PoC（5命令 via HostBridge）
  ├─ 命令: const, binop, compare, jump, ret
  ├─ HostBridge経由で実行
  └─ DoD: シンプルなプログラムend-to-end実行

Gate D（その次）: op_eq Migration（下準備）
  ├─ NoOperatorGuard + 8型
  └─ 5-8ゴールデンケース

Gate E（最後）: Integration & Docs
  ├─ E2E 10ケース PASS
  └─ ドキュメント完備
```

---

## 🎯 主要成果物

### 1️⃣ **Parser canonical JSON**（Gate A）
- Hakorune製Parserが決定的AST JSON出力
- キー辞書順ソート、非決定性排除
- v1==v2==v3検証の土台

### 2️⃣ **MIR Builder v1**（Gate B）
- 最小16命令サポート
- MIR JSON正規化
- Rust MIR Builder vs Hakorune MIR Builder比較

### 3️⃣ **VM Foundations PoC**（Gate C）
- 5命令のみ（const/binop/compare/jump/ret）
- HostBridge経由で実行
- Full Hakorune VM（selfhost/hakorune-vm/）への橋渡し

### 4️⃣ **Full Hakorune VM統合**（Gate C+）
- 既存実装（3,413行、22ハンドラー）
- HostBridge経由で実行可能に
- Rust VM vs Hakorune VM Golden Testing

---

## 🔄 全体フロー

```
【Phase 20.5実装フロー】

Week 1-2: Gate A/B（ChatGPT主導）
  ├─ Parser canonical JSON実装
  ├─ MIR Builder v1実装
  └─ MirIoBox.normalize()統合

Week 3-4: Gate C（ChatGPT + Claude協調）
  ├─ VM Foundations PoC（5命令）
  ├─ HostBridge設計・実装
  └─ Full Hakorune VM統合準備

Week 5: Full VM統合（両者協調）
  ├─ 5命令PoC → 22命令Full VM拡張
  ├─ selfhost/hakorune-vm/ 統合
  └─ HostBridge経由実行確認

Week 6: Gate D/E（Claude主導）
  ├─ op_eq Migration
  ├─ Golden Testing（Rust-VM vs Hako-VM）
  └─ CLI統合（--backend vm-hako）

Week 7-8: Integration & Docs
  ├─ E2E 10ケース
  ├─ ドキュメント整備
  └─ CI minimal green
```

---

## 🧪 テスト戦略

### テストピラミッド

```
         /\
        /  \    Level 3: Self-Compilation（1テスト、遅い）
       /____\   - v1が自分自身をコンパイル → v2
      /      \  - v2が自分自身をコンパイル → v3
     /        \ - 検証: v2 == v3
    /__________\
   /            \ Level 2: 包括的（10テスト、中速）
  /              \ - if/else、ループ、関数、Box、配列
 /________________\
/                  \ Level 1: スモークテスト（43+テスト、高速）
                     - 各MIR命令
                     - 基本パース
                     - JSON正規化
```

---

## 📦 ディレクトリ構造

```
Phase 20.5計画:
docs/development/roadmap/phases/phase-20.5/
├── INDEX_JA.md                       # ← このファイル（日本語版）
├── INDEX.md                          # 英語版索引
├── README.md                         # Phase概要（日本語）
├── PLAN.md                           # Gate実行計画（英語）
├── HAKORUNE_VM_DISCOVERY.md          # VM発見レポート（英語）
├── STRATEGY_RECONCILIATION.md        # 戦略比較（英語）
└── PURE_HAKORUNE_ROADMAP.md          # 全体ロードマップ（英語）

実装:
selfhost/hakorune-vm/                 # Full Hakorune VM（既存）
├── hakorune_vm_core.hako             # VM実行ループ
├── instruction_dispatcher.hako       # @match dispatch
├── *_handler.hako                    # 22ハンドラー
└── tests/*.hako                      # 26+テスト

selfhost/shared/mir/
└── mir_io_box.hako                   # MIR JSON I/O（tomoaki作）

selfhost/shared/json/
└── json_canonical_box.hako           # JSON正規化（ChatGPT追加）
```

---

## 🎯 成功基準サマリー

### 技術的基準

- [ ] Gate A: Parser canonical JSON（~10ケースPASS）
- [ ] Gate B: MIR Builder v1（16命令到達可能）
- [ ] Gate C: VM PoC（5命令end-to-end実行）
- [ ] Gate C+: Full Hakorune VM統合（22ハンドラー）
- [ ] Gate D: op_eq Migration（5-8ゴールデンPASS）
- [ ] Gate E: 統合テスト（E2E 10ケースPASS）

### パフォーマンス

- [ ] VM PoC実行時間: 測定可能
- [ ] Full VM性能: Rust VM比≥50%
- [ ] メモリ使用量: < 100MB

### 品質

- [ ] ドキュメント完備（日本語+英語）
- [ ] コードモジュール性（Box-based）
- [ ] エッジケース網羅
- [ ] レビュー承認（ChatGPT + Claude）

---

## 📚 関連フェーズ

### 前フェーズ

- [Phase 15.77 - 凍結EXE確定](../phase-15.77/)
- [Phase 15.76 - extern_c & Frozen Toolchain](../phase-15.76/)
- [Phase 15.75 - 脱Rust計画](../phase-15.75/)

### 次フェーズ

- **Phase 20.6 - Rust完全削除**
  - VM executor → Hakorune実装
  - Rustコードベース → 0行
  - Pure Hakorune自己ホスト

---

## 💬 開発体制

### 実装担当

- **tomoaki**: Hakorune VM実装完了（selfhost/hakorune-vm/）、MirIoBox設計
- **ChatGPT**: Gate A/B/C実装（Parser → MIR Builder → VM PoC）
- **Claude**: Full VM統合、Golden Testing、ドキュメント整備

### レビュー方針

- 各Week終了時: Self-review + スモークテスト
- Week 4, 6: Full review（ChatGPT + Claude）
- ブロッキング問題: 即座にエスカレーション

---

## 🔗 外部リソース

### 業界事例

- **Rust Bootstrap**: [Rust stage0 documentation](https://rustc-dev-guide.rust-lang.org/building/bootstrapping.html)
- **Go Bootstrap**: [Go 1.5 Bootstrap Process](https://go.dev/doc/go1.5#bootstrap)
- **OCaml Bootstrap**: [OCaml self-hosting](https://ocaml.org/docs/compiling-ocaml-projects)

### 論文

- [Rapid Self-Hosting Paper](../../../../private/papers-active/rapid-selfhost-ai-collaboration/)
- [Reflections on Trusting Trust](https://www.cs.cmu.edu/~rdriley/487/papers/Thompson_1984_ReflectiononTrustingTrust.pdf) (Ken Thompson, 1984)

---

## 📝 命名規則

### Stage命名

- **Stage 1**: Rustコンパイラ、凍結EXE (`hako-frozen-v1`)
- **Stage 2**: Hakoruneコンパイラ v1 (`bootstrap_v1`)
- **Stage 3**: Hakoruneコンパイラ v2 (`bootstrap_v2`)
- **Fixed Point**: v2 == v3（自己一貫性）

### ファイル命名

- テスト出力: `test_v1.json`, `test_v2.json`
- Bootstrap出力: `bootstrap_v2.c`, `bootstrap_v3.c`
- 常にバージョン接尾辞を含める

### 検証

- 完全一致比較: `diff`
- ハッシュチェック: `md5sum`
- 自動化: `tools/verify_bootstrap_chain.sh`

---

## 🌟 重要ポイント

### **Hakorune VM発見の意義**

- ✅ **36週間→6週間に短縮**（83%削減）
- ✅ **実装完了**: 3,413行、22ハンドラー、26+テスト
- ✅ **検証・統合に集中**: 実装ではなく、動作確認とGolden Testing

### **Gate方式の利点**

- ✅ **段階的進行**: 各Gateは独立してテスト可能
- ✅ **ロールバック容易**: 問題発生時は前Gateに戻れる
- ✅ **進捗可視化**: Gate完了=具体的マイルストーン

### **Pure Hakorune戦略**

- ✅ **"Rust=floor, Hakorune=house"**: Rust最小化（HostBridgeのみ）
- ✅ **単一実行パス**: Hakorune VMのみ
- ✅ **究極のBox理論**: VMもBoxで実装

---

**作成日**: 2025-10-14
**最終更新**: 2025-10-14
**状態**: Gate A実装中
**次回レビュー**: Week 2終了時（Gate B完了後）

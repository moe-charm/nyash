# 🚀 Hakorune開発マスタープラン

Status: Active Development
Last Updated: 2025-10-06
Purpose: Claude×Copilot×ChatGPT×Gemini協調開発の総合ロードマップ

## 📍 現在位置

- **現在フェーズ**: Phase 15.7 セルフホスティング実現への道筋
- **進捗**: 85-90%完成（残り10-15%、1-2週間）
- **次フェーズ**: Phase 15.7完了 → セルフホスティング達成！
- **備考**: Hakorune言語で Hakorune をコンパイル・実行する完全なセルフホスティングへ

---

## 🗺️ フェーズ概要

### ✅ **完了済みフェーズ（Phase 1-14）**

| Phase | 状態 | 概要 | 詳細リンク |
|-------|------|------|------------|
| 1-7 | ✅完了 | 言語設計・パーサー・基本機能実装 | - |
| 8.4 | ✅完了 | AST→MIR Lowering完全実装 | [phase_8_4_ast_mir_lowering.md](phase-8/phase_8_4_ast_mir_lowering.md) |
| 8.5 | ✅完了 | MIRダイエット（35→26→15命令） | [phase_8_5_mir_35_to_26_reduction.md](phase-8/phase_8_5_mir_35_to_26_reduction.md) |
| 8.6 | ✅完了 | VM性能改善（0.9倍→2倍以上） | [phase_8_6_vm_performance_improvement.md](phase-8/phase_8_6_vm_performance_improvement.md) |
| 9 | ✅完了 | プラグインシステム基盤 | [phase-9/](phase-9/) |
| 9.75g-0 | ✅完了 | BID-FFI Plugin System | [Phase-9.75g-0-BID-FFI-Developer-Guide.md](phase-9/Phase-9.75g-0-BID-FFI-Developer-Guide.md) |
| 10 | 📋計画 | Cranelift JIT（主経路、将来検討） | [phase_10_cranelift_jit_backend.md](phase-10/phase_10_cranelift_jit_backend.md) |
| 11 | ✅完了 | LLVM統合・AOT実装 | [phase-11/](phase-11/) |
| 11.8 | 📋計画 | MIR整理（Core-15→Core-13） | [phase-11.8_mir_cleanup/](phase-11.8_mir_cleanup/) |
| 12 | ✅完了 | TypeBox統合ABI | [phase-12/](phase-12/) |
| 13 | 📋計画 | Hakoruneブラウザー革命 | [phase-13/](phase-13/) |
| 14 | 📋計画 | パッケージング・CI改善 | [phase-14/](phase-14/) |

---

### 🔥 **Phase 15系: セルフホスティング実現（進行中）**

#### ✅ **Phase 15: セルフホスティング基盤**
- **概要**: Hakoruneコンパイラ・Hakorune VM実装の総合計画
- **詳細**: [phase-15/](phase-15/)

#### ✅ **Phase 15.5: JSON v0中心化・統一Call基盤革命** (2025-09 完了)
- **概要**: セルフホスティング前の基盤アーキテクチャ大改革
- **成果**:
  - Core Box統一
  - MIR命令安定化
  - LLVM PHI安定化
  - 型変換統一化
- **詳細**: [phase-15.5/README.md](phase-15.5/README.md)

#### 🔥 **Phase 15.7: セルフホスティング実現への道筋 - Hakoruneコンパイラ完成計画** (進行中)
- **期間**: 2025-09-28 ~ 2025-10-20（予想）
- **進捗**: **85-90%完成**（残り10-15%、1-2週間）
- **目標**: Hakorune言語で Hakorune をコンパイル・実行する
- **詳細**: [phase-15.7/README.md](phase-15.7/README.md)

**完了済み（85-90%）**:
- ✅ P2-A/B/C（Using解決系）完全実装
  - UsingResolverBox実装（1日で完了、見積もり7日 → **85%短縮！**）
  - NamespaceBox実装（1日で完了、見積もり5日 → **80%短縮！**）
  - Pipeline V2統合（1日で完了、見積もり3日 → **67%短縮！**）
- ✅ SignatureVerifier/MethodRegistry（品質強化）
- ✅ Hakorune VM基盤（InstructionScannerBox/OpHandlersBox/ProgramStateBox等）
- ✅ FlowRunner/JsonProgramBox
- ✅ Pipeline V2基礎実装
- ✅ Quick smokes 常緑（172/172 PASS）
- ✅ TimerBox実装完了
- ✅ Hakorune VM への改名完了（Mini-VM → Hakorune VM）

**残り10-15%（1-2週間）**:
- 🔥 **Hakorune VM命令拡張（最後の砦）**
  - newbox（2日・最重要）← **今ココ！**
  - boxcall（2日・最重要）
  - phi（2日）
  - load/store（2日）
  - externcall（1日）
- 🔲 セルフホストループE2E（1週間）

**教訓（Lessons Learned）**:
1. **Box-First設計の威力**: 新機能追加が予想の**9倍速**
2. **見積もりの精度**: 初期見積もりは慎重すぎた
3. **並行開発の難しさ**: 実際は順次開発が正解
4. **品質ファーストの重要性**: 計画外の成果が大きい

#### ✅ **Phase 15.8: LLVM→WASM実装** (2025-10-01~10-22 完了)
- **概要**: MIR16命令をWASMに変換、ブラウザ/エッジ環境で実行可能に
- **成果**: WebAssembly完全対応
- **詳細**: [phase-15.8/README.md](phase-15.8/README.md)

#### ✅ **Phase 15.9: VmConfig集約化** (2025-10-05 完了)
- **概要**: 環境変数42ファイル散在→1箇所集約
- **成果**: パフォーマンス向上、管理性改善
- **コミット**: `f1874b3b`

#### ✅ **Phase 15.10: Legacy Code大掃除** (2025-10-05 完了)
- **概要**: 2大ファイル→8小ファイル分割
- **成果**: デッドコード470行削除、純削減400行
- **コミット**: `43679766`, `f6cbbf48`, `f1f3b83e`
- **詳細**: legacy.rs分割（calls:617行 + boxes:518行）

#### ✅ **Phase 15.11: StringHelpers共通ライブラリ箱化** (2025-10-05 完了)
- **概要**: セルフホストコード重複削減
- **成果**: 14ファイル統合で335行純削減
- **詳細**:
  - Phase 15.11: 319行削減 (380削除 - 61追加)
  - Phase 15.11.1: 15行削減 (22削除 - 7追加) - ChatGPT協力
  - 合計削減: 335行
  - 重複削除: 7種類のヘルパー関数を統合
- **コミット**: `6ba6b026` (本体), `d07f3af3` (追加統合), `0de80fa6` (docs)

#### 📋 **Phase 15.12候補: index_of_from統一**
- **概要**: `index_of_from` → CfgNavigatorBox統合
- **見込み**: 60-100行削減
- **詳細**: `docs/development/proposals/ideas/improvements/phase-15-12-index-of-from-consolidation.md`

---

### 🌟 **Phase 16-20系: マクロシステム（計画・一部完了）**

#### ✅ **Phase 16: マクロ革命（Rust実装版）** (2025-09-19 完了、バグあり)
- **状態**: Rust実装完了、バグ発見により暫定対応中
- **成果**:
  - AST Pattern Matching実装
  - Quote/Unquote実装
  - Match式構文実装
  - @derive(Equals, ToString)実装
  - @test ランナー実装
- **問題**: フル機能にバグあり
- **暫定対応**: `.hako`で単純置き換えマクロ実装中（`apps/macros/`）
- **詳細**: [phase-16-macro-revolution/README.md](phase-16-macro-revolution/README.md)

#### 📋 **Phase 20: マクロフル機能（Hakorune実装版）** (Phase 15.7完了後)
- **状態**: 計画中（セルフホスティング完了後に開始）
- **目的**: Phase 16のRust実装をHakoruneでセルフホスト実装に書き直す
- **優先機能**:
  1. @derive(Equals, ToString) - 最も便利
  2. @test ランナー - セルフホストコードのテスト自動化
  3. AST Pattern Matching - 複雑なマクロ基盤
  4. Quote/Unquote - テンプレート生成
- **詳細**: [phase-20-macro-full-features/README.md](phase-20-macro-full-features/README.md)
- **移行ガイド**: [phase-20-macro-full-features/MIGRATION.md](phase-20-macro-full-features/MIGRATION.md)

### 🌟 **Phase 17以降（将来計画）**

| Phase | 状態 | 概要 | 詳細リンク |
|-------|------|------|------------|
| 17 | 🧪計画中 | LoopForm Self‑Hosting＋Hakorune VM | [phase-17-loopform-selfhost/](phase-17-loopform-selfhost/) |
| 17+ | 💡候補 | Rust所有権統合（オプショナル） | [rust-ownership-fusion](../../private/ideas/new-features/2025-09-22-rust-ownership-fusion.md) |
| 22 | 💡構想 | Hakorune LLVM Compiler | [phase-22/README.md](phase-22/README.md) |

---

## 🎯 Hakorune実行モード併用戦略

### 🌟 インタープリター＋コンパイラ併用の価値

#### 実行モード使い分け
```
開発時: Rust VM（デバッグ・即時実行）
本番時: Rust VM（Pythonのように実用的）
        OR
        WASM/AOT（性能要求時）
配布時: AOT native（最高性能）
Web時:  WASM（ブラウザ対応）
```

#### インタープリターの強み
- **即時実行**: コンパイル不要で高速開発
- **デバッグ容易**: 実行時情報の完全把握
- **非同期完全対応**: Rust async/awaitで真の並行処理
- **動的性**: 実行時評価・REPL対応
- **十分な性能**: 多くのユースケースで実用的（Pythonが証明）

---

## 🚀 **Phase 15.7詳細: セルフホスティング実現への道筋**

### 🎯 **Phase 15.7の真の目的**

**「Hakorune で Hakorune をコンパイルする」完全なセルフホスティングの実現**

### 🔄 **セルフホストループの具体的4ステップ**

```
┌──────────────────────────────────────┐
│ Step 1: .hako ソース解析             │
│    ↓                                  │
│ Step 2: MIR JSON生成（コンパイラ）   │
│    ↓                                  │
│ Step 3: MIR JSON実行（Hakorune VM）  │
│    ↓                                  │
│ Step 4: 出力検証（パリティテスト）    │
└──────────────────────────────────────┘
```

### 📅 **推奨実装順序**

#### **Week 1-2: Hakoruneコンパイラ MVP完成（P2優先）** ✅ 完了！
- ~~Day 1-2: branch/jump最小生成~~
- ~~Day 3: LocalSSA.ensure_cond最終化~~
- ~~Day 4-7: UsingResolverBox実装~~
- ~~Day 8-10: NamespaceBox実装~~
- ~~Day 11-14: Pipeline V2統合（using解決）~~

#### **Week 3: Hakorune VM命令拡張（最優先）** 🔥 今ココ！
- Day 1-2: newbox実装（Box生成）
- Day 3-4: boxcall + phi 並行実装
- Day 5-6: load/store + externcall 並行実装
- Day 7: 統合テスト・スモークテスト

#### **Week 4: セルフホストループE2E**
- Day 1-2: .hakoソース→MIR JSON生成確認
- Day 3-4: MIR JSON→Hakorune VM実行確認
- Day 5-6: パリティテスト（Rust VM vs Hakorune VM）
- Day 7: ブートストラップ達成！🎉

### 📊 **実装優先度マトリックス（2025-10-06更新）**

| 項目                   | 優先度   | ステータス | 理由       | 見積 | 実績 | 担当領域 |
|----------------------|-------|-------|----------|------|------|------|
| branch/jump生成        | 🔴 P2 | ✅完了 | 制御フロー必須  | 2日   | 2日 | コンパイラ |
| LocalSSA.ensure_cond | 🔴 P2 | ✅完了 | 条件分岐安定化  | 1日   | 1日 | コンパイラ |
| **UsingResolverBox実装** | 🔴 P2-A | ✅**完了** | **using解決の核心** | 1週間 | **1日**✨ | コンパイラ |
| **NamespaceBox実装** | 🔴 P2-B | ✅**完了** | 名前空間解決 | 5日 | **1日**✨ | コンパイラ |
| **Pipeline V2統合（using）** | 🔴 P2-C | ✅**完了** | using→MIR変換 | 3日 | **1日**✨ | コンパイラ |
| **SignatureVerifier** | - | ✅**完了** | **計画外追加** | - | **1日** | コンパイラ |
| **MethodRegistry拡大** | - | ✅**完了** | **計画外追加** | - | **1日** | コンパイラ |
| **JsonCursorBox採用** | - | ✅**完了** | **計画外追加** | - | **1日** | 共通 |
| **Hakorune VM改名** | - | ✅**完了** | **ブランディング統一** | - | **1日** | Hakorune VM |
| **Hakorune VM newbox実装** | 🟡 P1-A | 🔥**最優先** | **Box生成（最重要！）** | 2日 | **未着手** | Hakorune VM |
| **Hakorune VM boxcall実装** | 🟡 P1-B | 🔥未着手 | **メソッド呼び出し** | 2日 | **未着手** | Hakorune VM |
| Hakorune VM phi実装 | 🟡 P1-C | 📝計画 | SSA合流 | 2日 | 未着手 | Hakorune VM |
| Hakorune VM load/store実装 | 🟡 P1-D | 📝計画 | メモリアクセス | 2日 | 未着手 | Hakorune VM |
| Hakorune VM externcall実装 | 🟡 P1-E | 📝計画 | print等外部呼び出し | 1日 | 未着手 | Hakorune VM |
| match式完全対応 | 🟡 P1-F | 📝計画 | 頻繁に使用 | 2日 | 未着手 | コンパイラ |
| Hakorune VM unaryop/typeop | 🟢 P3-A | 📝計画 | 単項演算・型操作 | 2日 | 未着手 | Hakorune VM |
| 最適化パス | 🟢 P3-B | 📝計画 | 性能向上 | 1週間 | 未着手 | コンパイラ |
| エラーハンドリング | 🟢 P3-C | 📝計画 | UX向上 | 3日 | 未着手 | コンパイラ |

**凡例**:
- 🔴 P2: 最優先（セルフホスティング必須）
- 🟡 P1: 高優先度（基本機能実装）
- 🟢 P3: 中優先度（改善・UX向上）
- ✅完了 / 🔥最優先 / 🔥未着手 / 📝計画 / ✨予想より早い達成

**達成状況**:
- ✅ **P2系完全達成**（コンパイラー側：using解決・品質強化）
- 🔥 **P1系が最優先**（Hakorune VM命令拡張：残り10-15%）

### 🎯 **達成基準（明確な終了条件）**

✅ **Phase 15.7完了 = 以下すべて満たす**:
1. UsingResolverBox/NamespaceBox動作
2. Hakorune VM 14命令すべて実装
3. .hakoソース→MIR JSON→Hakorune VM実行成功
4. c0（Rustコンパイラ）→c1（Hakoruneコンパイラ）動作
5. c1→c1'（自己コンパイル）成功
6. Quick smokes 全PASS維持

---

## 📊 Phase 15系の全体進捗

```
Phase 15系全体進捗: ████████████░ 85-90%完成

完了済み：
✅ Phase 15: セルフホスティング基盤設計
✅ Phase 15.5: JSON v0中心化・統一Call基盤革命
✅ Phase 15.8: LLVM→WASM実装
✅ Phase 15.9: VmConfig集約化
✅ Phase 15.10: Legacy Code大掃除
✅ Phase 15.11: StringHelpers共通ライブラリ箱化
🔥 Phase 15.7: セルフホスティング実現（85-90%、残り1-2週間）

次のステップ：
1. Hakorune VM命令拡張（newbox/boxcall/phi/load/store/externcall）
2. セルフホストループE2E検証
3. ブートストラップ達成！🎉
```

---

## 🏗️ 重要な設計原則

### 🧱 Box-First原則: 「箱理論」で足場を積む

Hakoruneは「Everything is Box」。実装・最適化・検証のすべてを「箱」で分離・固定し、いつでも戻せる足場を積み木のように重ねる。

#### 実践テンプレート（開発時の合言葉）
- 「箱にする」: 設定・状態・橋渡しはBox化（例: JitConfigBox, HandleRegistry）
- 「境界を作る」: 変換は境界1箇所で（VMValue↔JitValue, Handle↔Arc）
- 「戻せる」: フラグ・feature・env/Boxで切替。panic→フォールバック経路を常設
- 「見える化」: ダンプ/JSON/DOTで可視化、回帰テストを最小構成で先に入れる
- 「Fail-Fast」: エラーは隠さず即座に失敗。フォールバックより明示的エラー

### 🏗️ Everything is Box
- すべての値がBox（StringBox, IntegerBox, BoolBox等）
- ユーザー定義Box: `box ClassName { field1: TypeBox field2: TypeBox }`
- **MIR凍結セット**: 16命令で全機能実現！

---

## 🎓 重要な教訓（Lessons Learned）

### ✅ **Phase 15.7からの学び**

1. **Box-First設計の威力**
   - 新機能追加が予想の**9倍速**で完了
   - UsingResolver/Namespace実装は1日ずつで完了（見積もり18日 → 実績2日）
   - Pipeline V2の強固な設計が成功の鍵

2. **見積もりの精度**
   - 初期見積もりは慎重すぎた
   - コンパイラー側: 見積もり18日 → 実績2日
   - 基盤の成熟度を過小評価していた

3. **並行開発の難しさ**
   - 実際は順次開発が正解
   - Using解決がモジュールシステムの基盤
   - コンパイラー完成 → Hakorune VM拡張の順が合理的

4. **品質ファーストの重要性**
   - 計画外の成果が大きい
   - SignatureVerifier/MethodRegistry/JsonCursorBox
   - Fail-Fast文化の確立が開発速度を加速

---

## 📞 連絡・相談方法

技術的相談や進捗報告は、以下の方法でお気軽にどうぞ：

1. 📝 GitHub Issues・Pull Request
2. 📋 docs/CURRENT_TASK.md コメント
3. 🤖 AI大会議 (重要な技術決定)
4. 💬 コミットメッセージでの進捗共有

どんな小さなことでも相談大歓迎です！
一緒にHakoruneを最高の言語にしていきましょう🚀

---

**最終更新**: 2025-10-06 (Phase 15.7進捗反映・名前統一)
**作成者**: Claude (全面書き直し・Phase 15系統合)

### 🎯 重要な変更点

- ✅ **Phase 15系全面追加**（15.5/15.7/15.8/15.9/15.10/15.11）
- 🔄 **現在位置更新**（Phase 12 → Phase 15.7）
- 📊 **進捗率明確化**（85-90%完成、残り10-15%）
- 🎯 **セルフホスティング予定明確化**
- 📚 **教訓・Lessons Learned追加**
- 🏗️ **Box-First原則明記**

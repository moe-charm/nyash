# 🚀 Nyash開発マスタープラン

Status: Active Development  
Last Updated: 2025-09-02  
Purpose: Claude×Copilot×ChatGPT×Gemini×Codex協調開発の総合ロードマップ

## 📍 現在位置

- 現在フェーズ: Phase 12 TypeBox統合ABI・セルフホスティング準備
- 最新成果: 🔥 **Nyash ABIをC実装TypeBoxで提供** - Rust依存排除への道！
- 次フェーズ: Phase 12.0.5 Nyash ABI C実装開始
- 備考: GeminiとCodexの深い考察により、セルフホスティングへの明確な道筋が見えました。

## 🗺️ フェーズ概要

| Phase | 状態 | 概要 | 詳細リンク |
|-------|------|------|------------|
| 8.4 | ✅完了 | AST→MIR Lowering完全実装 | [phase_8_4_ast_mir_lowering.md](phase-8/phase_8_4_ast_mir_lowering.md) |
| 8.5 | ✅完了 | MIRダイエット（35→26→15命令） | [phase_8_5_mir_35_to_26_reduction.md](phase-8/phase_8_5_mir_35_to_26_reduction.md) |
| 8.6 | 🔄進行中 | VM性能改善（0.9倍→2倍以上） | [phase_8_6_vm_performance_improvement.md](phase-8/phase_8_6_vm_performance_improvement.md) |
| 9 | 📅予定 | JIT実装 | [phase-9/](phase-9/) |
| 9.75g-0 | ✅完了 | BID-FFI Plugin System | [Phase-9.75g-0-BID-FFI-Developer-Guide.md](phase-9/Phase-9.75g-0-BID-FFI-Developer-Guide.md) |
| 9.8 | 📅予定 | BIDレジストリ + 自動コード生成 | [phase_9_8_bid_registry_and_codegen.md](phase-9/phase_9_8_bid_registry_and_codegen.md) |
| 10 | 📅予定 | Cranelift JIT（主経路） | [phase_10_cranelift_jit_backend.md](phase-10/phase_10_cranelift_jit_backend.md) |
| 11 | ✅完了 | LLVM統合・AOT実装（依存重い） | [phase-11/](phase-11/) |
| 11.8 | 📅予定 | MIR整理（Core-15→Core-13） | [phase-11.8_mir_cleanup/](phase-11.8_mir_cleanup/) |
| 12 | 🔄進行中 | TypeBox統合ABI・セルフホスティング準備 | [phase-12/](phase-12/) |
| 12.5 | 📅予定 | MIR15最適化戦略 | [phase-12.5/](phase-12.5/) |
| 12.7 | 📅予定 | AI-Nyash Compact Notation Protocol (ANCP) | [phase-12.7/](phase-12.7/) |
| 13 | 📅予定 | Nyashブラウザー革命 | [phase-13/](phase-13/) |
| 14 | 📅予定 | パッケージング・CI改善 | [phase-14/](phase-14/) |
| 15 | 🌟実現可能 | セルフホスティング（C実装ABI経由） | [phase-15/](phase-15/) |

---

## 🎯 Nyash実行モード併用戦略

### 🌟 インタープリター＋コンパイラ併用の価値

#### 実行モード使い分け
```
開発時: インタープリター（デバッグ・即時実行・非同期フル対応）
本番時: インタープリター（Pythonのように実用的）
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

## 📊 Phase別詳細

### 🚨 Phase 8.6: VM性能改善 - 最優先課題（進行中）

**Summary**:
- **緊急問題**: VMがインタープリターより0.9倍遅い（性能回帰！）
- **目標**: 2倍以上高速化でVM実行を実用レベルに引き上げ
- **担当**: Copilot主導（GitHub Issue #112）

**技術的課題**:
```bash
# 現状のベンチマーク結果
Interpreter: 110.10ms (ベースライン)
VM:          119.80ms (0.9倍 - 遅い...)
Target:       55.00ms (2倍高速化目標)
```

**推定原因と対策**:
- **デバッグ出力過多**: `println!`による性能劣化
- **HashMap操作重い**: ValueId → VM値の変換コスト
- **命令ディスパッチ非効率**: switch文ベースディスパッチ

---

### 🎊 Phase 9.75g-0: BID-FFI Plugin System - 完全完了！ ✅

**革命的成果**: NyashがプラグインでBox型を動的拡張可能に！

```nyash
// これが現実になった！
local file = new FileBox()        // プラグイン提供
local db = new PostgreSQLBox()    // 将来: プラグイン提供  
local gpu = new CudaBox()         // 将来: プラグイン提供
```

**References**:
- [Phase-9.75g-0-BID-FFI-Developer-Guide.md](phase-9/Phase-9.75g-0-BID-FFI-Developer-Guide.md)
- tools/plugin-tester/ (プラグイン診断ツール)

---

### 📦 Phase 9.8: BIDレジストリ + 自動コード生成ツール

**Summary**:
- Phase 9.75g-0完了により準備完了
- BID→各ターゲットのスタブ生成自動化

**革命的価値**:
```bash
# 🎯 1つのプラグインが4バックエンド全対応！
nyash bid gen --target wasm   bid.yaml  # WASM用import生成
nyash bid gen --target vm     bid.yaml  # VM用関数テーブル生成  
nyash bid gen --target llvm   bid.yaml  # AOT用declare生成（LLVM実装時）
```

---

### 🏆 Phase 10: Cranelift JIT（主経路）

**Summary**:
- MIR→VMを維持しつつ、ホットパスをCraneliftでJIT化
- 目標: VM比2倍以上の高速化
- LLVM AOTは設計資産は維持しつつ、Phase 11以降に検討
- **🌟 NEW: GC切り替え可能ランタイム（世界初の柔軟なメモリ管理）**

**Start Gate（着手前の必須完了）**:
- ✅ MIRダイエット（15命令）整合完了
- ✅ VM統計: `--vm-stats` でホット関数抽出可能
- 🔄 Proof of Concept: MIR→CLIFの最小Lower
- ❓ BoxCall/Array/MapのJIT最適化

**実装ステップ**:
1. **Phase 10.1**: Proof of Concept（2週間）
2. **Phase 10.2**: 基本実装（4週間）
3. **Phase 10.3**: 非同期の扱い（最小）
4. **Phase 10.4**: GC切り替え可能ランタイム（2-3ヶ月）
5. **Phase 10.5**: セルフホスティング（並行実装）

---

### 🔧 Phase 11: LLVM統合・AOT実装（完了 - 依存重い）

**Summary**:
- ✅ LLVM IRへの変換実装完了
- ✅ AOT（Ahead-of-Time）コンパイル動作確認
- ✅ ネイティブ実行ファイル生成成功

**得られた知見**:
- **依存関係が重い**: LLVM自体のビルド時間・サイズが巨大
- **動作は確認**: 技術的には成功、実用性に課題
- **Cranelift回帰**: 軽量な代替として再評価

---

### 📐 Phase 11.8: MIR整理（Core-15→Core-13）

**Summary**:
- ArrayGet/ArraySet → BoxCall統合
- PluginInvoke → BoxCall統合  
- 最終的にCore-13を目指す

**詳細**: [phase-11.8_mir_cleanup/](phase-11.8_mir_cleanup/)

---

### 🎯 Phase 12: TypeBox統合ABI・セルフホスティング準備（進行中）

**Summary**:
- TypeBox革命：型情報もBoxとして扱う統一設計
- C ABI + Nyash ABI完全統合
- 🔥 **Nyash ABIのC実装**でRust依存排除！

**革命的成果**:
1. TypeBox：プラグイン間Box生成を可能に
2. 統合ABI：C/Nyash ABIをシームレス統合
3. **セルフホスティング**：C実装ABIで実現可能！

**AI専門家の評価**:
- Gemini：「技術的妥当性が高く、哲学とも合致した極めて優れた設計」
- Codex：「16バイトアライメント、セレクターキャッシング等の具体案」

---

### ⚡ Phase 12.5: MIR15最適化戦略 - コンパイラ丸投げ作戦

**Summary**:
- 「CPU（コンパイラ）に丸投げできるところは丸投げ」
- MIR15の美しさ（15命令）を保ちながら実用的性能達成
- 自前最適化は最小限、成熟したコンパイラ技術を活用

**最適化境界線**:
- **MIR側**: カノニカル化・軽量最適化のみ
- **コンパイラ側**: ループ最適化・SIMD・レジスタ割当等

**ヒントシステム**:
- 命令は増やさずメタデータでヒント付与
- pure/readonly/noalias/likely等の属性
- Cコンパイラ/Cranelift/LLVMへ機械的マップ

**詳細**: [phase-12.5/](phase-12.5/)

---

## 🧠 AI大会議から得られた技術的知見

### Gemini先生の助言
- ✅ エスケープ解析・ボックス化解除が性能の鍵  
- ✅ wasmtime compileは短期的に実用的
- ✅ WASM実行は確実に高速（13.5倍実証済み）
- 🔄 Cranelift → LLVM段階的アプローチ

### codex先生の助言
- ✅ MIR前倒し実装推奨（全バックエンドが恩恵）
- ✅ wasmtime互換性管理が重要
- ✅ CPU差異対応 (baseline/v3二段ビルド)
- ✅ 起動時間・割当削減・配布体験がKPI

### Claude統合分析
- ✅ 実用価値最大化: WASM+AOTで十分な競争力
- ✅ 開発効率: Cranelift JITの恩恵限定的
- ✅ Everything is Box最適化が差別化の核心
- ✅ 時間効率: 2-3ヶ月節約でLLVM集中投資

---

## 💡 協調開発への具体的お願い

### 🔧 Phase 8.6 VM性能改善（最優先）
- ❓ 命令ディスパッチのボトルネック特定方法は？
- ❓ HashMap操作の最適化戦略は？  
- ❓ デバッグ出力削除による性能改善測定は？
- ❓ Direct threading実装の現実的アプローチは？

### 🚀 長期戦略相談
- ❓ インタープリターとコンパイラの互換性保証は？
- ❓ MIR→LLVM IR変換の基本的な実装戦略は？
- ❓ Box型のLLVM表現として最適なアプローチは？
- ❓ エスケープ解析によるスタック化判定は？

---

## 🌟 Phase 15: セルフホスティング（実現可能！）

**革命的発見**: Nyash ABIをC実装TypeBoxで提供することで、Rust依存を排除！

### 実現への道筋（明確化）
1. **Phase 12.0.5**: Nyash ABI C Shim実装（Rust FFI経由）
2. **Phase 13**: C実装の完全化（基本型・参照カウント）
3. **Phase 14**: NyashでABI再実装（AOTでC ABI公開）
4. **Phase 15**: Nyashコンパイラ自身をNyashで実装！

### 技術的革新
- **TypeBox哲学**: ABIすらBoxとして扱う究極の統一
- **C ABI基盤**: 最も安定した普遍的インターフェース
- **段階的移行**: 既存Rust実装との共存期間を確保

---

## 📊 進捗管理・コミュニケーション

### 🤝 協調開発ルール
- ✅ 大きな変更前にはdocs/CURRENT_TASK.mdで情報共有
- ✅ ベンチマーク機能は最優先で維持
- ✅ 競合発生時は機能優先度で解決
- ✅ AI専門家（Gemini/Codex）の深い考察を活用

### 品質保証
- ✅ cargo check でビルドエラーなし
- ✅ 既存ベンチマークが regression なし
- ✅ 新機能のドキュメント整備
- ✅ テストケース追加・CI通過

---

## 🎯 期待される成果

### 達成済み
- 🏆 RefNew/RefGet/RefSet WASM完全動作
- 🏆 MIR命令削減完了（35→26→15命令、Phase 8.5）
- 🏆 Phase 9.75g-0 BID-FFI Plugin System完全完了
- 🏆 警告削減100%達成（Phase 9.75j）

### 進行中・予定
- 🔄 VM性能改善進行中（Phase 8.6）- GitHub Issue #112
- 📅 Cranelift JIT（Phase 10）: VM比2×以上の高速化
- 📅 非同期ネイティブ実装: async/await完全対応
- 📅 インタープリター併用: 開発・本番両対応

---

## 📞 連絡・相談方法

技術的相談や進捗報告は、以下の方法でお気軽にどうぞ：

1. 📝 GitHub Issues・Pull Request
2. 📋 docs/CURRENT_TASK.md コメント
3. 🤖 AI大会議 (重要な技術決定)
4. 💬 コミットメッセージでの進捗共有

どんな小さなことでも相談大歓迎です！
一緒にNyashを最高の言語にしていきましょう🚀

---

**最終更新**: 2025-08-26 (copilot_issues.txt統合・Markdown化)  
**作成者**: Claude (ファイル統合・構造整理)

### 🎯 重要な変更点
- ✅ **Phase 9.75g-0 BID-FFI Plugin System完全完了**
- 🔄 **Phase 8.6 VM性能改善を最優先** (進行中)
- 📦 **Phase 9.8 BIDレジストリ** (Phase 8.6完了後の次期重点)
- 🔍 **Phase 10 Cranelift JIT** (主経路として確定)
- 🌟 **統一ロードマップ化** (phasesフォルダに集約)
## 🌈 Phase 22構想 - Nyash LLVM Compiler (将来)
- LLVMコンパイラ自体をNyashで実装
- C++薄ラッパー(20-30関数) + Nyash実装(100-200行)
- ビルド時間: 5-7分 → 即時反映
- 詳細: [Phase 22 README](phase-22/README.md)

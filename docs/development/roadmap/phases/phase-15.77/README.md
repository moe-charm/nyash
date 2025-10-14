# Phase 15.77 - 凍結EXE確定フェーズ

**期間**: 2025-11-09 - 2025-12-20（6週間）
**状態**: 🔜 Phase 15.76完了後開始

---

## 🎯 このフェーズで実現すること

**凍結EXE（hako-frozen-v1.exe）確定 + Rust層最小化**

1. **凍結EXE作成**: hako-frozen-v1.exe ビルド・タグ付け・配布
2. **Rust層最小化**: 99,406行 → 100-200行（VM実行エンジンのみ）
3. **単一パーサ体制**: Hakoruneパーサーのみ開発（Rust凍結）
4. **安全な試行錯誤**: いつでも凍結EXEに戻れる基盤確立

---

## 💡 このフェーズの位置づけ

### Phase 15.76で確立した「背骨」を使って実際に凍結

```
Phase 15.76（背骨確立）
├── extern_c構文実装 ✅
├── LLVM Backendプラグイン化 ✅
└── AOT導線整備（MIR JSON→.o→EXE）✅

Phase 15.77（凍結実行）← 今ここ
├── 凍結EXE作成・配布
├── Rust層最小化実行
└── 単一パーサ体制確立

Phase 15.78〜（脱Rust加速）
└── Hakoruneパーサー完全実装
```

---

## 🎯 成功基準（DoD）

### 1️⃣ 凍結EXE確定

#### ビルド・動作確認
- [ ] `hako-frozen-v1.exe` ビルド成功（Linux/macOS/Windows）
- [ ] 170 PASS維持（quick-selfhost）
- [ ] AOT導線動作確認（MIR JSON→.o→EXE）

#### タグ付け・配布
- [ ] Git tag作成（`v1.0.0-frozen`）
- [ ] 配布物生成（`hako-frozen-v1-linux-x64.tar.gz`等）
- [ ] README/LICENSE同梱
- [ ] SHA256チェックサム生成

#### 配布先
```
releases/
├── v1.0.0-frozen/
│   ├── hako-frozen-v1-linux-x64.tar.gz
│   ├── hako-frozen-v1-macos-arm64.tar.gz
│   ├── hako-frozen-v1-windows-x64.zip
│   ├── SHA256SUMS.txt
│   └── INSTALL.md
```

### 2️⃣ Rust層最小化

#### 目標
```
現状: 99,406行（Phase 15.75開始時）
↓
削減: 89.5%
↓
目標: 100-200行（VM実行エンジンのみ）
```

#### 段階的削減計画
```rust
// Week 1-2: Parser削除（凍結EXE内のみ残す）
src/front/parser_layer/    ❌ 削除（凍結EXE経由）
src/front/ast/             ❌ 削除（MIR直接生成へ）

// Week 3-4: MIR Builder削除（Hakorune実装へ）
src/backend/mir_builder/   ❌ 削除（Hakorune実装使用）

// 残すもの（100-200行）
src/backend/mir_interpreter/  ✅ 残す（VM実行エンジン）
src/runtime/ffi/              ✅ 残す（extern_c基盤）
src/main.rs                   ✅ 残す（エントリーポイント）
```

### 3️⃣ 単一パーサ体制確立

#### Hakoruneパーサーをデフォルトへ
- [ ] `apps/selfhost-compiler/` をデフォルト呼び出し
- [ ] Rustパーサーは凍結EXE内のみ（緊急時用）
- [ ] スモークテスト全緑（Hakoruneパーサー使用）

#### 2重メンテ回避
```
Before（Phase 15.76以前）:
├── Rustパーサー（開発中）
└── Hakoruneパーサー（開発中）← 2重メンテ地獄

After（Phase 15.77）:
├── Rustパーサー（凍結、緊急時のみ）
└── Hakoruneパーサー（開発中）← 単一開発ライン
```

### 4️⃣ ドキュメント整備

#### 使用ガイド
- [ ] `docs/guides/frozen-toolchain-usage.md` - 凍結EXE使用方法
- [ ] `docs/guides/rust-minimization.md` - Rust最小化手順
- [ ] `INSTALL.md` - 配布物インストールガイド

#### 運用マニュアル
- [ ] 凍結EXEへの戻り方
- [ ] トラブルシューティング
- [ ] FAQ（よくある質問）

---

## 📊 週次計画（Week 1-6）

### Week 1（2025-11-09 - 11-15）凍結EXE作成

**目標**: hako-frozen-v1.exe ビルド・動作確認

#### タスク
- [ ] Phase 15.76の成果物確認（extern_c/AOT導線）
- [ ] ビルド設定整備（Cargo.toml, build.rs）
- [ ] クロスコンパイル設定（Linux/macOS/Windows）
- [ ] 動作確認（quick-selfhost 170 PASS）

#### 成果物
```bash
./hako-frozen-v1 --version
# Hakorune v1.0.0-frozen (2025-11-15)

./hako-frozen-v1 program.hako
# OK (VM実行)

./hako-frozen-v1 --backend llvm program.hako
# OK (LLVM AOT)
```

### Week 2（2025-11-16 - 11-22）タグ付け・配布物生成

**目標**: Git tag作成・配布物公開

#### タスク
- [ ] Git tag作成（`v1.0.0-frozen`）
- [ ] 配布物生成スクリプト作成
- [ ] SHA256チェックサム生成
- [ ] INSTALL.md作成

#### 成果物
```
releases/v1.0.0-frozen/
├── hako-frozen-v1-linux-x64.tar.gz    (12MB)
├── hako-frozen-v1-macos-arm64.tar.gz  (10MB)
├── hako-frozen-v1-windows-x64.zip     (15MB)
├── SHA256SUMS.txt
└── INSTALL.md
```

### Week 3（2025-11-23 - 11-29）Rust Parser削除

**目標**: Parser削除、凍結EXE経由に切り替え

#### タスク
- [ ] `src/front/parser_layer/` 削除準備
- [ ] 凍結EXE経由のParser呼び出し実装
- [ ] AST→MIR直接生成経路確立
- [ ] スモークテスト修正（Parser経路変更）

#### 削減予想
```
Before: 99,406行
After:  ~50,000行（Parser削除、約50%削減）
```

### Week 4（2025-11-30 - 12-06）MIR Builder削除準備

**目標**: MIR Builder Hakorune実装へ移行開始

#### タスク
- [ ] `apps/selfhost-compiler/mir_builder/` 検証
- [ ] Rust MIR Builder削除可能性確認
- [ ] ブリッジ層実装（Hakorune→Rust）
- [ ] 並行動作確認（Rust/Hakorune両方）

#### 削減予想
```
Before: ~50,000行
After:  ~5,000行（MIR Builder削除、約90%削減）
```

### Week 5（2025-12-07 - 12-13）Hakoruneパーサー移行

**目標**: Hakoruneパーサーをデフォルトへ

#### タスク
- [ ] `apps/selfhost-compiler/` をデフォルト呼び出し
- [ ] Rustパーサーを緊急時用に格下げ
- [ ] スモークテスト全緑確認（Hakorune使用）
- [ ] パフォーマンス計測（Rust vs Hakorune）

#### 期待結果
```bash
# デフォルト: Hakoruneパーサー
./hako program.hako  # Hakorune経由

# 緊急時: Rustパーサー（凍結EXE内）
./hako --fallback-rust-parser program.hako  # Rust経由
```

### Week 6（2025-12-14 - 12-20）ドキュメント・レビュー

**目標**: ドキュメント整備・統合テスト

#### タスク
- [ ] 凍結EXE使用ガイド作成
- [ ] Rust最小化手順書作成
- [ ] トラブルシューティングFAQ作成
- [ ] 統合テスト（quick-selfhost 全緑）
- [ ] ChatGPT/Claudeレビュー

#### 成果物
```
docs/guides/
├── frozen-toolchain-usage.md     # 使用方法
├── rust-minimization.md          # 最小化手順
└── frozen-toolchain-faq.md       # FAQ

docs/development/roadmap/phases/phase-15.77/
├── README.md                     # Phase概要
├── MILESTONE.md                  # ゴールライン
├── RUST_MINIMIZATION_PLAN.md     # 最小化計画
└── COMPLETION_REPORT.md          # 完了報告
```

---

## ❌ Out of Scope（このフェーズではやらない）

### 完全なRust削除
- VM実行エンジンは残す（100-200行）
- 理由: VM安定性・パフォーマンス

### Hakoruneパーサー完全実装
- 段階的移行（並行動作期間あり）
- 理由: 安全な移行を優先

### プロダクション配布
- まだ実験段階（v1.0.0-frozen）
- 理由: Phase 15.78以降で安定化

### Windows完全サポート
- Linux/macOS優先
- 理由: 開発環境がLinux/WSL中心

---

## 🚀 次のフェーズ（Phase 15.78〜）

### Phase 15.78: Hakoruneパーサー完全実装
- 全構文サポート（macro/async/await等）
- Rustパーサーとの完全パリティ
- パフォーマンス最適化

### Phase 15.79: 完全な脱Rust
- VM実行エンジンもHakorune実装へ
- Rust層完全削除（0行）
- Pure Hakorune達成

### Phase 15.80: プロダクション化
- v2.0.0リリース
- 安定性・セキュリティ監査
- 公式配布開始

---

## ⚠️ リスク & 対策

### リスク1: 凍結EXEのサイズ
**問題**: 静的リンクで15MB超の可能性
**対策**: 動的リンク検討、圧縮配布

### リスク2: クロスコンパイル失敗
**問題**: Windows/macOSビルドが難しい
**対策**: Linux優先、他環境は次フェーズ

### リスク3: Rust削除時の不安定化
**問題**: 削除作業でテスト失敗増加
**対策**: 段階的削除、各段階でテスト確認

### リスク4: Hakoruneパーサーの未熟
**問題**: Rustパーサーと完全パリティ未達
**対策**: 並行動作期間を長く取る、Rustフォールバック維持

---

## 📚 関連リソース

### 前フェーズ
- [Phase 15.76 - extern_c & Frozen Toolchain](../phase-15.76/)
- [Phase 15.75 - 脱Rust大作戦](../phase-15.75/)

### 論文資料
- [Rapid Self-Hosting Paper](../../../../private/papers-active/rapid-selfhost-ai-collaboration/)
- [Frozen Toolchain Pattern Evidence](../../../../private/papers-active/rapid-selfhost-ai-collaboration/03_DATA_ANALYSIS.md)

### 業界標準パターン
- **Rust**: stage0（凍結ツールチェーン）
- **Go**: Go 1.4 frozen（bootstrap用）
- **OCaml**: ocamlc frozen（自己ホスト用）

---

## 💬 開発体制

### 実装担当
- **ChatGPT**: Rust削除・Hakorune実装主導
- **Claude**: レビュー・ドキュメント整備
- **tomoaki**: 戦略判断・方向決定

### レビュー方針
- 各Week終了時にレビュー
- 170 PASS維持を最優先
- 問題発生時は即座に凍結EXEへロールバック

---

**作成日**: 2025-10-14
**Phase開始予定**: 2025-11-09（Phase 15.76完了後）
**想定期間**: 6週間

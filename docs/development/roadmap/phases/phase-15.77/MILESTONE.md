# Phase 15.77 Milestone & Goal Line

## 🎯 このフェーズのゴール

**凍結EXE（hako-frozen-v1.exe）確定 + Rust層を100-200行まで最小化**

Phase 15.76で確立した「extern_c + .o生成」の背骨を使って、実際に凍結EXEを作成し、Rust層を最小化する。

---

## ✅ Goal Line（受け入れ基準/DoD）

### 1️⃣ 凍結EXE確定

#### ビルド・動作確認
- [ ] `hako-frozen-v1` ビルド成功（Linux必須、macOS/Windows推奨）
- [ ] 170 PASS維持（quick-selfhost プロファイル）
- [ ] AOT導線動作（MIR JSON→.o→EXE）
- [ ] extern_c動作確認（getpid/strlen/system + LLVM Backend）

#### バージョン情報
```bash
./hako-frozen-v1 --version
# Output:
# Hakorune v1.0.0-frozen
# Build Date: 2025-11-15
# Git Commit: abc123def456
# Features: extern_c, llvm_backend, frozen_parser
```

#### Git Tag
- [ ] Tag作成: `v1.0.0-frozen`
- [ ] Tag message: "Phase 15.77: Frozen toolchain v1.0.0"
- [ ] Push to remote: `git push origin v1.0.0-frozen`

#### 配布物生成
```
releases/v1.0.0-frozen/
├── hako-frozen-v1-linux-x64.tar.gz      (12-15MB)
│   ├── hako-frozen-v1                   (実行ファイル)
│   ├── README.md                        (概要)
│   ├── LICENSE                          (ライセンス)
│   └── INSTALL.md                       (インストール手順)
├── hako-frozen-v1-macos-arm64.tar.gz    (10-12MB)  ← 推奨
├── hako-frozen-v1-windows-x64.zip       (15-18MB)  ← 推奨
└── SHA256SUMS.txt                       (チェックサム)
```

#### チェックサム検証
```bash
# SHA256チェックサム生成
sha256sum hako-frozen-v1-*.tar.gz > SHA256SUMS.txt

# 検証コマンド
sha256sum -c SHA256SUMS.txt
# Output:
# hako-frozen-v1-linux-x64.tar.gz: OK
```

### 2️⃣ Rust層最小化

#### 削減計画
```
現状（Phase 15.75開始時）: 99,406行
↓
Phase 15.77目標: 100-200行（99.8%削減）

削減内訳:
- Parser削除       : ~30,000行削減
- AST削除          : ~10,000行削除
- MIR Builder削除  : ~40,000行削除
- 雑多なコード削除 : ~19,200行削減
- 残す（VM+FFI）   : ~200行
```

#### 段階的削減チェックリスト

**Week 3: Parser削除**
- [ ] `src/front/parser_layer/` 削除
- [ ] `src/front/ast/` 削除
- [ ] 凍結EXE経由のParser呼び出し実装
- [ ] スモークテスト修正（Parser経路変更）
- [ ] 削減確認: ~40,000行削減 → 残り~59,000行

**Week 4: MIR Builder削除準備**
- [ ] `apps/selfhost-compiler/mir_builder/` 検証完了
- [ ] Rust MIR Builder削除可能性確認
- [ ] ブリッジ層実装（Hakorune→Rust）
- [ ] 並行動作確認（Rust/Hakorune両方動作）
- [ ] 削減準備: ~40,000行削減予定 → 残り~19,000行予定

**Week 5: MIR Builder削除実行**
- [ ] `src/backend/mir_builder/` 削除
- [ ] Hakorune MIR Builderをデフォルトへ
- [ ] スモークテスト全緑確認
- [ ] 削減確認: ~40,000行削減 → 残り~19,000行

**Week 6: 雑多なコード削除**
- [ ] 未使用依存削除（Cargo.toml整理）
- [ ] デッドコード削除
- [ ] 最終調整（~200行に収束）
- [ ] 削減確認: ~18,800行削減 → 残り~200行

#### 最終構成（目標）
```rust
// src/main.rs (~50行)
fn main() {
    // 凍結Parser呼び出し → MIR取得
    let mir = frozen_parse_to_mir(source);
    // VM実行
    vm::execute(mir);
}

// src/backend/mir_interpreter/ (~100行)
mod vm {
    pub fn execute(mir: Mir) -> Result<Value> {
        // 最小VM実行ループ
    }
}

// src/runtime/ffi/ (~50行)
mod ffi {
    pub fn call_extern(symbol: &str, args: Vec<Value>) -> Value {
        // extern_c実行
    }
}

// 合計: ~200行
```

### 3️⃣ 単一パーサ体制確立

#### Hakoruneパーサーをデフォルトへ
- [ ] `apps/selfhost-compiler/` をデフォルト呼び出し
- [ ] Rustパーサーは凍結EXE内のみ（緊急時用）
- [ ] 環境変数 `HAKO_USE_FROZEN_PARSER=1` で切替可能

#### 動作モード
```bash
# デフォルト: Hakoruneパーサー
./hako program.hako
# 内部: apps/selfhost-compiler/ 経由

# 緊急時: 凍結Rustパーサー
HAKO_USE_FROZEN_PARSER=1 ./hako program.hako
# 内部: hako-frozen-v1 経由
```

#### 2重メンテ回避確認
- [ ] Rustパーサーのコード変更停止（凍結確定）
- [ ] Hakoruneパーサーのみ開発継続
- [ ] スモークテスト全緑（Hakorune使用）
- [ ] ドキュメント更新（Rust凍結の明記）

### 4️⃣ ドキュメント整備

#### 使用ガイド
- [ ] `docs/guides/frozen-toolchain-usage.md`
  - 凍結EXEのインストール方法
  - 基本的な使用方法
  - トラブルシューティング
- [ ] `docs/guides/rust-minimization.md`
  - Rust最小化の手順
  - 各週の作業内容
  - ロールバック方法

#### 配布物ドキュメント
- [ ] `INSTALL.md` - 配布物同梱
  - システム要件
  - インストール手順
  - 動作確認方法
- [ ] `README.md` - 配布物同梱
  - Hakorune概要
  - 凍結EXEの説明
  - リンク集

#### Phase完了報告
- [ ] `docs/development/roadmap/phases/phase-15.77/COMPLETION_REPORT.md`
  - 達成内容
  - 削減行数の実績
  - 残された課題
  - 次フェーズへの引き継ぎ

---

## ❌ Out of Scope（このフェーズではやらない）

### VM実行エンジンの削除
- 理由: VM安定性・パフォーマンス維持
- 次フェーズ: Phase 15.79で検討

### Hakoruneパーサーの完全実装
- 理由: 段階的移行を優先（安全性）
- 次フェーズ: Phase 15.78で完全実装

### プロダクション配布
- 理由: まだ実験段階（v1.0.0-frozen）
- 次フェーズ: Phase 15.80でプロダクション化

### Windows完全サポート
- 理由: 開発環境がLinux/WSL中心
- 対応: Linux優先、Windows は推奨レベル

---

## 🚀 次フェーズ開始条件（Phase 15.78へ）

### 必須条件
1. **凍結EXE安定動作**: 170 PASS維持、AOT導線動作
2. **Rust層100-200行達成**: 削減確認、VM+FFIのみ残存
3. **単一パーサ体制確立**: Hakoruneデフォルト、Rust凍結

### 推奨条件
4. 配布物公開完了（Linux必須）
5. ドキュメント整備完了
6. ChatGPT/Claudeレビュー完了

---

## 📊 進捗管理（Week 1-6）

### Week 1（2025-11-09 - 11-15）凍結EXE作成
- [ ] Phase 15.76成果物確認
- [ ] ビルド設定整備
- [ ] クロスコンパイル設定
- [ ] 動作確認（170 PASS）

**マイルストーン**: hako-frozen-v1 初回ビルド成功

### Week 2（2025-11-16 - 11-22）タグ付け・配布
- [ ] Git tag作成（v1.0.0-frozen）
- [ ] 配布物生成スクリプト
- [ ] SHA256チェックサム
- [ ] INSTALL.md作成

**マイルストーン**: 配布物公開準備完了

### Week 3（2025-11-23 - 11-29）Parser削除
- [ ] Parser削除準備
- [ ] 凍結EXE経由実装
- [ ] スモークテスト修正
- [ ] 削減確認（~40,000行）

**マイルストーン**: Parser削除完了、~59,000行

### Week 4（2025-11-30 - 12-06）MIR Builder削除準備
- [ ] Hakorune MIR Builder検証
- [ ] 削除可能性確認
- [ ] ブリッジ層実装
- [ ] 並行動作確認

**マイルストーン**: MIR Builder削除準備完了

### Week 5（2025-12-07 - 12-13）Hakoruneパーサー移行
- [ ] Hakoruneデフォルト化
- [ ] Rustパーサー格下げ
- [ ] スモークテスト全緑
- [ ] パフォーマンス計測

**マイルストーン**: 単一パーサ体制確立

### Week 6（2025-12-14 - 12-20）ドキュメント・レビュー
- [ ] 使用ガイド作成
- [ ] 最小化手順書作成
- [ ] FAQ作成
- [ ] 統合テスト
- [ ] レビュー完了

**マイルストーン**: Phase 15.77完了

---

## ⚠️ リスク & 対策

### リスク1: 凍結EXEサイズ
**問題**: 静的リンクで15MB超
**影響**: 配布・ダウンロード時間増加
**対策**:
- 動的リンク検討
- UPX圧縮（Linux/macOS）
- 優先度: 低（機能優先）

### リスク2: クロスコンパイル失敗
**問題**: Windows/macOSビルド困難
**影響**: 配布物不足
**対策**:
- Linux優先配布
- CI/CDで自動ビルド
- 優先度: 中（Linux必須、他推奨）

### リスク3: Rust削除時の不安定化
**問題**: 削除でテスト失敗増加
**影響**: 170 PASS維持困難
**対策**:
- 段階的削除（週次確認）
- 各段階でテスト実行
- ロールバック手順明確化
- 優先度: 高（最重要）

### リスク4: Hakoruneパーサー未熟
**問題**: Rustパリティ未達
**影響**: 単一パーサ体制移行困難
**対策**:
- 並行動作期間延長
- Rustフォールバック維持
- 優先度: 高（重要）

---

## 🎯 成功の定義

### ミニマム（最低限）
1. ✅ hako-frozen-v1 ビルド成功（Linux）
2. ✅ 170 PASS維持
3. ✅ Rust層500行以下

### ターゲット（目標）
4. ✅ 配布物公開（Linux + macOS/Windows推奨）
5. ✅ Rust層100-200行達成
6. ✅ 単一パーサ体制確立

### ストレッチ（理想）
7. ✅ 全プラットフォーム配布（Linux/macOS/Windows）
8. ✅ Rust層100行以下
9. ✅ プロダクション配布準備完了

---

## 📈 削減行数トラッキング

### 進捗表
| Week | 作業内容 | 削減行数 | 残り行数 | 削減率 |
|------|---------|---------|---------|--------|
| 開始 | Phase 15.75基準 | - | 99,406 | 0% |
| Week 3 | Parser削除 | ~40,000 | ~59,000 | 40.2% |
| Week 4 | MIR Builder準備 | - | ~59,000 | 40.2% |
| Week 5 | MIR Builder削除 | ~40,000 | ~19,000 | 80.9% |
| Week 6 | 雑多削除 | ~18,800 | ~200 | 99.8% |

### グラフ（イメージ）
```
100,000 ┤
 90,000 ┤
 80,000 ┤
 70,000 ┤
 60,000 ┤           ╭──Week 4
 50,000 ┤           │
 40,000 ┤           │
 30,000 ┤           │
 20,000 ┤      Week 3│
 10,000 ┤           ╰──Week 5
      0 ┤                 ╰──Week 6 (200行)
        └─────────────────────────→
        開始  W3   W4   W5   W6
```

---

**作成日**: 2025-10-14
**Phase開始予定**: 2025-11-09
**想定期間**: 6週間（2025-11-09 - 2025-12-20）
**関連**: Phase 15.76完了後、Phase 15.78へ

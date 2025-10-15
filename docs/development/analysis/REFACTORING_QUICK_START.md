# リファクタリング クイックスタート

**最終更新**: 2025-10-15
**参照**: [統合リファクタリングロードマップ](INTEGRATED_REFACTORING_ROADMAP.md)

---

## 🚀 今すぐ実行可能（30分 → 2,383行削減）

### Step 1: バックアップファイル削除（5分）

```bash
cd /home/tomoaki/git/hakorune-selfhost

# 削除
rm src/backend/mir_interpreter/handlers/calls/method.rs.bak.1760057047

# 確認
git status
```

**削減**: 327行

---

### Step 2: BID Codegen実験コード削除（15分）

```bash
# README確認（念のため）
cat src/bid-codegen-from-copilot/README.md
cat src/bid-converter-copilot/README.md

# 削除
rm -rf src/bid-codegen-from-copilot
rm -rf src/bid-converter-copilot

# Cargo.toml参照確認
grep -r "bid-codegen\|bid-converter" Cargo.toml
# → 出力なしならOK
```

**削減**: 1,894行

---

### Step 3: Plugin Legacy Proxy削除（10分）

```bash
# 参照確認（再確認）
grep -r "plugin_box_legacy" src --include="*.rs"
# → 出力なしならOK

# 削除
rm src/runtime/plugin_box_legacy.rs

# mod.rsから参照削除
# src/runtime/mod.rs 内の以下の行をコメントアウトまたは削除:
# pub mod plugin_box_legacy;
```

**削減**: 158行

---

### Step 4: 未使用警告修正（15分）

#### 4.1 type_registry.rs

```rust
// src/runtime/type_registry.rs:92
// 以下の行を削除:
#[allow(dead_code)]
```

#### 4.2 dispatch.rs

```rust
// src/runner/dispatch.rs:349
// 以下の行を削除:
use std::io::Write;
```

#### 4.3 ffi_bridge.rs

```rust
// src/runtime/plugin_loader_v2/enabled/ffi_bridge.rs:419
// box_type変数を削除または使用
```

#### 4.4 mir_json_emit.rs

```rust
// src/runner/mir_json_emit.rs:205
// entry_id_u32変数を削除または使用
```

**削減**: 4行（警告ゼロ化）

---

### Step 5: ビルド＆テスト（30分）

```bash
# ビルド
cargo build --release

# Rustテスト
cargo test

# スモークテスト
tools/smokes/v2/run.sh --profile quick
```

**期待結果**:
- ✅ ビルド成功
- ✅ cargo test 全PASS
- ✅ スモークテスト 170+ PASS

---

### Step 6: コミット（5分）

```bash
git add -A
git commit -m "refactor(phase1): Quick Wins完了 - 即座削除可能ファイル一掃

- ✅ バックアップファイル削除: 327行
- ✅ BID Codegen実験コード削除: 1,894行
- ✅ Plugin Legacy Proxy削除: 158行
- ✅ 未使用警告修正: 4行
- ✅ 削減合計: 2,383行（2.4%）
- ✅ 全テストPASS

Phase 1完了。Phase 2（構造改善）へ。

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
"

git push
```

---

## ⏳ 次のステップ（Phase 2 - 4週間後）

### 前提条件
- ✅ Phase 20.5完了（Hakorune VM検証・統合）
- ✅ Plugin安定性確認（1週間連続テスト）

### 実施内容
1. **Legacy VM handlers削除**: 1,145行
2. **src/boxes/削除**: 3,000行
3. **MIR Builder legacy削除**: 52行

**削減合計**: 4,197行（4.2%）

---

## 📋 チェックリスト

### Phase 1（今日実行）

- [ ] Step 1: バックアップファイル削除
- [ ] Step 2: BID Codegen削除
- [ ] Step 3: Plugin Legacy Proxy削除
- [ ] Step 4: 警告修正
- [ ] Step 5: ビルド＆テスト
- [ ] Step 6: コミット＆プッシュ

### Phase 2（Phase 20.5後）

- [ ] Phase 20.5完了確認
- [ ] Plugin安定性確認（1週間）
- [ ] Legacy handlers削除
- [ ] src/boxes/削除
- [ ] テスト＆コミット

### Phase 3（Phase 2後）

- [ ] Selfhost compiler整理
- [ ] Backend統合判断
- [ ] ドキュメント完備

---

## 🎯 成果予測

| Phase | 削減行数 | 削減率 | 所要時間 |
|-------|---------|--------|---------|
| Phase 1 | 2,383 | 2.4% | **1-2時間** ⚡今すぐ |
| Phase 2 | 4,197 | 4.2% | 4週間 |
| Phase 3 | 1,245 | 1.2% | 6週間 |
| **合計** | **7,825** | **7.9%** | **11週間** |

---

## ⚠️ 注意事項

1. **Step毎にテスト**: 各削除後に必ずビルド確認
2. **Git履歴保持**: 削除前にcommitして復元可能に
3. **段階的実行**: 一度に全部やらない
4. **Fail-Fast**: エラーが出たら即座に報告

---

## 💡 トラブルシューティング

### ビルドエラーが出た場合

```bash
# 最後の正常なコミットに戻る
git log --oneline -5
git revert <commit-hash>

# または
git reset --hard HEAD~1
```

### テスト失敗が出た場合

```bash
# 詳細ログ確認
NYASH_CLI_VERBOSE=1 tools/smokes/v2/run.sh --profile quick

# 特定のテストのみ実行
tools/smokes/v2/run.sh --profile quick --filter "test_name"
```

---

## 📚 関連ドキュメント

- **詳細計画**: [統合リファクタリングロードマップ](INTEGRATED_REFACTORING_ROADMAP.md)
- **Phase 20.5**: [README](../roadmap/phases/phase-20.5/README.md)
- **Legacy検出**: [Legacy Code Detection Report](legacy-code-detection-report.md)

---

**🚀 準備OK！今すぐStep 1から始めましょう！**

# Branching Strategy — wasm-development 専用ブランチ化

## 🌿 ブランチ構成（Phase 15: 2本柱体制）

### **wasm-development** ← **このブランチ**
- **目的**: LLVM→WASM実装（Phase 15.8）
- **開始**: 2025-10-01（selfhostからfork）
- **範囲**: `src/llvm_py/` のWASM拡張実装
- **タスク**: `CURRENT_TASK_WASM.md`

### **selfhost**（メインブランチ）
- **目的**: セルフホスティング実装（Phase 15.7 Pipeline v2）
- **範囲**: `apps/selfhost-compiler/` の実装
- **タスク**: `CURRENT_TASK.md`（このブランチには存在しない）

---

## 🚨 重要原則: 独立開発・選択的統合

### ✅ DO（推奨）
1. **独立開発**: 各ブランチで独自に実装を進める
2. **CURRENT_TASK分離**: `CURRENT_TASK_WASM.md` vs `CURRENT_TASK.md`
3. **選択的cherry-pick**: 必要な変更のみ取り込む
4. **コミット単位**: 小さく論理的に分割

### ❌ DON'T（禁止）
1. **git merge selfhost**: 自動マージ禁止（コンフリクト多発）
2. **CURRENT_TASK.md共有**: タスク管理の混乱を招く
3. **大量cherry-pick**: 必要な変更のみ厳選
4. **ブランチ間違い**: 必ず `git branch --show-current` で確認

---

## 🔄 作業フロー

### wasm-development での作業
```bash
# 1. ブランチ確認（必須！）
git branch --show-current  # → wasm-development

# 2. 開発作業
vim src/llvm_py/...

# 3. コミット＆プッシュ
git add src/llvm_py/
git commit -m "llvm_py(wasm): ..."
git push private wasm-development  # 'private' remote使用
```

### selfhost からの変更取り込み（必要時のみ）
```bash
# 1. selfhostの変更確認
git log selfhost --oneline -10

# 2. 必要なコミットのみcherry-pick
git cherry-pick <commit-hash>

# 3. コンフリクト解決（必要な場合）
git status
vim <conflicted-file>
git add <conflicted-file>
git cherry-pick --continue
```

---

## 📁 ファイル管理

### wasm-development 専用ファイル
- `CURRENT_TASK_WASM.md` ← **このブランチのタスク管理**
- `BRANCHING.md` ← **このファイル**
- `src/llvm_py/builders/phi_handler.py`
- `src/llvm_py/builders/instruction_context.py`
- `src/llvm_py/targets/` （全ファイル）

### selfhost 専用ファイル（このブランチには存在しない）
- `CURRENT_TASK.md` ← **selfhostブランチのタスク管理**
- `apps/selfhost-compiler/pipeline_v2/` （実装詳細）

### 共通ファイル（慎重に扱う）
- `CLAUDE.md` ← **両ブランチ共通**（更新時は慎重に）
- `README.md`
- `docs/development/roadmap/phases/phase-15.8/README.md`

---

## 🛡️ コンフリクト回避戦略

### 原則
1. **CURRENT_TASK分離**: 絶対に共有しない
2. **ディレクトリ分離**: `src/llvm_py/` vs `apps/selfhost-compiler/`
3. **実装タイミング**: 同時編集を避ける

### コンフリクト発生時
```bash
# 1. 状態確認
git status

# 2. 差分確認
git diff HEAD

# 3. 手動解決
vim <conflicted-file>

# 4. 解決確認
git add <conflicted-file>
git cherry-pick --continue  # または git merge --continue
```

---

## 📊 進捗管理

### wasm-development
- **タスク**: `CURRENT_TASK_WASM.md`
- **週次サマリー**: `CLAUDE.md` の Phase 15.8 セクション
- **詳細ドキュメント**: `docs/development/roadmap/phases/phase-15.8/`

### selfhost
- **タスク**: `CURRENT_TASK.md`（selfhostブランチ）
- **週次サマリー**: `CLAUDE.md` の Phase 15.7 セクション
- **詳細ドキュメント**: `docs/development/selfhosting/pipeline_v2.md`

---

## 🎯 Phase 15終了後の統合計画

### Phase 15.8完了時
1. **wasm-development完了確認**: 全テストPASS
2. **selfhostへのPR作成**: GitHub PR経由で統合
3. **コードレビュー**: 変更内容を慎重に確認
4. **マージ**: PR経由で統合（conflict解決含む）

### 統合基準
- ✅ 全スモークテストPASS
- ✅ ビルド成功（`cargo build --release --features llvm`）
- ✅ ドキュメント更新完了
- ✅ コードレビュー完了

---

## 🚨 よくあるミス＆対策

### ミス1: ブランチ間違い
```bash
# ❌ 悪い例
# 気づかずselfhostで作業してしまう

# ✅ 対策
# 作業開始前に必ず確認
git branch --show-current
```

### ミス2: 大量マージでコンフリクト
```bash
# ❌ 悪い例
git merge selfhost  # コンフリクト地獄

# ✅ 対策
# 必要な変更のみcherry-pick
git cherry-pick <specific-commit>
```

### ミス3: CURRENT_TASK.md共有
```bash
# ❌ 悪い例
# CURRENT_TASK.mdを両ブランチで共有

# ✅ 対策
# wasm-development: CURRENT_TASK_WASM.md
# selfhost: CURRENT_TASK.md
```

---

## 📚 参考リソース

- **Phase 15.8計画**: [docs/development/roadmap/phases/phase-15.8/README.md](docs/development/roadmap/phases/phase-15.8/README.md)
- **Phase 15.7計画**: [docs/development/roadmap/phases/phase-15.7/README.md](docs/development/roadmap/phases/phase-15.7/README.md)
- **CLAUDE.md**: 両ブランチの週次進捗サマリー
- **Git best practices**: [docs/guides/git-workflow.md](docs/guides/git-workflow.md)（存在する場合）

---

**作成日**: 2025-10-01
**作成者**: Claude Code + ユーザー協働
**更新**: wasm-development独立化に伴い新規作成

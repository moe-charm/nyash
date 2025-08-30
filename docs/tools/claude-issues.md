# Claude実行環境の既知のバグ

## 🐛 Bash Glob展開バグ（Issue #5811）

**問題：** Claude Code v1.0.61-1.0.81でglob展開がパイプと一緒に使うと動作しない

```bash
# ❌ 失敗するパターン（asteriskが"glob"という文字列に置換される）
ls *.md | wc -l          # エラー: "ls: 'glob' にアクセスできません"
find . -name "*.rs"      # エラー: "glob"になる
ls src/backend/vm_*.rs   # エラー: "glob: そのようなファイルやディレクトリはありません"

# ✅ 回避策1: bash -c でラップ（最も簡単）
bash -c 'ls *.md | wc -l'
bash -c 'ls src/backend/vm_*.rs | xargs wc -l'
# → Claudeではなくbash自身がglob展開するので動作する

# ✅ 回避策2: findコマンドを使う（最も確実）
find src/backend -name "vm_*.rs" -exec wc -l {} \;

# ✅ 回避策3: 明示的にファイル名を列挙
wc -l src/backend/vm.rs src/backend/vm_values.rs

# ✅ 回避策4: ls + grepパターン  
ls src/backend/ | grep "^vm_" | xargs -I{} wc -l src/backend/{}
```

**影響を受けるパターン：**
- `*.md`, `*.rs` - 通常のglob
- `src/*.py` - パス付きglob  
- `file[12].md` - 文字クラス
- `file{1,2}.md` - ブレース展開

**根本原因：** Claudeのコマンド再構築機能のバグ（`pattern`ではなく`op`フィールドを使用）

## 🚨 コンテキスト圧縮時の重要ルール

### ⚠️ **コンテキスト圧縮を検出した場合の必須手順**

**コンテキスト圧縮** = 会話履歴が要約される現象（conversation summaryで検出可能）

#### 🛑 **絶対にやってはいけないこと**
- **推測で作業を続行しない**
- 不完全な情報で重要な変更をしない  
- ビルドチェックを飛ばさない
- ユーザー確認なしに進行しない

#### ✅ **必ず実行すべき手順**
1. **⏸️ 作業停止** - 「コンテキスト圧縮を検出しました」と報告
2. **📊 状況確認** - 以下を必ずチェック：
   ```bash
   git status                    # 現在の変更状況
   git log --oneline -3         # 最近のcommit履歴
   cargo check                  # ビルド状況
   ```
3. **📋 現在タスク確認** - `CURRENT_TASK.md` を読み取り
4. **🤝 明示的確認** - ユーザーに「次に何をしましょうか？」と確認

#### 📍 **現在状況の記録場所**
- **進行中タスク**: `CURRENT_TASK.md`
- **最後の安定状態**: git commit hash  
- **ビルド状況**: `cargo check` の結果
- **重要な制約**: CURRENT_TASK.md内の注意事項

#### 💡 **圧縮時によくある混乱の回避**
- 「何をしていたか」→ `CURRENT_TASK.md`で確認
- 「ビルドできるか」→ `cargo check`で確認  
- 「どこまで進んだか」→ `git log`で確認
- 「次は何か」→ **ユーザーに明示的に確認**
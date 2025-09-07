# 論文執筆アクションプラン

## 🎯 即座に実行すべきタスク（ChatGPT5提案ベース）

### Week 1: 基盤実装とデモ準備

#### Day 1-2: TaskGroupBox完成
```bash
# 実装
- [ ] TaskGroupBox.spawn メソッド実装
- [ ] スコープ終了時の自動joinAll
- [ ] テストケース作成

# 確認コマンド
./target/release/nyash apps/tests/taskgroup-join-demo/main.nyash
```

#### Day 3-4: GUI Box最小実装
```nyash
# 必要なBox
- [ ] WindowBox（ウィンドウ作成）
- [ ] ButtonBox（ボタン）
- [ ] CanvasBox（描画）
- [ ] LabelBox（テキスト表示）

# プラグイン選択
- Ubuntu: GTK or SDL2
- Windows: Win32 or SDL2
- 共通: Dear ImGui?
```

#### Day 5-7: デモアプリ作成
```nyash
# hello-gui.nyash
box HelloApp from GuiBox {
    render() {
        return me.window("MIR15 Demo", [
            me.label("15 instructions!"),
            me.button("Click", () => print("Clicked!")),
            me.canvas(200, 200)
        ])
    }
}
```

### Week 2: 評価実験と執筆

#### Day 8-9: 命令カバレッジ測定
```bash
# プロファイリング実装
NYASH_MIR_PROFILE=1 ./target/release/nyash hello-gui.nyash
NYASH_MIR_PROFILE_JSON=1 ./target/release/nyash hello-gui.nyash > coverage.json

# 可視化スクリプト
python3 tools/visualize_coverage.py coverage.json
```

#### Day 10-11: バックエンド等価性検証
```bash
# 各バックエンドで実行
./run_all_backends.sh hello-gui.nyash
diff vm_output.log jit_output.log
diff jit_output.log aot_output.log
```

#### Day 12-14: 論文執筆開始
- [ ] Chapter 3: MIR Design（既存素材活用）
- [ ] Chapter 4: Implementation
- [ ] Chapter 5: Evaluation（実験結果）

### Week 3: 論文完成

#### Day 15-17: 理論と考察
- [ ] Chapter 2: Box Theory（数式整理）
- [ ] Chapter 6: Discussion
- [ ] Chapter 7: Related Work

#### Day 18-20: 統合と推敲
- [ ] 全体の流れ確認
- [ ] 図表作成
- [ ] 英文校正

#### Day 21: arXiv投稿
- [ ] LaTeXフォーマット変換
- [ ] 最終チェック
- [ ] 投稿

## 📋 必須チェックリスト

### 実装
- [ ] TaskGroupBox動作確認
- [ ] GUI最小デモ（Ubuntu）
- [ ] GUI最小デモ（Windows）
- [ ] 命令プロファイラー

### 評価
- [ ] 命令使用分布グラフ
- [ ] バックエンド比較表
- [ ] GUIスクリーンショット
- [ ] 性能測定結果

### 論文
- [ ] Abstract（日英）
- [ ] 8章すべて執筆
- [ ] 図表10個以上
- [ ] 参考文献30本以上

## 🚀 並列実行可能タスク

### 開発チーム
1. TaskGroupBox実装
2. GUIプラグイン開発
3. プロファイラー実装

### 執筆チーム
1. Box Theory執筆
2. Related Work調査
3. 図表作成

## 💡 成功の鍵

1. **デモ最優先**: 動くGUIがなければ説得力ゼロ
2. **データ収集**: 測定なくして論文なし
3. **ストーリー**: 「なぜ15で十分か」を明確に

## 📊 リスク管理

### 高リスク項目
- GUI実装の遅延 → SDL2で統一？
- 性能問題 → 最適化は後回し
- 論文分量不足 → 実装詳細を追加

### 対策
- 毎日進捗確認
- 問題は即座にChatGPT5相談
- 最小動作を優先

## 🎯 最終目標

**2025年9月末**: arXivに投稿完了

「15命令でGUIが動く」という衝撃的事実を世界に発信！
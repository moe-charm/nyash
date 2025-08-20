# VM: Micro-profiling & backend selection polish

Summary:
- VM 実行の簡易プロファイル（命令カウント/時間）と `--backend` の選択UI/エラー改善。

Scope/Tasks:
- [ ] VM: 命令実行回数/時間の簡易集計（ログ出力）
- [ ] CLI: backend 未実装時のメッセージ改善（jit/wasm 選択時）
- [ ] README: backend 選択例の追記

Acceptance Criteria:
- プロファイルログが出力され、選択ミス時のUI/メッセージが改善


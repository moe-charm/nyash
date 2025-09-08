# CURRENT TASK – Cranelift (AOT/JIT-AOT, Windows/Egui)

目的
- Cranelift AOT ルートをWindowsで安定化し、EguiBox（type_id=70）で実ウィンドウが表示されるところまで仕上げる。

現状（2025-09-08）
- AOT/EXE の生成・実行は成功。プラグイン読み込みもOK（EguiBox birth 成功）。
- ただし EXE 実行は Result: 0 で即終了（with-egui 機能が無効なDLLを拾っている、または実行がノンブロッキングのスタブ経路になっている可能性）。

このブランチで完了した整備（抜粋）
- ObjectBuilder（AOT）のTLS化: 1関数=1 FunctionBuilderでCFG/borrow競合を解消。
- by-name拡張シム（AOT向け）: `nyash_plugin_invoke_name_call3_i64` を追加（引数3個まで直接渡せる）。
- 非コアBoxはポリシー解決（type_id/method_id）で plugin_invoke を優先、その後name-invoke。
- Windowsリンク修正: `User32/Gdi32/Shell32/Ole32/Advapi32/Ws2_32/Ntdll` を追加。
- 実行補助: `tools/windows/run_app_egui.ps1`（stdout/stderrをlogsへ保存し、コンソールを保持）。

タスク（優先順）
1) with‑egui DLL の常用化
   - `plugins/nyash-egui-plugin` を `--features with-egui` でビルド済みか確認。
   - 生成された `nyash_egui_plugin.dll`（target/release）を優先してロードできているか確認。
2) jit-direct で name-invoke 連鎖の確認（事前健全性）
   - 期待ログ: `[LOWER] EguiBox.{birth,open,uiLabel,run,close} via name-invoke`。
3) AOT/EXE 実行時のログ確認
   - logs/app_egui_stderr.log に `[EGUI] M_OPEN/M_RUN` が記録されること。
4) まだ表示されない場合の切り分け
   - DLLのwith‑egui未適用（あるいは古いDLLが先に拾われている）
   - nyash.toml の `plugin_paths` 優先度（`plugins/*/target/release` を先頭へ）

手順（Windows / PowerShell）
- ビルド→リンク→EXE（jit-directで.o生成）
  - `pwsh -File tools\windows\build_egui_aot.ps1 -Input apps\egui-hello\main.nyash -Out app_egui.exe -Verbose`
- 実行（ログ保持）
  - `pwsh -File tools\windows\run_app_egui.ps1 -Exe .\app_egui.exe -Verbose`
  - ログ: `logs/app_egui_stdout.log`, `logs/app_egui_stderr.log`
- 事前確認（jit-directの降下ログ）
  - `$env:NYASH_JIT_TRACE_LOWER='1'; .\target\release\nyash.exe --jit-direct apps\egui-hello\main.nyash`

補足（WSLからの呼び出し例）
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Set-Location 'C:\\git\\nyash-project\\nyash_cranelift'; .\\tools\\windows\\build_egui_aot.ps1 -Input apps\\egui-hello\\main.nyash -Out app_egui.exe -Verbose"`
- `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Set-Location 'C:\\git\\nyash-project\\nyash_cranelift'; .\\tools\\windows\\run_app_egui.ps1 -Exe app_egui.exe -Verbose"`

参考（実装メモ）
- by-name拡張シム: argc>=4で `nyash_plugin_invoke_name_call3_i64` を使用（AOT名呼び出しで3引数まで直接渡す）。
- name-invokeの引数上限: jit-directはTLS経由>2も読めるが、AOTは3個までに制限。
- AOTオブジェクト出力: `NYASH_AOT_OBJECT_OUT=... --jit-direct` のみを使用（VMではなくjit-direct）。

---

担当ブランチ: `cranelift-dev`
- 続き: 上記タスクの1)〜3)を順に実施し、Eguiの実ウィンドウ表示まで到達させる。

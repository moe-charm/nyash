# Cranelift AOT で Egui Hello を実行する手順（Windows）

本ガイドは、Cranelift AOT 経路で Egui の hello サンプル（プラグイン版）をネイティブ EXE として実行する最短手順です。

前提
- 対象: Windows（PowerShell 推奨）
- プラグイン Egui を使用（with-egui 機能を有効化）
- JIT ランタイムは封印（デフォルト無効）。対象バックエンドは Interpreter / VM / Cranelift AOT / LLVM AOT の4つ。

用語
- ユーザーBox: Nyash で実装した通常のクラス
- プラグインBox: DLL/so 経由の TypeBox（v2 PluginBoxV2）
- コア埋め込みBox: ランタイム同梱の最小セット（旧: ビルトイン）

手順
1) プラグイン（Egui）を GUI 対応でビルド
   - `cd plugins/nyash-egui-plugin`
   - `cargo build --release --features with-egui`

2) Nyash 本体をビルド（Cranelift AOT ツール込み）
   - リポジトリ直下へ戻る: `cd ../../`
   - `cargo build --release --features cranelift-jit`

3) AOT EXE を生成（ワンショット推奨）
   - Windows ワンショット: `pwsh -File tools/windows/build_egui_aot.ps1 -Input apps/egui-hello-plugin/main.nyash -Out app_egui`
     - このスクリプトは他スクリプトをネスト呼び出しせず、引数を確実に伝播します（従来の「スクリプト→スクリプト」連鎖で引数が落ちる問題を解消）。
   - 共通版（PowerShell）: `powershell -ExecutionPolicy Bypass -File tools/build_aot.ps1 -Input apps/egui-hello-plugin/main.nyash -Out app_egui`
   - 代替（Bash 版）: `bash tools/build_aot.sh apps/egui-hello-plugin/main.nyash -o app_egui`

4) 実行（画面が表示されれば成功）

備考
- .o 生成時（Nyash 実行）にもウィンドウが開きます。リンクを継続するため、いったんウィンドウを閉じてください。

WSL で表示されない場合（Wayland→X11 切り替え）
- 症状: `WaylandError(Connection(NoCompositor))` などで即終了しウィンドウが出ない。
- 対処 1（推奨）: X11 に強制
  - `WAYLAND_DISPLAY=` を空にして Wayland を無効化し、X11 を選択させます。
  - 実行例: `WAYLAND_DISPLAY= WINIT_UNIX_BACKEND=x11 ./app_egui`
  - 必要に応じて: `LIBGL_ALWAYS_INDIRECT=1 WAYLAND_DISPLAY= WINIT_UNIX_BACKEND=x11 ./app_egui`
- 対処 2: Wayland を正しく通す（WSLg）
  - `export XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir`
  - 実行例: `WINIT_UNIX_BACKEND=wayland ./app_egui`
- チェック: `echo $XDG_RUNTIME_DIR $WAYLAND_DISPLAY $DISPLAY` が `… wayland-0 :0` のように設定されているか、`ls -ld /mnt/wslg/runtime-dir` でパスが存在するか確認。
   - 実行はリポジトリ直下（`nyash.toml` と plugins の相対解決に必要）
   - 任意ログ: `set NYASH_CLI_VERBOSE=1`（PowerShell: `$env:NYASH_CLI_VERBOSE='1'`）
   - 実行: `./app_egui.exe`

トラブルシュート
- プラグインが見つからない
  - `nyash.toml` の `[plugin_paths].search_paths` に `plugins/*/target/release` が含まれているか確認
  - それでも解決しない場合は `nyash_egui_plugin.dll` を EXE と同フォルダへコピーして暫定回避
- ログで確認（任意）
  - `NYASH_DEBUG_PLUGIN=1` でプラグイン登録/ロードの詳細を出力
  - `NYASH_CLI_VERBOSE=1` で補助的な実行ログを有効化

補足
- 旧ビルトイン Egui（apps/egui-hello/main.nyash）は `gui-builtin-legacy` 機能に隔離されました。
  AOT での GUI はプラグイン版（apps/egui-hello-plugin/main.nyash）を使用してください。
- JIT ランタイム（Cranelift JIT 直実行）は封印中です。必要時のみ `--features "cranelift-jit,jit-runtime"` で有効化してください。

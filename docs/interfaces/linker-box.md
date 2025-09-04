LinkerBox 仕様（Phase 15 準備 / Windows優先）

目的
- AOT 生成されたオブジェクト（`.o/.obj`）を NyRT とリンクして実行可能ファイルを得る統一レイヤ。
- 初期実装は「外部リンカー呼び出し」（MSVC `link.exe` または `lld-link`、Unix は `cc`）で動作し、将来的に lld 内蔵連携へ置換可能な設計とする。

前提／エントリポイント
- 既定のエントリポイントは NyRT 側の `main()`（`crates/nyrt` に実装）。
- AOT 側は `ny_main`（想定: `() -> i64`）を定義し、NyRT の `main()` が `ny_main()` を呼び出す。
- よって通常は `/ENTRY:` 指定は不要（NyRT をリンクしない特殊構成でのみ上書き可能とする）。

入力と出力
- 入力: 一つ以上のオブジェクト（`.obj`/`.o`）、追加ライブラリ群、ライブラリ検索パス。
- 既定ライブラリ: NyRT（Windows: `nyrt.lib`、Unix: `libnyrt.a`）。
- 出力: 実行ファイル（Windows: `*.exe`、Unix: 実行ビット付きバイナリ）。

環境変数（LinkerBox が解釈）
- `NYASH_LINKER`: 使用リンカーを強制。`link` | `lld-link` | `cc`（未指定は OS/環境から自動推定）
- `NYASH_LINK_FLAGS`: 追加フラグ（空白区切り）。
- `NYASH_LINK_VERBOSE=1`: 実コマンドラインを表示。
- `NYASH_LINK_ENTRY=<symbol>`: エントリポイントを明示的に指定（既定は未指定で NyRT の `main` を使用）。
- `NYASH_LINK_OUT=<path>`: 出力先を明示的に指定（CLI引数 `-o` が優先）。

Windows（MSVC / lld-link）
- 既定探索順: `link.exe` → `lld-link`。
- 代表フラグ:
  - `link.exe`: `/OUT:<exe>` `/SUBSYSTEM:CONSOLE`（既定） `/LIBPATH:<dir>` `nyrt.lib` 他
  - `lld-link`: `-OUT:<exe>` `-SUBSYSTEM:CONSOLE` `-LIBPATH:<dir>` `nyrt.lib`
- `PATH`/`LIB`/`LIBPATH` の整合に注意（Developer Command Prompt を推奨）。

Unix（参考）
- 代表フラグ: `cc -o <exe> <objs...> <lib paths> -L... -lnyrt -ldl -lpthread`
- 既定のオブジェクト形式/ターゲットはホストに従う。

CLI API（想定）
- `nyash --linker link|lld-link|cc --libpath <dir> --lib nyrt [--entry nyash_main] -o app <objs...>`
- AOT パスでは内部的に LinkerBox を呼び出し、上記環境変数も透過的に反映する。

エラー方針
- ツールチェーン未検出（リンカー不在）: わかりやすい対処案を表示（MSVC のセットアップ／lld の導入）。
- 未解決シンボル: `ny_main`/NyRT 関連の欠落を優先表示。
- 引数/パスのクォート: 空白を含むパスは安全にクォートして実行。

将来拡張
- 内蔵 lld を採用した一体化（外部プロセス呼び出しからの置換）。
- ターゲットトリプルの明示指定とクロスリンク（フェーズ後半）。
- 追加ランタイムやプラグイン静的リンクのオプション化。


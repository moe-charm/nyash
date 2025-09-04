Cranelift AOT Box: 設計ノートと obj 出力 PoC（Phase 15 準備）

目的
- Nyash → MIR → Cranelift AOT（C ABI）→ オブジェクト（.o/.obj）→ リンク → EXE の最小パイプラインを確立する前準備。
- 本ブランチでは「設計と仕様の確定（ドキュメント化）」のみを行い、実装は別ブランチ `phase-15/self-host-aot-cranelift` で着手する。

対象範囲（P0）
- PoC として `ny_main`（i64 → i64 返し）を定義する最小オブジェクトを Cranelift で生成できること。
- 生成物を NyRT（`crates/nyrt`）と静的リンクして実行可能ファイルを作成できること。
- 実行結果として `Result: 42` 等の既知の値確認を行うこと。

アーキテクチャ概要
- CraneliftAotBox（本ドキュメントの主題）
  - 役割: MIR から Cranelift IR（CLIF）を生成し、`cranelift-object` でオブジェクトを出力する。
  - 出力: ターゲット環境に応じた COFF/ELF/Mach-O（`cranelift-object` の既定に従う）。
  - シグネチャ: PoC は `ny_main: () -> i64`（将来的には引数の受け渡しや NyRT 呼び出しを拡張）。
- LinkerBox（別タスク、別文書で仕様化）
  - 役割: 生成された `.o/.obj` を NyRT（`libnyrt.a`/`nyrt.lib`）とリンクして EXE を作る。
  - Windows は `link.exe`/`lld-link`、Linux は `cc` 経由を想定（詳細は LinkerBox 仕様にて）。

ABI / 連携
- エントリ: `ny_main` を EXE から呼び出す形。NyRT 側が `main()` 内から `ny_main()` を適切に呼び出して結果を表示（または検証）する想定。
- ランタイム: PoC 段階では NyRT の最低限（起動/終了）に依存。将来的に checkpoint や GC バリアなどの外部関数を `extern "C"` で参照可能にする。

PoC 受入基準（P0）
- `.o/.obj` に `ny_main` シンボルが定義されている。
- `libnyrt.a`/`nyrt.lib` とリンクして実行可能ファイルが作成できる。
- 実行すると標準出力に既知の値（例: `Result: 42`）が出力される。

想定コマンド（リンク例）
- Linux: `cc -o app ny_main.o target/release/libnyrt.a -ldl -lpthread`
- Windows (MSVC): `link ny_main.obj nyrt.lib /OUT:app.exe`
- 実行時設定: 実行ファイルと同じディレクトリに `nyash.toml` を配置することでプラグイン解決を容易にする（NyRT は exe 直下→CWD の順で探索）。

CLI/ツール統合（案）
- バックエンドキー: `--backend cranelift-aot`
- PoC フラグ: `--poc-const N`（`ny_main` が `N` を返す単機能）
- 補助スクリプト（設計のみ、本ブランチでは作成しない）:
  - `tools/aot_smoke_cranelift.sh apps/APP/main.nyash -o app`
  - 流れ: Nyash → MIR → CraneliftAotBox → `.o` → LinkerBox/cc → `app`

ロードマップ
- P0: PoC スタブ `ny_main` 定数返し、リンク/実行確認。
- P1: 最小 MIR（`const_i64`/`add_i64`/`ret`）のマッピング。
- P2: NyRT チェックポイント呼び出しなど最小の外部関数連携。
- P3: Plugin 経由の I/O など実用的な呼び出しの一部導入。

既知のリスクと対策
- プラットフォーム ABI 差異: 既定の呼出規約を使用し、まず Linux で動作確認。
- オブジェクト形式差: `cranelift-object` の既定に寄り添う。必要に応じてターゲット指定を導入。
- 重複実装の懸念: 既存のオブジェクトビルダ（JIT/emit系）の再利用・抽象化を検討。

実装方針（別ブランチで実施）
- フィーチャ: `cranelift-aot = ["dep:cranelift-object"]`
- モジュール: `src/backend/cranelift/aot_box.rs` を追加し、PoC 用 `compile_stub_ny_main_i64` を提供。
- CLI 統合: `--backend cranelift-aot` と PoC フラグの導入（PoC 期間は一時的）。


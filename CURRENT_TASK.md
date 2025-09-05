# CURRENT TASK (Compact) — Phase 15 / Self-Hosting（Ny→MIR→MIR-Interp→VM 先行）

このドキュメントは「いま何をすれば良いか」を最小で共有するためのコンパクト版です。詳細は git 履歴と `docs/`（phase-15）を参照してください。

— 最終更新: 2025‑09‑05 (R1–R5 反映)

■ 進捗サマリ
- Phase 12 クローズアウト完了。言語糖衣（12.7-B/P0）と VM 分割は反映済み。
- Phase 15（Self-Hosting: Cranelift AOT）へフォーカス移行。
  - 設計/仕様ドキュメントとスモーク雛形を追加済み。
    - 設計: `docs/backend-cranelift-aot-design.md`
    - API案: `docs/interfaces/cranelift-aot-box.md`
    - LinkerBox: `docs/interfaces/linker-box.md`
    - スモーク仕様: `docs/tests/aot_smoke_cranelift.md`
    - 雛形スクリプト: `tools/aot_smoke_cranelift.sh`, `tools/aot_smoke_cranelift.ps1`
- README にセルフホスト到達の道筋を明記（C ABI を Box 化）。

■ 現在のフォーカス（優先順）
1) Ny から `nyash.toml` を読む最小ユーティリティ（ny-config）
   - FileBox + TOMLBox で `nyash.toml` の `env`/`tasks`/`plugins`/`box_types` を取得する Ny スクリプト（apps/std/ny-config.nyash）。
2) Ny スクリプトプラグインの列挙・読み込み方針
   - `nyash.toml` に `[ny_plugins]` を追加（純Nyashのプラグイン列挙）。Runner にオプトイン・フック（`NYASH_LOAD_NY_PLUGINS=1`/`--load-ny-plugins`）。
3) 直結ブリッジ（実験）の段階導入
   - `--parser ny`/`NYASH_USE_NY_PARSER=1` で v0 を直結実行（JSONはデバッグダンプへ縮退）。整合/スモーク拡充。
4) AOT P2 継続
   - CraneliftAotBox/LinkerBox のスタブから RUN スモークまでの仕上げと計測。

■ ブランチ/構成（Phase 15）
- 実装ブランチ: `phase-15/self-host-ny-mir`
- 既存 Workspace は維持（`crates/*`）。
- 方針: crates 側は変更せず「Nyash スクリプト + nyash.exe」だけで実装・運用（Windows優先）。
  - 例: `C:\git\nyash-project\nyash_self\nyash` 直下で `target\release\nyash` 実行。
- Nyash 製パーサは `apps/ny-parser-nyash/`（Nyashコード）として配置（最初は最小サブセット）。
- MIR 解釈層は既存 `backend/mir_interpreter.rs` と `runner/modes/mir_interpreter.rs` を拡充。
- AOT 関連の雛形は `src/backend/cranelift/` に維持（feature gate: `cranelift-aot`）。

■ 直後に回すタスク（2本運用）
- E) R4 finalize: Interpreter 実行前の BID v2 初期化の同期化（FileBox/TOMLBox 安定化）
  - 目的: `apps/std/ny-config.nyash` の初動「Unknown Box type」解消
  - 内容: init 完了→Interpreter 構築の順序固定、最小スモーク追加
- F) AOT P2(step‑2): emit→link→run の計測雛形
  - 目的: .o→実行→サイズ/時間ログを `tools/aot_smoke_cranelift.{sh,ps1}` に反映

■ 予定（R5 拡張: Ny Plugins → Namespace）
- Phase A（最小）: 共有レジストリ `NyModules` を追加し、`env.modules.set/get` で exports を登録/取得。
  - `[ny_plugins]` は戻り値（Map/StaticBox）を「ファイルパス→名前空間」に変換して登録。
  - 名前空間導出: ルート相対・区切りは `.`、拡張子除去・無効文字は `_`。予約 `nyashstd.*` 等は拒否。
- Phase B（範囲）: 共有Interpreterオプション（`NYASH_NY_PLUGINS_SHARED=1`）で静的定義を共有。ログに REGISTERED を出力。
- Phase C（言語結線）: `using <ns>` を `NyModules` 参照→未解決時にファイル/パッケージ解決（nyash.link）へフォールバック。

■ 直近で完了したこと（主要抜粋）
- R1: JSON v0 ブリッジ（`--ny-parser-pipe`/`--json-file`）、変換器 `src/runner/json_v0_bridge.rs`、スモーク追加
- R2: ラウンドトリップ E2E（`tools/ny_roundtrip_smoke.{sh,ps1}`）
- R3: 直結ブリッジ v0（`--parser ny`/`NYASH_USE_NY_PARSER=1`、`NYASH_DUMP_JSON_IR=1`）→ `return (1+2)*3` で 9
- R5: Ny スクリプトプラグイン（[ny_plugins]）列挙＋実行（OK/FAIL 出力・列挙のみガード付き）
- AOT P2(step‑1): RUN スモーク配線（最小オブジェクト生成＋実行ログ）

- ■ 直近で完了したこと（主要抜粋）
- T0: MIRインタープリタ強化（分岐/比較/PHI/extern/Box最小）＋ Runner 観測ログ
- T1: Nyash製ミニパーサ（整数/四則/括弧/return）→ JSON IR v0 出力
- T2: JSON IR v0 → MIRModule 受け口（`--ny-parser-pipe`）
- T3: CLI 切替/ヘルプ（`--ny-parser-pipe`/`--json-file`、mirヘルプ追補）
- T4: Docs/Samples/Runner scripts（apps/ny-mir-samples, tools/*, README 追補）
- Phase 15 起点準備
  - CLIに `--backend cranelift-aot` と `--poc-const` を追加（プレースホルダ動作）。
  - `src/backend/cranelift/{mod.rs,aot_box.rs,linker_box.rs}` の雛形追加（feature gate）。
  - MIR解釈層スケルトン（`semantics/eval.rs` と `backend/mir_interpreter.rs`）の確認

■ 開発者向けクイックメモ
- ビルド
  - VM/JIT: `cargo build --release --features cranelift-jit`
  - LLVM（必要時）: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm`
  - AOT（導入後）: `cargo build --release --features cranelift-aot`
- スモーク（DRYRUN→実行）
  - `./tools/aot_smoke_cranelift.sh release`
  - 実行モード: `CLIF_SMOKE_RUN=1`
- 参照
  - Phase 15 概要/ロードマップ: `docs/development/roadmap/phases/phase-15/README.md`, `docs/development/roadmap/phases/phase-15/ROADMAP.md`
  - ハンドオフ: `docs/handoff/phase-15-handoff.md`
  - 設計/API: `docs/backend-cranelift-aot-design.md`, `docs/interfaces/*`

■ 合否基準（P0: Ny→MIR→MIR-Interp→VM 最小成立）
- 自作Nyashパーサ（最小サブセット）が Nyash で動作し、テスト入力から中間形式(JSON暫定)を生成できる。
- Runner が中間形式を MIRModule に変換し、MIR 解釈層で実行して既知の結果（例: `Result: 42`）を出力する。
- 代表ケース（整数四則演算/括弧/return）で往復が安定。

■ JSON IR v0（暫定スキーマ）
- version: 整数（例: 0）
- kind: 固定 "Program"
- body: 配列（Stmt[]）
- Stmt（最小）
  - { "type": "Return", "expr": Expr }
- Expr（最小）
  - { "type": "Int", "value": 123 }
  - { "type": "Binary", "op": "+"|"-"|"*"|"/", "lhs": Expr, "rhs": Expr }
- error（失敗時）
  - { "version":0, "kind":"Error", "error": { "message": "...", "span": {"start":N, "end":M} } }
- 例
  - `return 1+2*3` → {"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Binary","op":"+","lhs":{"type":"Int","value":1},"rhs":{"type":"Binary","op":"*","lhs":{"type":"Int","value":2},"rhs":{"type":"Int","value":3}}}}]}
  - `return (1+2)*3` → `Binary('*', Binary('+',1,2), 3)` の形で生成

■ 補足（優先/範囲）
- 先行するのは Ny→MIR→MIR-Interp→VM の自己ホスト経路（AOTはP2以降）。
- OS 優先: Windows →（後続で Linux/macOS）。
- メモリ/GC: P0は整数演算/定数返し中心でNyRT拡張不要。
- Codex 非同期運用: `tools/codex-async-notify.sh`／`tools/codex-keep-two.sh` 継続利用。

## 実行コマンド（サマリ）
- VM/JIT 実行例
  - `printf "Hello\n" | NYASH_CLI_VERBOSE=0 ./target/release/nyash apps/ny-echo/main.nyash`
  - `printf "Hello\n" | NYASH_CLI_VERBOSE=0 ./target/release/nyash --backend vm apps/ny-echo/main.nyash`
- AOT スモーク（Phase 15）
  - Unix/WSL: `./tools/aot_smoke_cranelift.sh release`
  - Windows: `pwsh -File tools/aot_smoke_cranelift.ps1 -Mode release`
  - 実行時: `CLIF_SMOKE_RUN=1` を付与

- JSON v0 ブリッジ（R1 Quick Start）
  - パイプ実行（Unix/WSL）: `printf '{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Binary","op":"+","lhs":{"type":"Int","value":1},"rhs":{"type":"Binary","op":"*","lhs":{"type":"Int","value":2},"rhs":{"type":"Int","value":3}}}}]}' | ./target/release/nyash --ny-parser-pipe`
  - ファイル指定（Unix/WSL）: `./target/release/nyash --json-file sample.json`
  - スモーク（Unix/Windows）: `./tools/ny_parser_bridge_smoke.sh` / `pwsh -File tools/ny_parser_bridge_smoke.ps1`

- E2E ラウンドトリップ（R2）
  - Unix/WSL: `./tools/ny_roundtrip_smoke.sh`
  - Windows: `pwsh -File tools/ny_roundtrip_smoke.ps1`
  - tmux通知で並列実行（例）:
    - `CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "./tools/ny_roundtrip_smoke.sh" codex`
    - `CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "pwsh -File tools/ny_roundtrip_smoke.ps1" codex`

- Ny プラグイン列挙（R5）
  - 有効化: `--load-ny-plugins` または `NYASH_LOAD_NY_PLUGINS=1`
  - `nyash.toml` 例:
    ```toml
    ny_plugins = [
      "apps/std/ny-config.nyash",
      "apps/plugins/my-helper.nyash"
    ]
    ```
  - 実行: 列挙に加え、Interpreterで順次実行（ベストエフォート）。
  - ガード: `NYASH_NY_PLUGINS_LIST_ONLY=1` で列挙のみ（実行しない）
  - 注意: プラグインスクリプトは副作用の少ない初期化/登録処理に限定推奨。

## トレース/環境変数（抜粋）
- AOT/Link: `NYASH_LINKER`, `NYASH_LINK_FLAGS`, `NYASH_LINK_VERBOSE`
- ABI: `NYASH_ABI_VTABLE=1`, `NYASH_ABI_STRICT=1`
- VM/JIT: `NYASH_VM_PIC_STATS`, `NYASH_JIT_DUMP` など従来通り

---
詳細な履歴や議事録は docs 配下の Phase 15 セクションを参照してください。
 - ny-config（R4）
  - `./target/release/nyash apps/std/ny-config.nyash`
  - 現状: Interpreter 経路のプラグイン初期化順序により FileBox/TOMLBox を使うには Runner 側の微調整が必要（VM 経路への移行 or プラグイン登録の早期化）。スクリプト本体は追加済み。
- 直結ブリッジ v0（R3 Quick Start）
  - `printf 'return (1+2)*3\n' > t.ny && NYASH_USE_NY_PARSER=1 NYASH_DUMP_JSON_IR=1 ./target/release/nyash t.ny`

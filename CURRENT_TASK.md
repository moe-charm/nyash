# CURRENT TASK (Compact) — Phase 15 / Self-Hosting（Ny→MIR→MIR-Interp→VM 先行）

このドキュメントは「いま何をすれば良いか」を最小で共有するためのコンパクト版です。詳細は git 履歴と `docs/`（phase-15）を参照してください。

— 最終更新: 2025‑09‑05 (Phase 15.15 反映, JITオンリー / 一旦中断可)

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

■ 現在のフォーカス（JITオンリー／一旦の着地）
1) Core 緑維持（完了）
   - `tools/jit_smoke.sh` / Roundtrip(A/B) / Bootstrap(c0→c1→c1') / Using E2E = PASS
2) CI 分離（完了）
   - Core（常時）: `tools/jit_smoke.sh` + Roundtrip
   - Plugins（任意）: `NYASH_SKIP_TOML_ENV=1 ./tools/smoke_plugins.sh`（strict既定OFF、`NYASH_PLUGINS_STRICT=1`でON）
3) Self‑host E2E（完了）
   - ny_plugins 有効 + `NYASH_USE_NY_COMPILER=1` の自己ホストE2Eをオプションゲートで運用
4) クリーンアップ（完了）
   - 未使用import/変数の整理、runner責務分割、tools出力体裁の統一

■ ブランチ/構成（Phase 15）
- 実装ブランチ: `phase-15/self-host-ny-mir`
- 既存 Workspace は維持（`crates/*`）。
- 方針: crates 側は変更せず「Nyash スクリプト + nyash.exe」だけで実装・運用（Windows優先）。
  - 例: `C:\git\nyash-project\nyash_self\nyash` 直下で `target\release\nyash` 実行。
- Nyash 製パーサは `apps/ny-parser-nyash/`（Nyashコード）として配置（最初は最小サブセット）。
- MIR 解釈層は既存 `backend/mir_interpreter.rs` と `runner/modes/mir_interpreter.rs` を拡充。
- AOT 関連の雛形は `src/backend/cranelift/` に維持（feature gate: `cranelift-aot`）。

■ 再開TODO（優先順）
1) std Ny実装の実体化（P0/P1）
   - string: length/concat/slice/indexOf/equals (+trim/split/startsWith/endsWith)
   - array: push/pop/len/slice (+map/each/filter)
   - map: get/set/len/keys (+values/entries/forEach)
   - jit_smoke に機能検証を常時化（Coreは `NYASH_DISABLE_PLUGINS=1`）
2) NyコンパイラMVPのsubset拡張
   - let/call/return に続き if/ブロック/関数引数まで拡張し、`NYASH_USE_NY_COMPILER=1` スモークを充実
3) Self‑host E2E ゲートの昇格
   - 連続N回グリーン後にCI optional→requiredへ昇格（trace/hash基準）
4) Plugins厳格ONの段階移行
   - Core‑13準拠サンプルへ置換し、`NYASH_PLUGINS_STRICT=1` ゲートで順次ONに復帰

■ 予定（R5 拡張: Ny Plugins → Namespace）
- Phase A（最小）: 共有レジストリ `NyModules` を追加し、`env.modules.set/get` で exports を登録/取得。
  - `[ny_plugins]` は戻り値（Map/StaticBox）を「ファイルパス→名前空間」に変換して登録。
  - 名前空間導出: ルート相対・区切りは `.`、拡張子除去・無効文字は `_`。予約 `nyashstd.*` 等は拒否。
- Phase B（範囲）: 共有Interpreterオプション（`NYASH_NY_PLUGINS_SHARED=1`）で静的定義を共有。ログに REGISTERED を出力。
- Phase C（言語結線）: `using <ns>` を `NyModules` 参照→未解決時にファイル/パッケージ解決（nyash.link）へフォールバック。

■ 直近で完了したこと（主要抜粋／JIT）
- R1: JSON v0 ブリッジ（`--ny-parser-pipe`/`--json-file`）、変換器 `src/runner/json_v0_bridge.rs`、スモーク追加
- R2: ラウンドトリップ E2E（`tools/ny_roundtrip_smoke.{sh,ps1}`）
- R3: 直結ブリッジ v0（`--parser ny`/`NYASH_USE_NY_PARSER=1`、`NYASH_DUMP_JSON_IR=1`）→ `return (1+2)*3` で 9
- R5: Ny スクリプトプラグイン（[ny_plugins]）列挙＋実行（OK/FAIL 出力・列挙のみガード付き）
  - NyModules登録/名前空間導出/Windows正規化の仕様確定・回帰スモーク
  - using/namespace（ゲート）・nyash.link最小・resolverキャッシュ・実行時フック（提案付き診断）
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

■ 再開用クイックメモ（JITのみ）
- ビルド
  - VM/JIT: `cargo build --release --features cranelift-jit`
  - LLVM（必要時）: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm`
  - AOT（導入後）: `cargo build --release --features cranelift-aot`
- スモーク（JIT/VM）
  - Core: `NYASH_DISABLE_PLUGINS=1 NYASH_CLI_VERBOSE=1 ./tools/smoke_vm_jit.sh`
  - Parser Bridge: `./tools/ny_parser_bridge_smoke.sh`
  - Roundtrip: `./tools/ny_roundtrip_smoke.sh`（A/B）
  - Plugins: `NYASH_SKIP_TOML_ENV=1 ./tools/smoke_plugins.sh`（厳格は `NYASH_PLUGINS_STRICT=1` 時のみON）
  - Bootstrap: `./tools/bootstrap_selfhost_smoke.sh`
  - Using/Resolver: `./tools/using_e2e_smoke.sh`

■ 状態
- JIT自己ホストMVP: 到達（E2E/ブートストラップ/ドキュメント/CI分離まで完了）
- リファクタ: Step1/2/3 完了（未使用掃除・runner分割・tools体裁統一）
- 次回は「std実装の実体化」と「Nyコンパイラsubset拡張」から再開
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
- AOT/LLVM 系は後段（当面OFF）

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

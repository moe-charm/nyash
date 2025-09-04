Phase 15 — Self-Hosting (Cranelift AOT) 引き継ぎメモ

概要（2025-09-05）
- 目的: Nyash → MIR → Cranelift AOT → オブジェクト → リンク → EXE の最小パイプライン確立に向けた準備（設計/仕様/スモーク雛形）。
- 実装は別ブランチ `phase-15/self-host-aot-cranelift` で着手予定。現状はドキュメントと雛形スクリプトまで整備。

このブランチで完了したこと
- Cranelift AOT 設計とインタフェース草案のドキュメント追加:
  - docs/backend-cranelift-aot-design.md
  - docs/interfaces/cranelift-aot-box.md
- LinkerBox 仕様とAOTスモーク仕様（擬似出力）
  - docs/interfaces/linker-box.md
  - docs/tests/aot_smoke_cranelift.md
- Phase 15 集約README
  - docs/phase-15/README.md
- スモーク雛形（DRYRUN既定。CLIF_SMOKE_RUN=1で実行）
  - tools/aot_smoke_cranelift.sh（Unix/WSL）
  - tools/aot_smoke_cranelift.ps1（Windows）

次にやること（別ブランチで実装）
1) ブランチ作成: `git switch -c phase-15/self-host-aot-cranelift`
2) CraneliftAotBox（PoC）
   - `src/backend/cranelift/aot_box.rs` を追加
   - `compile_stub_ny_main_i64(val, out_obj)` で `.o/.obj` を出力
   - Cargo feature: `cranelift-aot = ["dep:cranelift-object"]`
3) LinkerBox（Windows優先）
   - `.o/.obj` + NyRT（`libnyrt.a`/`nyrt.lib`）で EXE を生成
   - 環境変数: `NYASH_LINKER`/`NYASH_LINK_FLAGS`/`NYASH_LINK_VERBOSE`
4) CLI 統合（PoC）
   - `--backend cranelift-aot` と `--poc-const N`
5) スモーク
   - apps/ny-hello → emit → link → run → `Result: 42` を確認
   - 既存スクリプト雛形を “実行” モードで動くよう配線

運用メモ（Codex 非同期 2本）
- 2本起動: `CODEX_MAX_CONCURRENT=2 CODEX_DEDUP=1 ./tools/codex-keep-two.sh codex "<Task A>" "<Task B>"`
- 1本起動: `CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "<task>" codex`
- ログ: `~/.codex-async-work/logs/`

スモーク雛形の使い方（DRYRUN）
- Unix/WSL: `./tools/aot_smoke_cranelift.sh release`
- Windows: `pwsh -File tools/aot_smoke_cranelift.ps1 -Mode release`
- 実行モード: `CLIF_SMOKE_RUN=1` を付与（AOT実装が入った後に使用）

参考リンク
- docs/phase-15/README.md（全体像）
- docs/backend-cranelift-aot-design.md（AOT設計）
- docs/interfaces/cranelift-aot-box.md（CraneliftAotBox API案）
- docs/interfaces/linker-box.md（LinkerBox仕様）
- docs/tests/aot_smoke_cranelift.md（スモーク仕様と擬似出力）

補足（メモリ/GC）
- P0/P1（定数返し/整数演算）では追加のメモリ系Boxは不要。
- P2以降で配列/文字列の生成・更新をAOTから行う場合、NyRTに最小のC ABI（roots/barrier/alloc系）を追加予定（docsに設計案を後続追記）。


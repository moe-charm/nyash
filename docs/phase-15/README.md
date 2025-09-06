Phase 15 — Self-Hosting (Cranelift AOT) 準備メモ

注意: Phase 15 の正本ドキュメントは `docs/development/roadmap/phases/phase-15/` 配下です。全体の入口は `INDEX.md` を参照してください。
→ docs/development/roadmap/phases/phase-15/INDEX.md

目的
- Nyash → MIR → Cranelift AOT（C ABI）→ オブジェクト → リンク → EXE の最小パイプライン確立。
- 本ブランチでは「影響小・再現性高い」準備（設計/仕様/スモーク雛形）に限定し、実装は別ブランチで行う。

現状ステータス（このブランチ）
- 設計ノート: docs/backend-cranelift-aot-design.md
- インタフェース草案: docs/interfaces/cranelift-aot-box.md
- LinkerBox 仕様: docs/interfaces/linker-box.md
- AOTスモーク仕様（擬似出力）: docs/tests/aot_smoke_cranelift.md
- スモーク雛形（DRYRUN 既定）:
  - tools/aot_smoke_cranelift.sh（Unix/WSL）
  - tools/aot_smoke_cranelift.ps1（Windows）

ハンドオフ
- 引き継ぎの全体像と運用メモは docs/handoff/phase-15-handoff.md を参照。

次ブランチで実装する項目（phase-15/self-host-aot-cranelift）
- CraneliftAotBox: `compile_stub_ny_main_i64` → `.o/.obj` を出力。
- LinkerBox: `.o/.obj` + NyRT（libnyrt）で EXE にリンク（Windows優先）。
- CLI統合: `--backend cranelift-aot` と PoC フラグ（`--poc-const`）。
- スモーク実行: apps/ny-hello → EXE 生成・起動確認。

合否基準（P0）
- `ny_main` を定義するオブジェクトを生成できる。
- NyRT とリンクして EXE を生成できる。
- 実行し、既知の値（例: `Result: 42`）を出力。

補足
- Windowsを先行サポートし、Linux/macOS は後続対応。
- 実出力やビルドログは `tools/codex-async-notify.sh` のログ参照運用を継続。

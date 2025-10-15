# CURRENT_TASK — Status and Next Steps (2025‑10‑16)

このページは「いま何をしていて、次に何をするか」を 1 画面で把握できるようにするダッシュボードだよ。最新の作業に合わせて随時更新していくにゃ。

## Snapshot
Updates (today)
- Phase‑31（static → singleton 正規化）進捗
  - A‑1b 完了: 「関数スコープのシングルトン・キャッシュ」を導入して、同一関数内の `me` プレースホルダ重複生成を解消。
    - 実装: `MirBuilder.current_fn_singletons` を追加し、`maybe_prepend_static_me()` から `current_fn_singleton()` を使用。
    - `main`/メソッド/静的メソッドの各 lowering フェーズでキャッシュの save/restore を実施。
  - A‑1c 完了: ModuleFunction call の Verifier と VM 側整備
    - Verifier が ModuleFunction の受領者を検査（Known かつ Box 型のときに Fail‑Fast）。
    - VM Router/legacy fallback は常に receiver 前提。Void 受領者は即時エラーに。
    - ModuleFunction トランポリンを `handlers/calls/trampolines.rs` に分離し、Array/Map/String/Console を表駆動化。
  - A‑2 着手: `Const Void` (静的 me) を `static_singleton::get()` で実体 BoxRef 化。
    - `runtime/static_singleton.rs` を追加し、`OnceCell<Mutex<…>>` で Box 単位のシングルトンを lazy 初期化。
    - Interpreter `handle_const` が `MirType::Box` の場合に singleton を取得して受領者を具体化。
- Json canonicalization fix
  - `hostbridge.extern_invoke` の引数をプリミティブ化する正規化ヘルパを導入。Plugin ArrayBox でも正しく文字列を渡せるようになったよ。
  - `JsonCanonicalBox.canonicalize` を純 String→String 経路に統一して `json_canonical_box_vm` / `mirio_canonicalize_vm` スモークが PASS したにゃ。
  - `host_handles::release()` を追加してホストアンカー経由の一時ハンドルを解放。
- Docs
  - Phase‑31 計画書を `docs/development/roadmap/phases/phase-31-box-Normalization/INDEX_JA.md` に追加済み。

Open issues / blockers
- Phase‑31 残: ドキュメント/テスト更新と、Plugin 既存 ABI へのトランポリン実配線（registry へ新エントリ登録）。
- Frozen guide への Windows 例追記・マクロ SKIP 解消など、以前から残っている P0 がまだ開いているにゃ。

## Prioritized TODOs
- **P0 — 直近解消したいもの**
  1. Frozen guide: “Static runtime（Windows）example” 追記（ドキュメント）
  2. マクロ系 SKIP の撤去（Array.length / Map.keys/values / derive equals の安定化）
- **P1 — quality of life**
  - Doctor: structured error messages（missing clang/llvmlite/allowlist/lib paths）
  - Harness: tighter logs for `--target windows` & optional IR dump hint
  - Gate C: reduce deprecate/alias noise earlier in runner; aim for true PASS (no SKIP) in nyvm_* smokes
- **P2 — later**
  - CI: build-only job for `llvm_backend` / harness smoke（opt-in）
  - CI: optional Windows cross pipeline doc（no runner）

## Guardrails / Principles
- Fail-Fast: no silent fallback for FFI/extern; defaults stay strict
- Minimal ENV: config broadens allowlist but never changes default semantics
- Structure first: helpers isolated under `tools/aot/` と `tools/aot/windows/`
- Docs placement: `docs/guides/`, `docs/reference/`, `docs/development/roadmap/` の既存ディレクトリに限定

## How to Reproduce (quick memo)
- WSL（Linux 単体ビルド）
  - `./target/release/hakorune --backend mir --emit-mir-json build/mir/main.mir.json examples/simple_return.hako`
  - `tools/aot/emit_object_via_extern_c.sh build/mir/main.mir.json build/obj/main.o`
  - `tools/aot/link_with_clang.sh -o bin/hako-frozen-v1 build/obj/main.o`
- WSL → Windows（COFF）
  - `python3 tools/llvmlite_harness.py --in build/mir/main.mir.json --target windows --out build/obj/main_win.obj`
  - `clang link_stub_main.c nyrt_min_stubs_win.S main_win.obj -o test_main.exe`

## References
- Frozen toolchain guide: `docs/guides/frozen-toolchain.md`
- Windows 実績レポート: `build/WINDOWS_LINK_TEST_REPORT.md`
- Frozen v1 Box spec: `docs/reference/boxes/frozen_v1.md`
- Roadmap Phase‑15.77: `docs/development/roadmap/phases/phase-15.77/INDEX.md`
- Phase‑31 計画: `docs/development/roadmap/phases/phase-31-box-Normalization/INDEX_JA.md`

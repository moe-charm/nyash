# Smokes — How‑To（前提→手順→検証）

目的
- 代表スモークを素早く回して、回帰を検知する。

前提
- リリースビルド済み: `cargo build --release --features llvm`
- LLVM 18 が導入済み（AOT 経路のとき）

手順（推奨ランナー）
1) LLVM curated
   - 実行: `tools/smokes/curated_llvm.sh [--phi-off]`
   - `--phi-off`: `NYASH_MIR_NO_PHI=1` を有効化し、検証を緩和
2) PHI 不変条件パリティ
   - 実行: `tools/smokes/curated_phi_invariants.sh`
   - PyVM と llvmlite の stdout/exit code を比較

手動スモーク（例）
- Core (LLVM): `examples/llvm11_core_smoke.nyash`
- Async (LLVM only):
  - `apps/tests/async-await-min/main.nyash`
  - `apps/tests/async-spawn-instance/main.nyash`
  - `apps/tests/async-await-timeout-fixed/main.nyash`（`NYASH_AWAIT_MAX_MS=100`）

アーカイブ（非推奨）
- `tools/smokes/archive/` に旧ランナー（JIT/Cranelift 時代）が存在
  - `smoke_phase_10_10.sh`, `smoke_vm_jit.sh`, `smoke_async_spawn.sh`, `jit_smoke.sh`, `aot_smoke_cranelift.sh`
  - これらは基本使わず、curated 系を使用

便利フラグ
- `NYASH_LLVM_USE_HARNESS=1`: llvmlite ハーネス経由
- `NYASH_MIR_NO_PHI=1`, `NYASH_VERIFY_ALLOW_NO_PHI=1`: PHI 無しモード

検証
- 0 で成功、非 0 で失敗（CI 連携可）

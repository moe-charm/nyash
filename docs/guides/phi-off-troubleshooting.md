## PHI-off Troubleshooting

Scope: MIR PHI-off (edge-copy) policy with LLVM harness PHI synthesis.

> Phase‑15 では PHI-on が既定だよ。このガイドは `NYASH_MIR_NO_PHI=1` を明示してレガシー edge-copy モードを再現しているときだけ参照してね。

Symptoms and Hints

- Merge block contains self-copy to merged value
  - Symptom: Verifier (edge-copy strict) complains or trace shows unexpected write at merge.
  - Check: Enable `NYASH_VERIFY_EDGE_COPY_STRICT=1` and locate offending merge block.
  - Fix: Ensure copies are inserted in predecessors only; merge binds the already-defined dst.

- Missing predecessor copy into merged destination
  - Symptom: Edge-copy strict reports missing pred; phi-trace checker shows `missing=[...]`.
  - Check: `tools/phi_trace_check.py --file <trace.jsonl> --summary` (or drop `--summary` to see diffs on error).
  - Fix: Builder/Bridge must insert `Copy{dst=merged}` at that predecessor end.

- Synthesized zero in PHI wiring
  - Symptom: phi-trace shows `zero=1` or checker with `--strict-zero` fails.
  - Check: Ensure the source value exists at predecessor end (resolve_i64/snapshot); add casts/boxing at pred end.
  - Fix: Route casts at pred end (not at merge); for strings/handles, ensure i64 handle flows across blocks.

Tools
- JSON trace: set `NYASH_LLVM_TRACE_PHI=1` and `NYASH_LLVM_TRACE_OUT=<path>`
- One-shot: `tools/phi_trace_run.sh <app.nyash> [--strict-zero]`
- Strict verifier (PHI-off): `NYASH_VERIFY_EDGE_COPY_STRICT=1 cargo test --lib`

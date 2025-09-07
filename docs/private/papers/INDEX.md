# Private Papers Index

- Paper A: MIR13 / Core‑13 IR
  - main: papers/paper-a-mir13-ir-design/main-paper-jp.md
  - spec: papers/paper-a-mir13-ir-design/MIR13_CORE13_SPEC.md
  - artifacts: papers/paper-a-mir13-ir-design/_artifacts/

- Paper B: Nyash 言語と実行モデル（LifeBox/birth‑fini）
  - main: papers/paper-b-nyash-execution-model/main-paper-jp.md
  - artifacts: papers/paper-b-nyash-execution-model/_artifacts/
  - figures: papers/paper-b-nyash-execution-model/figures/

- Paper E: LifeBox Model / LoopForm IR（LoopSignal）
  - main: papers/paper-e-loop-signal-ir/main-paper-jp.md
  - appendix: papers/paper-e-loop-signal-ir/appendix-rewrites.md, appendix-effects.md
  - reviews: papers/paper-e-loop-signal-ir/claude_output.md, gemini_output.md, synthesis.md

Build (Pandoc):
- bash tools/papers/build.sh a-jp  # or b-jp / all
- output: docs/private/out/

# Self Current Task — Overview (llvm)

目的
- LLVM AOT の Core‑13 前提化を完了し、VM と一致する代表ケースを安定化。
- BitOps/Shift の構文着地に合わせ、AOT 側の IR 生成/実行互換を維持。

指針
- Opaque Pointer 対応済（get_element_type 不使用）。
- `env.box.new/_i64x` は NyRT shim に統一、引数は i64 正規化。
- BinOp/Compare は i64/f64 必要分を網羅する。


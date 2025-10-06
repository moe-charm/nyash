# Phase 15.9 — LLVM Optimization (Wasm stage)

Goal
- Validate the user hypothesis: “inner loop header is too heavy”, and identify simple, structural fixes that reduce loop overhead without changing semantics.

Context (v4 vs v2)
- v2 inner header (bb12):
  - 5× PHI
  - Timer check + compare (extern; non-optimizable by LICM)
- v4 inner header (bb11) observed (simplified):
  - 5× PHI
  - const 100 (loop-invariant but rebuilt every iteration)
  - copy ×2 (operands normalization)
  - icmp + copy + br

Problem (confirmed)
- A loop-invariant constant is emitted inside the loop header:
  - `%63 = const 100` appears in the header block each iteration.
  - This is invariant across the loop and should be hoisted to a preheader or function scope.
- Redundant copies around compare (`copy %52` and `copy %63`) add extra instructions.

Root cause candidates
- The MIR/JSON v0 → LLVM lowering currently materializes small constants near use-sites (per-block), not at a preheader.
- Minimal canonicalization in the loop header leaves copy and const in place; LLVM may not fully clean due to block structure or missing canonical preheader.

Proposed fixes (small, structural)
1) LICM at our level (preferred minimal impact)
   - Add a tiny MIR optimizer pass that hoists loop-invariant const and simple pure ops to the loop preheader.
   - Scope: integer/float constants and read-only intrinsics; skip any extern calls or stateful ops.
   - Acceptance: no `const <imm>` inside loop headers after pass; PHI remain grouped at the block top.

2) LLVM builder placement policy
   - When lowering ICmp with an immediate operand, materialize the constant in the preheader (or re-use a function-scope constant) instead of in the header.
   - Re-use strategy: map literals (e.g., i64 100) to a single LLVM const Value per function.

3) Copy-folding in headers
   - Remove trivial copies feeding ICmp; wire ICmp directly from PHI and the hoisted constant.
   - Prefer: `icmp lt %inner, 100` without intermediate `copy`.

4) Ensure loop preheader exists (if not, create)
   - Loop canonicalization: PHI nodes grouped at header top, and a unique preheader to host invariants.
   - Keep invariants in preheader; leave only PHI + compare + branch in the header.

Where to implement (order of preference)
- MIR optimizer (licm_simplify.rs): simplest, backend-agnostic、可逆（既定OFF→段階ONでも可）。
- LLVM builder (ny-llvmc): constant placement and copy-folding in codegen (fast win even without a general LICM).
- JSON v0 bridge: avoid introducing header-local const where not necessary.

Validation plan
- IR dump: `NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_DUMP_IR=tmp/phase159.ll ...` で IR を出力。
- Gate tests:
  - Loop header must contain: PHI×N + icmp + br（Timer系は除外）。
  - Disallow `= addi`/`= const` in the header unless used by PHI init.
- Performance smoke:
  - micro-bench loop (100 iters) before/after：ヘッダ命令数削減を確認（静的）＋実行時間（相対）

Acceptance criteria
- v4 inner headerから `const 100` が除去（preheaderへ移動）。
- copy×2 が 0〜1 まで削減（オペランドを直接使用）。
- PHI は先頭に並び、ヘッダ終端は `icmp + br` のみ（Timer チェックあるケースは例外）。
- 代表スモーク（quick）グリーン維持。IR 検査は dev/gated に限定。

Implementation sketch
- Pass: `mir/optimizer/licm_simplify.rs`（新規）
  - 入力: CFG + ブロック毎の命令列
  - 手順:
    1) ループ自然ヘッダ検出（back-edgeからの簡易検出でOK）
    2) 入力ブロック集合の指す値の使用を解析し、`const` かつ side‑effect なし・ループ内で値が変わらないものを抽出
    3) preheader がなければ作成、対象命令を preheader に移動／再生成
    4) ヘッダの copy をオペランド直接使用に書き換え（SSA更新は局所）
  - 出力: MIR（構造不変）

- Builder/LLVM 補助
  - ICmp 生成時に即値を優先（copy 不要）。
  - 定数値は function-scope のコンスタントテーブルから再利用（i64:HashMap）。

Risk/Notes
- LICM の対象は const のみに限定（第1段階）。
- Timer 系の extern 呼び出しは不変とみなさない（移動禁止）。
- preheader 生成で IR が分岐しても PHI 不変条件（先頭グループ化）は崩さない。

Next steps
- [ ] MIR: licm_simplify（const hoist）試験的に実装（既定OFFで dev のみ）
- [ ] LLVM builder: ICmp 即値最適化 + const 再利用テーブル
- [ ] IR 検査（dev）: ループヘッダに `const` が存在しないことを assert
- [ ] micro-bench を quick 任意ジョブに追加（CI 既定は維持）

Appendix — Evidence (v4 sample)
```
bb11:
  %61 = phi [%31, bb9], [%61, bb12]
  %60 = phi [%30, bb9], [%82, bb12]
  %57 = phi [%27, bb9], [%79, bb12]
  %53 = phi [%23, bb9], [%53, bb12]
  %52 = phi [%46, bb9], [%85, bb12]
  %63 = const 100          ; loop-invariant, should be hoisted
  %64 = icmp lt %52, %63
  br %64, label bb12, label bb13
```

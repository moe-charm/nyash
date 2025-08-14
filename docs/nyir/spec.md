# NyIR v1 Specification (Draft Skeleton)

Purpose
- Define NyIR (public intermediate representation) as the portable contract for all frontends/backends.
- Freeze the 25-instruction set, effects, ownership forest, weak semantics, and bus contract.

Status
- Version: nyir1.0 (draft)
- Change policy: backward-compatible extensions via feature bits; no undefined behavior.

1. Instruction Set (26) — Semantic Summary
- Tier-0: Const, BinOp, Compare, Branch, Jump, Phi, Call, Return
- Tier-1: NewBox, BoxFieldLoad, BoxFieldStore, BoxCall, ExternCall, Safepoint, RefGet, RefSet, WeakNew, WeakLoad, WeakCheck, Send, Recv
- Tier-2: TailCall, Adopt, Release, MemCopy, AtomicFence

For each instruction (to be filled):
- Semantics: preconditions, side-effects, failure behavior
- Effects: pure/mut/io/control
- Verification: structural/formal checks
- Example: minimal snippet

2. Effects Model
- pure: no observable side effects; can reorder with pure
- mut: mutates owned state; preserve order per resource
- io: interacts with outside world; preserve program order
- control: affects control flow/terminators

3. Ownership Forest
- Strong edges form a forest (strong in-degree ≤ 1).
- No strong cycles.
- Weak edges do not propagate ownership.
- Adopt/Release rules; Safepoint and split-fini notes.

4. Weak Semantics
- Representation: {ptr, generation}
- WeakLoad: returns null if generation mismatch
- WeakCheck: returns false if invalid
- O(1) on-access validation

5. Bus Contract
- Local: in-order delivery
- Remote: at-least-once (or selectable); specify ordering/dup policy

6. Formats
- Text .nyir: human-readable; module header, features, const pool, functions (blocks, instrs)
- Binary .nybc: sectioned; header, features, const pool, functions, metadata; varint encodings
- Feature bits and extension sections for evolution

7. Verification
- Program well-formedness: reachability, termination, dominance
- Phi input consistency
- Effects ordering constraints
- Ownership forest & weak/generation rules
- Bus safety checks

8. External Calls (Core)
- ExternCall: Interface to external libraries as Everything is Box principle
- Format: ExternCall { dst, iface_name, method_name, args }
- Effect annotation required (pure/mut/io/control)
- BID (Box Interface Definition) provides external API contracts

9. Mapping Guidelines
- WASM: imports, memory rules, (ptr,len) strings
- VM: function table mapping
- LLVM: declare signatures; effect to attributes (readonly/readnone, etc.)

10. Golden/Differential Testing
- Golden .nyir corpus; cross-backend consistency (interp/vm/wasm/llvm)

11. Versioning & Compatibility
- nyir{major.minor}; feature negotiation and safe fallback

12. Glossary
- Box, Effect, Ownership forest, Weak, Safepoint, Bus, Feature bit

Appendix
- Minimal examples (to be added)
- Rationale notes

------------------------------------------------------------
NyIR-Ext（拡張セット）
------------------------------------------------------------

目的
- NyIR Core（26命令）は基本セマンティクス凍結。外部世界接続（ExternCall）を含む基本機能確立。
- 拡張は言語固有機能に限定：「例外」「軽量並行/非同期」「アトミック」の3領域を段階追加。
- Core機能: すべての言語で必要な基本セマンティクス（制御フロー・Box操作・外部呼び出し）
- Extension機能: 特定言語でのみ必要な高級機能（例外処理・並行処理・アトミック操作）

Ext-1: 例外/アンワインド（exceptions）
- 命令（案）：
  - Throw(value)
  - TryBegin
  - TryEnd(handlers=[type:block])
- 効果: control（脱出）。mut/io と混在する場合は当該効果も従属。
- 検証: Try の整合（入れ子/終端）、未捕捉 Throw のスコープ明示、no-throw 関数属性の尊重。
- バックエンド指針:
  - LLVM: 言語EH or setjmp/longjmp 系へ lower
  - WASM: v0は例外未使用なら「エラー戻り値」へ降格（no-exception モード）

Ext-2: 軽量並行/非同期（concurrency）
- 命令（案）：
  - Spawn(fn, args...)
  - Join(handle)
  - Await(task|event)  // または Wait
- 効果: io（スケジューラ相互作用）＋ control（待機・解除）。
- 検証: Join/Spawn のライフサイクル均衡、待機の整合（任意でデッドロック静的ヒント）。
- バックエンド指針:
  - LLVM: pthread/std::thread/独自ランタイム
  - WASM: スレッド未使用ならランループ/Promise/MessageChannel 等で近似

Ext-3: アトミック（atomics）
- 命令（案）：
  - AtomicRmw(op, addr, val, ordering=seq_cst)
  - CAS(addr, expect, replace, ordering=seq_cst)
- 既存の AtomicFence は維持。
- 効果: mut（アトミック副作用）。必要に応じて atomic/volatile フラグを effect/属性で付与。
- バックエンド指針:
  - LLVM: `atomicrmw`, `cmpxchg`
  - WASM: Threads 提案が前提。未サポート環境では未実装 or 疑似ロック（デモ用途）

備考
- 可変引数は slice 表現で代替（IR 追加不要）。
- クロージャ捕捉は Box + env フィールド + BoxCall で表現（IR 追加不要）。
- 動的言語（辞書/配列/プロトタイプ）は標準Box（std）で受ける（IR 追加不要）。
- 関数属性に「エラー戻り値モード/no-exception」などのメタを付与し、例外禁止環境へも対応する。

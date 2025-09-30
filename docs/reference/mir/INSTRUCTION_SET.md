# Nyash MIR Instruction Set — Freeze Plan (Phase 15.7)

Status: Canonical (Source of Truth)
Last Updated: 2025-10-03

この文書は Nyash の MIR 命令セットの唯一の参照だよ。今フェーズで「凍結セット」を定義し、呼び出しを MirCall に統一する設計を確定する。実装は段階導入だけど、仕様はここに集約するよ。

単一の列挙はコードでここにある: `src/mir/instruction.rs`
呼び出し統一の定義はここ: `src/mir/definitions/call_unified.rs`

注意: Debug/Nop/Safepoint はメタ命令（計数外）。

Transition Note（要点）

- MirCall（Callee）へ統一: 呼び出しは 1 命令に集約（Call/BoxCall/ExternCall/NewBox/NewClosure/PluginInvoke を表現できる）
- 既定: Builder/VM/Printer は順次 MirCall 優先へ寄せる（レガシーは互換/診断対象）
- 統合の方向性（段階適用済み含む）
  - TypeCheck/Cast → TypeOp
  - WeakNew/WeakLoad → WeakRef（統合PoC）
  - BarrierRead/BarrierWrite → Barrier（統合版）
  - Print → ExternCall("env.console.log")（Deprecated）
  - PluginInvoke → MirCall(Method) に吸収（完全移行時は消滅）

診断/ガード
- レガシー検出: `src/mir/optimizer_passes/diagnostics.rs`
  - `NYASH_OPT_DIAG=1` で警告、`NYASH_OPT_DIAG_FORBID_LEGACY=1` で Fail‑Fast

## 現在の列挙 — 実装に存在する主な命令（網羅）
- Const
- Copy
- Load
- Store
- UnaryOp
- BinOp
- Compare
- Jump
- Branch
- Phi
- Return
- Call
- ExternCall
- BoxCall
  - Note: BoxCall carries optional `method_id` (numeric slot) when the builder can resolve the receiver type; otherwise falls back to name-only late bind. Universal methods use reserved slots: 0=toString, 1=type, 2=equals, 3=clone.
- NewBox
- ArrayGet
- ArraySet
- RefNew
- RefGet
- RefSet
- Await
- Print（Deprecated: ビルダーは発行しない。代わりに `ExternCall env.console.log` を使用）
- TypeOp（TypeCheck/Cast統合）
- WeakRef（WeakNew/WeakLoad統合）
- Barrier（Read/Write統合）
 - NewClosure
 - PluginInvoke（Deprecated → MirCallへ移行）
 - Throw / Catch（Deprecated → Result‑modeへ）
 - BarrierRead / BarrierWrite（Deprecated → Barrierへ）

## 凍結セット（本フェーズでの安定仕様）
- 基本演算(5): Const, UnaryOp, BinOp, Compare, TypeOp
- メモリ(2): Load, Store
- 制御(4): Branch, Jump, Return, Phi
- 呼び出し(1): MirCall（Callee で Global/Extern/ModuleFunction/Method/Constructor/Closure/Value を表現）
- GC(2): Barrier, Safepoint
- 構造(2): Copy, Nop（最適化/検証用・意味論不変）

参考: 配列/参照（ArrayGet/ArraySet/Ref*/Weak*）は段階的に Box/Extern へ統合。モジュール関数は Callee::ModuleFunction で一元表現し、NameConst/legacy は段階縮小する。

## 非推奨セット（段階的に削除）
- Throw / Catch（Result‑mode lowering へ移行）
- PluginInvoke（MirCall(Method) へ統合）
- BarrierRead / BarrierWrite（Barrier へ統合）
- TypeCheck / Cast（TypeOp に統合）
- RefNew / RefGet / RefSet, WeakNew / WeakLoad（WeakRef/BoxCall へ集約）

## MirCall（呼び出しの統一）
- 定義: `docs/reference/mir/call-unified.md` を参照
- ソース: `src/mir/definitions/call_unified.rs`
- 表現:
  - Global(String)
  - Extern(String)
  - ModuleFunction(String)
  - Method { box_name, method, receiver, certainty }
  - Constructor { box_type }
  - Closure { params, captures, me_capture }
  - Value(ValueId)

### レガシー→MirCall マッピング
- Call(func=const "name", args) → MirCall::global(name)（ビルトインのみ）
- Call(func=const "Class.method/N", args) → MirCall::module_function("Class.method/N")
- BoxCall(box_val, method, args[, method_id]) → MirCall::method(receiver=box_val, method)
- ExternCall(iface, method, args) → MirCall::external("iface.method")
- NewBox(box_type, args) → MirCall::constructor(box_type)
- NewClosure(params, captures, me) → MirCall::closure(...)
- PluginInvoke → MirCall::method（差分は効果/解決ポリシーに反映）

Notes
- Print は Extern 化（`env.console.log`）
- PHI の不変条件は別紙参照: `docs/reference/mir/phi_invariants.md`
- Builder/VMの Call サイトでは in‑block 材化（LocalSSA）を優先（仕様不変）

## Meta (降格対象; カウント外)
- Debug
- Nop
- Safepoint

## 同期ルール
- 命令の追加/削除/統合は、本ドキュメント→実装（列挙/Printer/Verifier/Optimizer/VM）→スモーク/ゴールデンの順で同期
- レガシー検出は常時ONで観測、削除は小粒差分・可逆を維持

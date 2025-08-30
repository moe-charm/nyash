# Nyash MIR Instruction Set (Canonical 26 → migrating to Core-15)

Status: Canonical (Source of Truth) — transitioning
Last Updated: 2025-08-25

この文書はNyashのMIR命令セットの唯一の参照（現状は26命令）だよ。Core-15 への段階移行を進めており、安定次第この文書を15命令へ更新し、テスト固定数も切り替える（移行中は26を維持）。

注意: Debug/Nop/Safepointはビルドモードでの降格用メタ命令であり、コア命令数には数えない。

Transition Note

- Builder/Rewrite/JIT は既に以下の統合を段階適用中
  - TypeCheck/Cast → TypeOp
  - WeakNew/WeakLoad → WeakRef
  - BarrierRead/BarrierWrite → Barrier
  - Print → ExternCall(env.console.log)（Deprecated）
  - PluginInvoke → BoxCall（Deprecated; 名前/スロット解決はBoxCall側で処理）
- VM/JIT の代表的な Core-15 カバー手順は `docs/reference/mir/MIR15_COVERAGE_CHECKLIST.md` を参照。
- Core-15 安定後に本ドキュメントの「Core Instructions」を15命令へ更新し、マッピング表を併記する。

## Core Instructions（26）
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

## Core-15（Target; 移行中の最小コア）
- 基本演算(5): Const, UnaryOp, BinOp, Compare, TypeOp
- メモリ(2): Load, Store
- 制御(4): Branch, Jump, Return, Phi
- Box(3): NewBox, BoxCall, PluginInvoke
- 配列(2): ArrayGet, ArraySet
- 外部(1): ExternCall

Notes
- Print/Debug/Safepointはメタ/Extern化（Print→ExternCall）。
- WeakRef/Barrier は統合済み（旧WeakNew/WeakLoad/BarrierRead/WriteはRewriteで互換）。
- Call は BoxCall へ集約（PluginInvokeはDeprecated）。

## Meta (降格対象; カウント外)
- Debug
- Nop
- Safepoint

## 同期ルール
- 命令の追加/削除/統合は、まずこの文書を更新し、次に実装（列挙/Printer/Verifier/Optimizer/VM）を同期。最後に「総数=26」テストを更新する。
- 実装が26を外れた場合はCIを赤にする（設計の合意なく増減させないため）。

# Nyash MIR Instruction Set (Canonical 26)

Status: Canonical (Source of Truth)
Last Updated: 2025-08-25

この文書はNyashのMIR命令セットの唯一の参照（26命令）だよ。実装は常にこの一覧に一致し、総数はテストで26に固定する。

注意: Debug/Nop/Safepointはビルドモードでの降格用メタ命令であり、コア26命令には数えない。

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
- Print
- TypeOp（TypeCheck/Cast統合）
- WeakRef（WeakNew/WeakLoad統合）
- Barrier（Read/Write統合）

## Meta (降格対象; カウント外)
- Debug
- Nop
- Safepoint

## 同期ルール
- 命令の追加/削除/統合は、まずこの文書を更新し、次に実装（列挙/Printer/Verifier/Optimizer/VM）を同期。最後に「総数=26」テストを更新する。
- 実装が26を外れた場合はCIを赤にする（設計の合意なく増減させないため）。

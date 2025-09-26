# 14命令のミニマルIRによる統一実行基盤設計（MIR14, PHIオフ方針）
著者: Nyash Project

要旨
Nyashは「Everything is Box」哲学を核に、14命令（MIR14）の最小IRでInterpreter/VM/JIT/AOT/GUIを目指してきた。本稿ではPhase‑15における設計判断として、MIR側のPHI生成を停止（PHI‑off, エッジコピー合流）し、PHI形成をLLVMハーネス側に委譲する方針を採用した経緯と効果を報告する。現在の評価範囲はPyVM（意味論リファレンス）とLLVM/llvmlite（AOT/EXEハーネス）に限定し、両者のパリティおよびLLVM側の性能・安定性を中心に示す。

> 更新メモ（2025-09-26）: Phase‑15 では PHI-on（MIR14）が既定に復帰したよ。この資料はPHI-off方針をアーカイブとして残しているよ。現行のポリシーは `docs/reference/mir/phi_policy.md` を参照してね。

## 1. はじめに
最小IRで多様な実行形態を統一する挑戦では、IRの表現力と実装コストの均衡が鍵となる。Nyashは命令の削減（27→13→14）とAPI統一（BoxCall）でIRを簡素に保ちつつ、評価基準をPyVM意味論とLLVM生成物に絞ることで、開発・検証速度を高めた。

## 2. MIR14の設計原則
- 命令セット: const/binop/unary/compare/branch/jump/ret/phi/call/boxcall/typeop/arrayget/arrayset/cast/…（詳細は `docs/reference/mir/INSTRUCTION_SET.md`）
- Box中心: 呼び出しとABI境界はBoxCall/PluginInvokeに一本化
- 可観測性: JSON v0、IRダンプ、PHI配線トレースを整備
- 非対象（現段階）: MIR側の最適PHI配置の探索・検証（責務をLLVMへ移譲）

## 3. PHIオフ方針とLLVM側合成
- 方針: MIRはPHIを出さず、分岐合流は「合流先で参照される値」を各前任ブロックからエッジコピーで集約
- LLVM: ブロック先頭にPHIを形成（typed incoming）、if‑merge前宣言等で安定性向上
- 不変条件（LLVM側）: PHIはブロック先頭にのみ配置、incomingは型付き `i64 <v>, %bb`（詳細: `docs/reference/mir/phi_invariants.md`）
- トグル:
  - 既定: `NYASH_MIR_NO_PHI=0`（PHI-on）
  - レガシー再現: `NYASH_MIR_NO_PHI=1`（PHI-off） + `NYASH_VERIFY_ALLOW_NO_PHI=1`

## 4. 実装概要（評価対象）
- PyVM: JSON v0→MIR実行の意味論基準。短絡やtruthy規約の基準線
- LLVM/llvmlite: AOT/EXE生成・IRダンプ・PHI合成の実働ライン
- 実行例:
  - LLVMハーネス: `NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_DUMP_IR=tmp/nyash.ll ...`
  - PHIトレース: `NYASH_LLVM_TRACE_PHI=1`

## 5. 評価計画
- パリティ: PyVM vs LLVMの出力一致（代表スモーク）
- 性能: LLVMの実行時間/起動時間/メモリ
- 安定性: PHIトレース整合、空PHI未発生の確認
- 再現コマンド:
  - parity: `tools/parity.sh --lhs pyvm --rhs llvmlite apps/tests/CASE.nyash`
  - build paper (PDF/TeX): `tools/papers/build.sh a-jp`

## 6. 関連研究
最小IR設計（LLVM/MLIR等）と、多層実行（Truffle/Graal）に対する立ち位置を簡潔に比較。Nyashは「IRは最小・PHIは生成系に委譲」という分担で整合を取る点に新規性。

## 7. 結論
MIR14の簡素化とPHI委譲により、設計・検証・配布ラインを細く強く維持できた。今後はLoopForm（MIR17）や実行器の拡張を、PyVM/LLVMの二系統基準で段階的に進める。

### 謝辞
AI協働（ChatGPT/Gemini）とコミュニティ貢献に感謝する。

### 付録
- 主要トグル: `NYASH_MIR_NO_PHI`, `NYASH_LLVM_USE_HARNESS`, `NYASH_LLVM_TRACE_PHI`
- 仕様参照: `docs/reference/mir/INSTRUCTION_SET.md`, `docs/reference/mir/phi_invariants.md`

### キーワード
ミニマルIR, SSA, PHI合成, LLVM, PyVM, BoxCall, 統一実行

# Mini‑VM 構築ロードマップ（Self‑Hosting 足場）

Status: active (Stage B → C 準備)

目的
- Nyashスクリプト製の極小VM（Mini‑VM）を段階的に整備し、PyVM依存を徐々に薄める。
- まずは「JSON v0 → 実行（print/if/loopの最小）」の芯を安定化し、自己ホストの足場にする。

原則
- 小さく進める（段階ゲート、既定OFF）。
- 既存Runner/マクロ/CIへの影響を最小化（導線はenvで明示）。
- まずは正しさ・可読性を優先。性能は後段で最適化。

Stages（概要）
- Stage A（完了）
  - 文字列スキャンで整数抽出→print、if（リテラル条件）の最小到達。
  - サンプル: `apps/selfhost-vm/mini_vm*.nyash`
  - スモーク: `tools/test/smoke/selfhost/mini_vm_*`
- Stage B（進行中）
  - stdinローダ（`NYASH_MINIVM_READ_STDIN=1`）[実装済]
  - JSON v0 ローダの最小強化（Print(Literal/FunctionCall)、BinaryOp("+")の最小）[実装中]
- Stage C（次）
  - 最小命令の芯：const / compare / branch / ret（loopの芯に直結）
  - binop(int+int) を本加算に変更（現状は簡易出力）
  - if/loop の代表ケースを Mini‑VM で実行（PyVM と出力一致）
- Stage D（整備）
  - 解析の健全化：最小トークナイザ/カーソル Box 抽出、JSON 走査の責務分離
  - 観測/安全：`NYASH_MINIVM_DEBUG=1`、最大ステップ、入力検証

受け入れ基準
- A: print/ifサンプルのスモーク常時緑
- B: stdin/argv経由のJSON供給で Print(Literal/FunctionCall)、BinaryOp("+") が正しく動作
- C: if/loop の簡易ケースが Mini‑VM で実行可能（PyVMと出力一致）
- D: 代表スモークが既定で安定（デバッグON時のみ追加出力）

実行・導線
- PyVM経由（既定）: `NYASH_VM_USE_PY=1` で Runner が MIR(JSON)→PyVM へ委譲
- Mini‑VM入力: `NYASH_MINIVM_READ_STDIN=1` で標準入力を `NYASH_SCRIPT_ARGS_JSON` に注入
- サンプル実行: `bash tools/test/smoke/selfhost/mini_vm_stdin_loader_smoke.sh`

関連
- 現在の短期タスクと進捗: `CURRENT_TASK.md` の「Mini‑VM 構築ロードマップ（整理）」

---

開発順序（迷わないための具体ステップ）

Now（今すぐ）
- compare の厳密化（<, == を先に完成 → その後 <=, >, >=, != を追加）
- binop(int+int) を本加算に修正（文字列→整数化→加算→文字列化）
- スモーク追加（各1本ずつ）：binop / compare / if（Mini‑VM 版）

Next（次の小粒）
- 最小トークナイザ/カーソル Box 抽出（index/substring を段階置換）
- FunctionCall の引数2個の最小対応（echo(a,b)→連結）とスモーク

Later（後で一気に）
- loop の芯（branch/jump/ret を活用）と代表スモーク
- ランナーの薄いFacade（PyVM/Interpreter 切替を関数で吸収。巨大Trait導入は後回し）

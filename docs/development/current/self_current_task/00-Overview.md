# Self Current Task — Overview (main)

目的
- main ブランチで Core‑13（MIR13）前提の制御フローを整備し、LLVM/Cranelift(EXE)/VM に綺麗に降ろす土台を完成させる。
- 箱言語の既存命令セット（Branch/Jump/Phi 他）を活かし、continue/break を新命令なしで表現する。

前提と指針
- MIR13 前提（純化モードを含む）。
- ループは canonical 形（preheader → header → body → latch → header、exit は単一）。
- continue/break は分岐のみで表現（continue→ヘッダ/ラッチ、break→単一 exit）。
- Verifier（支配関係/SSA）緑を最優先。post‑terminated 後の emit 禁止、合流点を明確化。

スコープ外
- 新規 MIR 命令の追加。
- try/finally と continue/break の相互作用（次段）。


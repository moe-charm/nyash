# Self Current Task — Now (main)

2025‑09‑08：現状と直近タスク
- LLVM 側 P0 完了（BitOps/Array/Echo/Map 緑）。VInvoke(by‑name/by‑id vector) は戻り値マッピングの暫定課題を確認中（Decisions 参照）。
- selfhosting-dev の作業を main に順次取り込み。VM/MIR 基盤は main で先に整える方針。

直近タスク（優先）
1) continue/break の lowering（Builder 修正のみで表現）
   - ループ文脈スタック {head, exit} を導入。
   - continue に遭遇 → head（または latch）へ br を emit し終端。
   - break に遭遇 → exit へ br を emit し終端。
   - post‑terminated 後に emit しない制御を徹底。
2) ループ CFG の厳密化
   - 単一 exit ブロックの徹底。
   - Phi はヘッダでキャリー変数を合流（SSA/支配関係が崩れない形）。
3) 検証とスモーク
   - Verifier 緑（dominance/SSA）。
   - VM のループ代表（単純/ネスト/早期継続・脱出）。
   - LLVM/Cranelift EXE に綺麗に降りる（br/phi ベースで問題なし）。

代表コマンド（例）
- ビルド: `cargo build --release`
- LLVM smoke: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) NYASH_LLVM_BITOPS_SMOKE=1 ./tools/llvm_smoke.sh release`
- VInvoke 調査: `NYASH_LLVM_VINVOKE_TRACE=1 NYASH_LLVM_VINVOKE_SMOKE=1 ./tools/llvm_smoke.sh release`


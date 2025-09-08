# Self Current Task — Now (llvm)

2025‑09‑08：現状と直近タスク
- P0 完了（BitOps/Array/Echo/Map 緑）。
- VInvoke(by‑name/by‑id vector) で戻り値マッピングの課題（ret_tag=3 でも 0 になる）。

直近タスク
1) VInvoke 戻り値の短期特例：既知メソッド（例: MapBox.get）の整数返りは i64 として保持。
2) by‑id vector（`nyash.plugin.invoke_tagged_v_i64`）も同様に統一。
3) 受け入れ：
   - `NYASH_LLVM_VINVOKE_SMOKE=1` → `VInvokeRc: 42`
   - `NYASH_LLVM_VINVOKE_RET_SMOKE=1` → `Result: 42`

代表コマンド
- `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) NYASH_LLVM_BITOPS_SMOKE=1 ./tools/llvm_smoke.sh release`
- `NYASH_LLVM_VINVOKE_TRACE=1 NYASH_LLVM_VINVOKE_SMOKE=1 ./tools/llvm_smoke.sh release`


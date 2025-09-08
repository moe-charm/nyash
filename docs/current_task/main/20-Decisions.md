# Self Current Task — Decisions (main)

2025‑09‑08
- ループ制御は既存命令（Branch/Jump/Phi）で表現し、新命令は導入しない。
- Builder に loop_ctx（{head, exit}）を導入し、continue/break を分岐で降ろす。
- Verifier の支配関係/SSA を崩さないよう、単一 exit と post‑terminated 後の emit 禁止を徹底。
- VInvoke（vector 経路）の戻り値は、短期は「既知メソッドの整数返り」を特例扱いで保持し、
  中期は nyash.toml の戻り型ヒント or NyRT シムの期待フラグで正道化。


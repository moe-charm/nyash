# Self Current Task — Decisions (llvm)

2025‑09‑08
- VInvoke（vector 経路）の戻り値：短期は既知メソッドの整数返りを i64 として保持（Unknown/Box でも整数扱いにする特例）。
- 中期は正道へ：
  - nyash.toml に戻り型ヒント（primitive/handle/f64/bool）を追加し codegen に供給、または
  - NyRT シムに「期待戻り形式」フラグを追加し codegen から通知。


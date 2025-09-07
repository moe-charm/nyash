# ChatGPT5 初期分析 - MIR引数配線問題の特定

## 分析内容（ChatGPT5より）

### 問題の核心
Registryの緩和: f64署名に対してi64実引数を許容（i64→f64コアーションOK）。
- native_f64=1時は、署名一致でmath.*のf64スタブへ薄接続（JIT内でf64を積んで型付き呼出し）。
- allow観測サンプルを追加:
  - 浮動小数: examples/jit_hostcall_math_sin_allow_float.nyash
  - 整数→f64強制: examples/jit_hostcall_math_sin_allow_int_coerce.nyash
- ただし現状のMIR経路だと、math.* のBoxCallに引数が載っておらず、両方ともsig_mismatchになる可能性が高いことを今回のログが示しています。

### 次の一手（おすすめ）
- 本丸: MIRでのBoxCall引数配線（MathBox系）
  - MIR Builder/Compilerのメソッド呼出しで、receiverとは別にcall引数が I::BoxCall { args: [...] } に正しく入るよう修正
  - これが入れば、今回のRegistry/Lowerの署名判定がそのまま活き、allow(sig_ok)→native_f64でJIT実行まで通ります
- 代替（デバッグ強化のみ）
  - 署名チェックイベントに argc_observed を入れて観測性を上げる（どの時点で0になっているかの特定を容易に）

このままMIR側の修正に入ってよければ、該当箇所を特定して最小差分でBoxCallのargsを正しく生成するパッチを出しますか？それとも先にイベントの詳細（argc_observed）を足して、手元ログの粒度を上げますか？

## 分析の鋭さ

1. **問題を一撃で特定**
   - 「MIR経路でmath.*のBoxCallに引数が載っていない」
   - これがすべての症状（sig_mismatch）の根本原因

2. **解決策の明確さ**
   - MIR Builder/Compilerの修正が本丸
   - 引数配線さえ直せば、既存のRegistry/Lowerがそのまま活きる

3. **観測可能性への配慮**
   - argc_observedによるデバッグ強化も提案
   - 問題の可視化を重視

この初期分析が、その後の俯瞰AI→実装AIの完璧な連携の起点となった。
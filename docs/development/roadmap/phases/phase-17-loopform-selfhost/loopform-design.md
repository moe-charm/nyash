# LoopForm Design — Macro‑Driven Loop Normalization

Goal
- ループで発生する「変数キャリア（loop‑carried values）」を Nyash のユーザーマクロで前段正規化し、MIR/LLVM側は素直に最適化可能な形にする。

Key Idea
- ループ状態（複数変数）をタプル（または専用Box）に束ねて“1個の搬送値（carrier）”として扱う。
- ループのヘッダで φ は常に“1つのタプル”にだけ付与される → PHIグルーピングの不変が自動で満たされる。

Normalization Pattern（whileのMVP）
```
// input
// i=0; sum=0; while (i<n) { sum = sum + a[i]; i = i + 1 }

// expanded
let __car0 = (i, sum);
head:
  let (i, sum) = __car_phi;
  if !(i < n) goto exit;
  let __car1 = (i + 1, sum + a[i]);
  __car_phi = φ(__car0, __car1);
  goto head;
exit:
  let (i, sum) = __car_phi;
```

Break/Continue（Week2）
- continue: “次のキャリア”を構築して head へ遷移。
- break: 現キャリアで exit へ遷移。
- ネスト時: 内側ループのcarrierと外側スコープの分解を明示する（名称はgensym）。

for/foreach（Week3）
- for (init; cond; step) は init→while(cond){ body; step } へ前処理後、同様に正規化。
- foreach は IteratorBox（MVP）経由の while 形式へ前処理。

Constraints / Notes
- try/finally/throw との相互作用はMVPでは未対応（将来ガイドで制約を明記）。
- キャリア自動抽出: 本体で再代入される変数集合を候補とし、ループ外で参照されるものを優先収集。
- 衛生: MacroCtx.gensym で __car_phi/__carK などの一意名を生成。

Integration
- ユーザーマクロ: `apps/macros/examples/loop_normalize_macro.nyash`
- 事前展開: selfhost‑preexpand auto（PyVM限定）で適用
- 検証: macro‑golden + LLVM PHI健全性スモーク（空PHIなし/先頭グループ化）

Future Work（先の先）
- MIRに LoopHeader/LoopLatch/LoopContinue/LoopExit の4命令を導入した正規形を検討（最適化/解析の高速化）。
- Boxベースの LoopState/Carrier の型体系を整理（型推論との接続）。


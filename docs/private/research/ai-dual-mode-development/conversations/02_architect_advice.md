# 俯瞰AI（Architect ChatGPT5）のアドバイス

## コンテキスト

にゃー：「これがアドバイザーchatgpt5の内容」

## 俯瞰AIの分析

わかった。ここは**MIRのBoxCall引数配線を直すのが本丸**。観測の強化は"ついで"に入れておく。
理由：Registry/Policy/Lower はもう利く。今詰まってるのは **MIR→Lower に args が落ちてない**一点。ここを刺すと `native_f64=1` で `math.*` が即通る。

---

# 決定

1. 先に **MIR側で BoxCall の引数を正しく生成**（最小差分）。
2. 併せて **Events に argc/arg_types を追加**（デバッグ保険）。

---

## ① MIR修正（最小パッチ雛形）

### IR定義（確認）

```rust
// ir.rs
pub enum Inst {
    // 既存:
    // ...
    BoxCall { target: Sym, recv: Option<Value>, args: Vec<Value>, dst: Option<Value> }, // ← args 必須
}
```

### MIR Builder（式の降ろし先を1点集中）

```rust
// src/mir/builder/expr.rs
fn lower_call_expr(&mut self, e: &CallExpr) -> Value {
    // 1) 受け手（recv）とターゲット解決（math.* は recv=None, target="math.sin" 等）
    let (target, recv, arg_exprs) = match e.kind {
        CallKind::Method { ref recv, ref name, ref args } => {
            (Sym::from_method(name), Some(self.lower_expr(recv)), args)
        }
        CallKind::Func   { ref path,  ref args } => {
            (Sym::from_path(path), None, args)
        }
    };

    // 2) 引数を **必ず** Vec<Value> に積む（ここが今回の本丸）
    let mut argv = Vec::with_capacity(arg_exprs.len());
    for a in arg_exprs {
        argv.push(self.lower_expr(a));
    }

    // 3) 結果レジスタを用意して BoxCall 生成
    let dst = self.new_temp();
    self.push(Inst::BoxCall { target, recv, args: argv, dst: Some(dst) });
    dst
}
```

## ② Lower（Coercion と Registry/Policy/Events の順序）

```rust
// src/jit/lower/builder.rs
fn lower_boxcall(&mut self, inst: &Inst) -> LowerResult {
    // Registry で署名確認（i64→f64 許容）
    match self.registry.check_with_coercion(target, &observed, self.cfg.native_f64) {
        Check::AllowCoerceF64(sig, mask) => {
            // mask[i]==true なら i64→f64 にキャスト挿入
            let mut coerced = Vec::with_capacity(args.len());
            for (i, &v) in args.iter().enumerate() {
                coerced.push(if mask[i] { self.cast_i64_to_f64(v) } else { v });
            }
            JitEventsBox::hostcall(func, target.as_str(), "ro", "allow(coerce_f64)");
            self.emit_hostcall_ro(target, recv, &coerced, dst, sig)
        }
        // ...
    }
}
```

## 分析の特徴

1. **問題の本質を一行で表現**
   - 「MIR→Lower に args が落ちてない一点」

2. **解決策の明確さ**
   - 最小差分での修正方法を具体的に提示
   - コード例で実装イメージを共有

3. **副次的改善も忘れない**
   - 「観測の強化は"ついで"に」
   - argc/arg_types追加でデバッグ保険
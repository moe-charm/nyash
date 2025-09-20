Exception Handling — Postfix catch / cleanup (Stage‑3)

Summary
- Nyash adopts a flatter, postfix-first exception style:
  - try is deprecated. Use postfix `catch` and `cleanup` instead.
  - `catch` = handle exceptions from the immediately preceding expression/call.
  - `cleanup` = always-run finalization (formerly finally), regardless of success or failure.
- This matches the language’s scope unification and keeps blocks shallow and readable.

Status
- Phase 1: normalization sugar（既存）
  - `NYASH_CATCH_NEW=1` でコア正規化パスが有効化。
  - 後置フォームは内部 `TryCatch` AST に変換され、既存経路で降下。
  - 実行時コストはゼロ（意味論不変）。
- Phase 2（実装済み・Stage‑3ゲート）
  - パーサが式レベルの後置 `catch/cleanup` を直接受理。
  - ゲート: `NYASH_PARSER_STAGE3=1`
  - 糖衣正規化はそのまま併存（関数糖衣専用）。キーワード直受理と二重適用はしない設計。

Syntax (postfix)
- Expression-level postfix handlers (Stage‑3):
  - `expr catch(Type e) { /* handle */ }`
  - `expr catch { /* handle (no variable) */ }`
  - `expr cleanup { /* always-run */ }`
  - Combine: `expr catch(Type e){...} cleanup{...}`
- Method/function calls are just expressions, so postfix applies:
  - `call(arg1, arg2) catch(Error e) { log(e) }`
  - `obj.method(x) cleanup { obj.release() }`

Precedence and chaining
- Postfix `catch`/`cleanup` binds to the immediately preceding expression (call/chain result), not to the whole statement.
- For long chains, we recommend parentheses to make intent explicit:
  - `(obj.m1().m2()) catch { ... }`
  - `f(a, b) catch { ... } cleanup { ... }`
- Parser rule (Stage‑3): postfix attaches once at the end of a call/chain and stops further chaining on that expression.

Diagram (conceptual)
```
// before (parse)
obj.m1().m2() catch { H } cleanup { C }

// precedence (binding)
obj.m1().[ m2()  ↖ binds to this call ] catch { H } cleanup { C }

// normalization (conceptual AST)
TryCatch {
  try:   [ obj.m1().m2() ],
  catch: [ (type:Any, var:None) -> H ],
  finally: [ C ]
}
```

Normalization (Phase 1)
- With `NYASH_CATCH_NEW=1`, postfix sugar is transformed into legacy `TryCatch` AST:
  - `EXPR catch(T e){B}` → `TryCatch { try_body:[EXPR], catch:[(T,e,B)], finally:None }`
  - `EXPR cleanup {B}` → `TryCatch { try_body:[EXPR], catch:[], finally:Some(B) }`
  - Multiple `catch` are ordered top-to-bottom; first matching type handles the error.
  - Combined `catch ... cleanup ...` expands to a single `TryCatch` with both blocks.
- Lowering uses the existing builder (`cf_try_catch`) which already supports cleanup semantics.

Semantics
- catch handles exceptions from the immediately preceding expression only.
- cleanup is always executed regardless of success/failure (formerly finally).
- Multiple catch blocks match by type in order; the first match is taken.
- In loops, `break/continue` cooperate with cleanup: cleanup is run before leaving the scope.

Migration notes
- try is deprecated: prefer postfix `catch/cleanup`.
- Member-level handlers (computed/once/birth_once/method) keep allowing postfix `catch/cleanup` (Stage‑3), unchanged.
- Parser acceptance of postfix at expression level will land in Phase 2; until then use the gate for normalization.

Examples
```
// Postfix catch on a call
do_work() catch(Error e) { env.console.log("error: " + e) }

// Always-run cleanup
open_file(path) cleanup { env.console.log("closed") }

// Combined
connect(url)
  catch(NetworkError e) { env.console.warn(e) }
  cleanup { env.console.log("done") }

// Stage‑3 parser gate quick smoke (direct acceptance)
//   NYASH_PARSER_STAGE3=1 ./target/release/nyash --backend vm \
//     apps/tests/macro/exception/expr_postfix_direct.nyash
```

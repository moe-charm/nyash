# Stage‑3 Exceptions Guide (MVP)

Status: Experimental behind gates. Designed to be small, predictable, and PHI‑off friendly.

Goals
- Keep the runtime simple and portable (VM/PyVM/LLVM/JIT) by avoiding backend‑specific unwinding.
- Express exceptions via structured control flow (blocks + jumps) in the JSON v0 Bridge.
- Prefer one `catch` per `try` (single catch), and branch inside the catch for different cases.

Enable
- Parser (Rust): `NYASH_PARSER_STAGE3=1`
- Bridge (Result‑mode lowering): `NYASH_TRY_RESULT_MODE=1`
- Block‑postfix gate: `NYASH_BLOCK_CATCH=1` (or `NYASH_PARSER_STAGE3=1`)

## Block‑Postfix Catch（try を無くす設計: Phase 15.5）

Motivation
- 深いネストを避け、スコープ＝例外境界を明示。Result‑mode/ThrowCtx/単一catch と強整合。

Syntax
- `{ body } catch (e) { handler } [cleanup { … }]`
- `{ body } cleanup { … }`（catch 省略可）

Policy
- 単一 catch（分岐は catch 内で）。パラメータ形 `(Type x)|(x)|()`。
- “同じ階層” のブロック内 throw のみ到達。外への自然伝播はしない（必要なら外側ブロックも後置 catch を付与）。
- MVP 静的検査: 直後に catch のない独立ブロックでの「直接の throw 文」はビルドエラー。
- 適用対象: 独立ブロック文に限定（if/else/loop ブロック直後には付与しない）。

Lowering（Result‑mode）
- Parser が後置 catch/cleanup を `TryCatch` に畳み込み（try_body=直前ブロック）。
- Bridge は既存の Result‑mode を使用：ThrowCtx によりネスト throw を単一 catch に集約、PHI‑off 合流（edge‑copy）。

Run examples
- JSON v0 → Bridge → PyVM: `bash tools/test/smoke/bridge/try_result_mode.sh`
  - Includes `block_postfix_catch.json` to confirm single‑catch + cleanup path.
  - Env: `NYASH_TRY_RESULT_MODE=1` is set by the script.


Examples
```nyash
// OK: 独立ブロック + 後置 catch
{
  do_something_dangerous()
} catch (e) {
  print("問題発生にゃ！")
} cleanup {
  cleanup()
}

// NG: if/else のブロック直後には付けない（必要なら独立ブロックで包む）
if cond {
  do_a()
} else {
  do_b()
}
// …ここに catch は付与しない

// 代わりに:
{
  if cond { do_a() } else { do_b() }
} catch (e) { handle(e) }
```

Notes
- 旧 `try { … } catch { … }` は段階的に非推奨化し、後置 catch を推奨。
- 静的検査の厳密化（関数の throws 効果ビット）は将来の拡張項目。

## Method‑Level Postfix Catch/Cleanup（Phase 15.6, planned）

Motivation
- 例外境界をメソッド定義レベルに持ち上げ、呼び出し側の try/catch ボイラープレートを削減する。

Syntax（gated）
- `method name(params) { body } [catch (e) { handler }] [cleanup { … }]`

Gate
- `NYASH_METHOD_CATCH=1`（または `NYASH_PARSER_STAGE3=1` と同梱）

Policy（MVP）
- 単一 catch。順序は `catch` → `cleanup` のみ許可。
- 近接優先: ブロック後置の catch があればそれが優先。次にメソッドレベル、最後に呼出し側での境界へ伝播（将来の効果型導入まで単純規則）。

Lowering（Result‑mode）
- メソッド本体の `block` を `TryCatch` に正規化して既存の Result‑mode/ThrowCtx/PHI‑off 降下を再利用する。

Examples
```nyash
box SafeBox {
  method query(sql) {
    return me.db.exec(sql)
  } catch (e) {
    me.log(e)
    return null
  } cleanup {
    me.db.close()
  }
}
```

Future
- ブロック先行・メソッド後置 `{ body } method name(..) [catch..] [cleanup..]` は Phase 16.x にて検討。

Syntax (accepted)
- `try { … } catch [(Type x)|(x)|()] { … } cleanup { … }`
- `throw <expr>`

Policy
- Single catch only (MVP): Parser may accept multiple, but Bridge uses the first one. Prefer branching inside the catch body.
- The thrown value is any Nyash value (string label or ErrorBox recommended). Match inside the catch to handle variants.

Branching patterns inside catch
- Label string (lightweight):
  ```nyash
  try {
    if cond { throw "Timeout" } else { throw "IO" }
  } catch (e) {
    if e == "Timeout" { handleTimeout() }
    else if e == "IO" { handleIO() }
    else { handleDefault() }
  } cleanup { cleanup() }
  ```

- peek (structured):
  ```nyash
  try {
    throw "IO"
  } catch (e) {
    peek e {
      "Timeout" => onTimeout(),
      "IO"      => onIO(),
      else      => onDefault(),
    }
  }
  ```

- ErrorBox (future‑friendly):
  ```nyash
  // Future: ErrorBox.kind()/message() helpers
  try { throw new ErrorBox("Timeout") } catch (e) {
    if e.kind() == "Timeout" { onTimeout() } else { onDefault() }
  }
  ```
  Note: ErrorBox helpers may be introduced later; until then prefer label strings or manual destructuring.

Lowering strategy (Bridge, Result‑mode)
- No MIR Throw/Catch. Instead, the Bridge uses blocks and jumps:
  1) try body is lowered normally.
  2) When a throw is encountered during try lowering, the current block jumps to the catch block and records `(pred, value)`.
  3) At the start of the catch, the parameter (if present) is bound via PHI (PHI‑off emits edge‑copies).
  4) cleanup always runs; variables merge at cleanup/exit using edge‑copy rules.
- Nested `throw` anywhere inside try is routed to the same catch via a thread‑local ThrowCtx.

Env flags
- `NYASH_PARSER_STAGE3=1`: accept syntax in Rust parser (default OFF).
- `NYASH_TRY_RESULT_MODE=1`: enable structured lowering (default OFF).
- Legacy: `NYASH_BRIDGE_TRY_ENABLE`, `NYASH_BRIDGE_THROW_ENABLE` remain but are bypassed in Result‑mode.

Testing
- Selfhost acceptance (JSON v0 → Bridge → PyVM): `tools/selfhost_stage3_accept_smoke.sh`
- Curated LLVM (non‑throw path confirmation): `tools/smokes/curated_llvm_stage3.sh`

Notes
- Single catch policy simplifies control flow and keeps PHI‑off merging consistent with short‑circuit/if.
- Multiple catch forms may be revisited later; prefer branching in catch for clarity and stability.

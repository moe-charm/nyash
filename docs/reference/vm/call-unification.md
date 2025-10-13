# MirCall Unified Route

目的
- Call/BoxCall/ExternCall/NewBox を統一し、`MirInstruction::Call`（= MirCall）から単一路で実行する。

呼び出し種別（Callee）
- Global(String)
- ModuleFunction(String)
- Method { box_name, method, receiver, certainty }
- Constructor { box_type }
- Closure { … }
- Value(ValueId)
- Extern(String)

ディスパッチ（実行器）
- Method → `runtime::method_router_box::route`（plugin → builtin の順で委譲）
- ModuleFunction → 直接ディスパッチ（テーブル）
- Global/Constructor/Closure/Value → 直接
- Extern → legacy のみ（plugin-only では明示エラー）

正規化（Normalize: BoxCall → Call）
- 安全ケースのみ段階導入（2025-10）
  - `methodRef.call([])` → `Callee::Method(..., args=[])`
  - `methodRef.call(argv)`（argv を同一ブロックの ArrayBox push 連鎖から再構成できる場合）
  - 由来: `Method(methodRef)` / `ModuleFunction("<Box>.methodRef/2")`（Array/Map/String を対象）
- 互換: BoxCall 起点 `methodRef` は Array のみ（型不明箇所のため保守的）
- 安全ホワイトリストの箱化
  - `normalize.rs` 内の `is_safe_core_method(&str,&str)` が唯一の判定点（Array/Map/String の安全APIを列挙）
  - ModuleFunction→Method と BoxCall→Method の両経路で同判定を使用（ドリフト防止）

エラー方針（抜粋）
- HostHandleRouter（early path）
  - 未知ハンドル: -1
  - 受け型不一致: -11
  - TLVデコード失敗: -13
  - 返却型不一致: -14
- plugin-only: Extern は明示エラー（diagnostic を安定化）
   - plugin-only では `Callee::Extern` は使用不可。実行時には明示エラー（例: "extern calls disabled (legacy-only)") を返し、Fail‑Fast とする。

実装ポインタ
- `src/mir/definitions/call_unified.rs`（Callee/MirCall）
- `src/mir/optimizer_passes/normalize.rs`（BoxCall→Call 降格）
- `src/runtime/method_router_box/{plugin.rs,builtin.rs,mod.rs}`（Method 統一）
- `src/runtime/host_handle_router/mod.rs`（HostHandle slots）

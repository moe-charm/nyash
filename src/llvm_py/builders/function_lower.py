from typing import Dict, Any


def lower_function(builder, func_data: Dict[str, Any]):
    """Lower a single MIR function to LLVM IR using the given builder context.

    This helper is a thin wrapper that delegates to the NyashLLVMBuilder's
    existing methods/attributes, enabling gradual file decomposition without
    changing semantics.
    """
    name = func_data.get("name", "unknown")
    builder.current_function_name = name

    import re
    params = func_data.get("params", [])
    blocks = func_data.get("blocks", [])

    # Determine function signature
    if name == "ny_main":
        func_ty = builder.i32.func_type([])
    else:
        m = re.search(r"/(\d+)$", name)
        arity = int(m.group(1)) if m else len(params)
        param_types = [builder.i64] * arity
        func_ty = builder.i64.func_type(param_types)

    try:
        builder.vmap.clear()
    except Exception:
        builder.vmap = {}
    builder.bb_map = {}
    builder.preds = {}
    builder.block_end_values = {}
    builder.def_blocks = {}
    builder.predeclared_ret_phis = {}

    # Ensure function exists or create one
    fn = None
    for f in builder.module.functions:
        if f.name == name:
            fn = f
            break
    if fn is None:
        from llvmlite import ir
        fn = ir.Function(builder.module, func_ty, name=name)

    # Create all basic blocks first
    from llvmlite import ir
    block_by_id = {}
    for b in blocks:
        bbid = int(b.get("id", 0))
        bb = fn.append_basic_block(name=f"bb{bbid}")
        block_by_id[bbid] = bb
        builder.bb_map[bbid] = bb

    # Predeclare ret PHIs if needed (if-merge prepass)
    from prepass.if_merge import plan_ret_phi_predeclare
    plan = plan_ret_phi_predeclare(block_by_id)
    if plan:
        if not hasattr(builder, 'block_phi_incomings') or builder.block_phi_incomings is None:
            builder.block_phi_incomings = {}
        for (bbid, pairs) in plan.items():
            for (ret_vid, preds_list) in pairs.items():
                builder.block_phi_incomings.setdefault(int(bbid), {}).setdefault(int(ret_vid), [])
                builder.block_phi_incomings[int(bbid)][int(ret_vid)] = [(int(p), int(ret_vid)) for p in preds_list]

    # Lower instructions per block
    from llvmlite.ir import IRBuilder
    from instructions import dispatcher  # if exists; else inline lowerers
    for b in blocks:
        bbid = int(b.get("id", 0))
        bb = block_by_id[bbid]
        builder_bb = IRBuilder(bb)
        builder.resolver.attach_function_and_block(fn, bb)
        insts = b.get("insts", [])
        for inst in insts:
            op = inst.get("op")
            # Delegate to existing NyashLLVMBuilder method for now
            builder.lower_instruction(op, inst, builder_bb)

    # Finalize PHIs after the function is fully lowered
    from phi_wiring import finalize_phis as _finalize_phis
    _finalize_phis(builder)


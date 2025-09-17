"""
Lowering helpers for while-control flow (regular structured)
"""

from typing import List, Dict, Any
import llvmlite.ir as ir
from instructions.safepoint import insert_automatic_safepoint

def lower_while_regular(
    builder: ir.IRBuilder,
    func: ir.Function,
    cond_vid: int,
    body_insts: List[Dict[str, Any]],
    loop_id: int,
    vmap: Dict[int, ir.Value],
    bb_map: Dict[int, ir.Block],
    resolver,
    preds,
    block_end_values,
):
    """Create a minimal while in IR: cond -> body -> cond, with exit.
    The body instructions are lowered using the caller's dispatcher.
    """
    i1 = ir.IntType(1)
    i64 = ir.IntType(64)

    # Create basic blocks: cond -> body -> cond, and exit
    cond_bb = func.append_basic_block(name=f"while{loop_id}_cond")
    body_bb = func.append_basic_block(name=f"while{loop_id}_body")
    exit_bb = func.append_basic_block(name=f"while{loop_id}_exit")

    # Jump from current to cond
    builder.branch(cond_bb)

    # Cond block
    cbuild = ir.IRBuilder(cond_bb)
    try:
        # Resolve against the condition block to localize dominance
        cond_val = resolver.resolve_i64(cond_vid, cond_bb, preds, block_end_values, vmap, bb_map)
    except Exception:
        cond_val = vmap.get(cond_vid)
    if cond_val is None:
        cond_val = ir.Constant(i1, 0)
    # Normalize to i1
    if hasattr(cond_val, 'type'):
        if isinstance(cond_val.type, ir.IntType) and cond_val.type.width == 64:
            zero64 = ir.Constant(i64, 0)
            cond_val = cbuild.icmp_unsigned('!=', cond_val, zero64, name="while_cond_i1")
        elif isinstance(cond_val.type, ir.PointerType):
            nullp = ir.Constant(cond_val.type, None)
            cond_val = cbuild.icmp_unsigned('!=', cond_val, nullp, name="while_cond_p1")
        elif isinstance(cond_val.type, ir.IntType) and cond_val.type.width == 1:
            # already i1
            pass
        else:
            # Fallback: treat as false
            cond_val = ir.Constant(i1, 0)
    else:
        cond_val = ir.Constant(i1, 0)

    # Insert a safepoint at loop header to allow cooperative GC
    try:
        import os
        if os.environ.get('NYASH_LLVM_AUTO_SAFEPOINT', '1') == '1':
            insert_automatic_safepoint(cbuild, builder.block.parent.module, "loop_header")
    except Exception:
        pass
    cbuild.cbranch(cond_val, body_bb, exit_bb)

    # Body block
    bbuild = ir.IRBuilder(body_bb)
    # The caller must provide a dispatcher to lower body_insts; do a simple inline here.
    # We expect the caller to have a method lower_instruction(builder, inst, func).
    lower_instruction = getattr(resolver, '_owner_lower_instruction', None)
    if lower_instruction is None:
        raise RuntimeError('resolver._owner_lower_instruction not set (needs NyashLLVMBuilder.lower_instruction)')
    for sub in body_insts:
        if bbuild.block.terminator is not None:
            cont = func.append_basic_block(name=f"cont_bb_{bbuild.block.name}")
            bbuild.position_at_end(cont)
        lower_instruction(bbuild, sub, func)
    # Ensure terminator: if not terminated, branch back to cond
    if bbuild.block.terminator is None:
        bbuild.branch(cond_bb)

    # Continue at exit
    builder.position_at_end(exit_bb)

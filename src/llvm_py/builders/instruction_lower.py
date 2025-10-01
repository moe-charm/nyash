from typing import Dict, Any
from llvmlite import ir
from trace import debug as trace_debug

# Import instruction context (箱理論)
from builders.instruction_context import InstructionContext

# Import instruction handlers
from instructions.const import lower_const
from instructions.binop import lower_binop
from instructions.compare import lower_compare
from instructions.unop import lower_unop
from instructions.controlflow.jump import lower_jump
from instructions.controlflow.branch import lower_branch
from instructions.ret import lower_return
from instructions.copy import lower_copy
from instructions.call import lower_call
from instructions.boxcall import lower_boxcall
from instructions.externcall import lower_externcall
from instructions.typeop import lower_typeop
from instructions.newbox import lower_newbox
from instructions.safepoint import lower_safepoint
from instructions.barrier import lower_barrier
from instructions.loopform import lower_while_loopform
from instructions.controlflow.while_ import lower_while_regular
from instructions.mir_call import lower_mir_call  # New unified handler
from instructions.phi import lower_phi  # PHI instruction for SSA merging


def lower_instruction(owner, builder: ir.IRBuilder, inst: Dict[str, Any], func: ir.Function):
    """Dispatch a single MIR instruction to appropriate lowering helper.

    owner is the NyashLLVMBuilder instance to access module, resolver, maps, and ctx.
    """
    op = inst.get("op")
    # Pick current vmap context (per-block context during lowering)
    vmap_ctx = getattr(owner, '_current_vmap', owner.vmap)

    # 箱理論: InstructionContext作成（PHI等の複雑な命令用）
    # Create instruction context box for complex instructions like PHI
    try:
        inst_ctx = InstructionContext.from_owner(owner, builder, builder.block)
    except Exception:
        inst_ctx = None  # Fallback if context creation fails

    if op == "const":
        dst = inst.get("dst")
        value = inst.get("value")
        lower_const(builder, owner.module, dst, value, vmap_ctx, owner.resolver)

    elif op == "binop":
        operation = inst.get("operation")
        lhs = inst.get("lhs")
        rhs = inst.get("rhs")
        dst = inst.get("dst")
        dst_type = inst.get("dst_type")
        lower_binop(builder, owner.resolver, operation, lhs, rhs, dst,
                    vmap_ctx, builder.block, owner.preds, owner.block_end_values, owner.bb_map,
                    dst_type=dst_type)

    elif op == "jump":
        target = inst.get("target")
        lower_jump(builder, target, owner.bb_map)

    elif op == "copy":
        dst = inst.get("dst")
        src = inst.get("src")
        lower_copy(builder, dst, src, vmap_ctx, owner.resolver, builder.block, owner.preds, owner.block_end_values, owner.bb_map, getattr(owner, 'ctx', None))

    elif op == "branch":
        cond = inst.get("cond")
        then_bid = inst.get("then")
        else_bid = inst.get("else")
        lower_branch(builder, cond, then_bid, else_bid, vmap_ctx, owner.bb_map, owner.resolver, owner.preds, owner.block_end_values)

    elif op == "ret":
        value = inst.get("value")
        lower_return(builder, value, vmap_ctx, func.function_type.return_type,
                     owner.resolver, owner.preds, owner.block_end_values, owner.bb_map, getattr(owner, 'ctx', None))

    elif op == "phi":
        # PHI instruction for SSA value merging - 箱理論実装
        # InstructionContext使用（将来的に全命令に展開予定）
        dst = inst.get("dst")
        incoming_list = inst.get("incoming", [])
        # Convert incoming list format: [{"block": bid, "value": vid}, ...]
        incoming = [(item.get("value"), item.get("block")) for item in incoming_list]

        # 箱化されたコンテキスト使用（inst_ctx経由でアクセス可能）
        # Using boxed context (accessible via inst_ctx)
        lower_phi(
            builder=builder,
            dst_vid=dst,
            incoming=incoming,
            vmap=vmap_ctx,
            bb_map=owner.bb_map,
            current_block=builder.block,
            resolver=owner.resolver,
            block_end_values=owner.block_end_values,
            preds_map=owner.preds
        )

    elif op == "compare":
        # Dedicated compare op
        operation = inst.get("operation") or inst.get("op")
        lhs = inst.get("lhs")
        rhs = inst.get("rhs")
        dst = inst.get("dst")
        cmp_kind = inst.get("cmp_kind")
        lower_compare(builder, operation, lhs, rhs, dst, vmap_ctx,
                      owner.resolver, builder.block, owner.preds, owner.block_end_values, owner.bb_map,
                      meta={"cmp_kind": cmp_kind} if cmp_kind else None,
                      ctx=getattr(owner, 'ctx', None))

    elif op == "unop":
        # Unary op: kind in {'neg','not','bitnot'}; src is operand
        kind = (inst.get("kind") or inst.get("operation") or "").lower()
        srcv = inst.get("src") or inst.get("operand")
        dst = inst.get("dst")
        lower_unop(builder, owner.resolver, kind, srcv, dst, vmap_ctx, builder.block,
                   owner.preds, owner.block_end_values, owner.bb_map, ctx=getattr(owner, 'ctx', None))

    elif op == "mir_call":
        # Unified MIR Call handling
        mir_call = inst.get("mir_call", {})
        dst = inst.get("dst")
        lower_mir_call(owner, builder, mir_call, dst, vmap_ctx, owner.resolver)

    elif op == "call":
        func_name = inst.get("func")
        args = inst.get("args", [])
        dst = inst.get("dst")
        lower_call(builder, owner.module, func_name, args, dst, vmap_ctx, owner.resolver, owner.preds, owner.block_end_values, owner.bb_map, getattr(owner, 'ctx', None))

    elif op == "boxcall":
        box_vid = inst.get("box")
        method = inst.get("method")
        args = inst.get("args", [])
        dst = inst.get("dst")
        lower_boxcall(builder, owner.module, box_vid, method, args, dst,
                      vmap_ctx, owner.resolver, owner.preds, owner.block_end_values, owner.bb_map, getattr(owner, 'ctx', None))
        # Optional: honor explicit dst_type for tagging (string handle)
        try:
            dst_type = inst.get("dst_type")
            if dst is not None and isinstance(dst_type, dict):
                if dst_type.get("kind") == "handle" and dst_type.get("box_type") == "StringBox":
                    if hasattr(owner.resolver, 'mark_string'):
                        owner.resolver.mark_string(int(dst))
            # Track last substring for optional esc_json fallback
            try:
                if isinstance(method, str) and method == 'substring' and isinstance(dst, int):
                    owner._last_substring_vid = int(dst)
            except Exception:
                pass
        except Exception:
            pass

    elif op == "externcall":
        func_name = inst.get("func")
        args = inst.get("args", [])
        dst = inst.get("dst")
        lower_externcall(builder, owner.module, func_name, args, dst,
                         vmap_ctx, owner.resolver, owner.preds, owner.block_end_values, owner.bb_map, getattr(owner, 'ctx', None))

    elif op == "newbox":
        box_type = inst.get("type")
        args = inst.get("args", [])
        dst = inst.get("dst")
        lower_newbox(builder, owner.module, box_type, args, dst,
                     vmap_ctx, owner.resolver, getattr(owner, 'ctx', None))

    elif op == "typeop":
        operation = inst.get("operation")
        src = inst.get("src")
        dst = inst.get("dst")
        target_type = inst.get("target_type")
        lower_typeop(builder, operation, src, dst, target_type,
                     vmap_ctx, owner.resolver, owner.preds, owner.block_end_values, owner.bb_map, getattr(owner, 'ctx', None))

    elif op == "safepoint":
        live = inst.get("live", [])
        lower_safepoint(builder, owner.module, live, vmap_ctx,
                        resolver=owner.resolver, preds=owner.preds,
                        block_end_values=owner.block_end_values, bb_map=owner.bb_map,
                        ctx=getattr(owner, 'ctx', None))

    elif op == "barrier":
        barrier_type = inst.get("type", "memory")
        lower_barrier(builder, barrier_type, ctx=getattr(owner, 'ctx', None))

    elif op == "while":
        # Experimental LoopForm lowering inside a block
        cond = inst.get("cond")
        body = inst.get("body", [])
        owner.loop_count += 1
        if not lower_while_loopform(builder, func, cond, body,
                                    owner.loop_count, owner.vmap, owner.bb_map,
                                    owner.resolver, owner.preds, owner.block_end_values,
                                    getattr(owner, 'ctx', None)):
            # Fallback to regular while (structured)
            try:
                owner.resolver._owner_lower_instruction = owner.lower_instruction
            except Exception:
                pass
            lower_while_regular(builder, func, cond, body,
                                owner.loop_count, owner.vmap, owner.bb_map,
                                owner.resolver, owner.preds, owner.block_end_values)
    else:
        trace_debug(f"[Python LLVM] Unknown instruction: {op}")

    # Record per-inst definition for lifetime hinting as soon as available
    try:
        dst_maybe = inst.get("dst")
        if isinstance(dst_maybe, int) and dst_maybe in owner.vmap:
            cur_bid = None
            try:
                cur_bid = int(str(builder.block.name).replace('bb',''))
            except Exception:
                pass
            if cur_bid is not None:
                owner.def_blocks.setdefault(dst_maybe, set()).add(cur_bid)
    except Exception:
        pass

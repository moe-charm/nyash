from typing import Optional

def ensure_ny_main(builder) -> None:
    """Ensure ny_main wrapper exists by delegating to Main.main/1 or main().
    Modifies builder.module in place; no return value.
    """
    has_ny_main = any(f.name == 'ny_main' for f in builder.module.functions)
    fn_main_box = None
    fn_main_plain = None
    for f in builder.module.functions:
        if f.name == 'Main.main/1':
            fn_main_box = f
        elif f.name == 'main':
            fn_main_plain = f
    target_fn = fn_main_box or fn_main_plain
    if target_fn is None or has_ny_main:
        return

    # Hide the target to avoid symbol conflicts
    try:
        target_fn.linkage = 'private'
    except Exception:
        pass
    # i32 ny_main() { return (i32) Main.main(args) | main(); }
    from llvmlite import ir
    ny_main_ty = ir.FunctionType(builder.i64, [])
    ny_main = ir.Function(builder.module, ny_main_ty, name='ny_main')
    entry = ny_main.append_basic_block('entry')
    b = ir.IRBuilder(entry)
    if fn_main_box is not None:
        # Build default args = new ArrayBox() via nyash.env.box.new_i64x
        i64 = builder.i64
        i8 = builder.i8
        i8p = builder.i8p
        # Declare callee
        callee = None
        for f in builder.module.functions:
            if f.name == 'nyash.env.box.new_i64x':
                callee = f
                break
        if callee is None:
            callee = ir.Function(builder.module, ir.FunctionType(i64, [i8p, i64, i64, i64, i64, i64]), name='nyash.env.box.new_i64x')
        # Create "ArrayBox\0" global
        sbytes = b"ArrayBox\0"
        arr_ty = ir.ArrayType(i8, len(sbytes))
        g = ir.GlobalVariable(builder.module, arr_ty, name='.ny_main_arraybox')
        g.linkage = 'private'
        g.global_constant = True
        g.initializer = ir.Constant(arr_ty, bytearray(sbytes))
        c0 = ir.Constant(builder.i32, 0)
        ptr = b.gep(g, [c0, c0], inbounds=True)
        zero = ir.Constant(i64, 0)
        args_handle = b.call(callee, [ptr, zero, zero, zero, zero, zero], name='ny_main_args')
        rv = b.call(fn_main_box, [args_handle], name='call_Main_main_1')
    else:
        # Plain main() fallback
        if len(fn_main_plain.args) == 0:
            rv = b.call(fn_main_plain, [], name='call_user_main')
        else:
            rv = ir.Constant(builder.i64, 0)
    if hasattr(rv, 'type') and isinstance(rv.type, ir.IntType) and rv.type.width != 32:
        rv64 = b.trunc(rv, builder.i64) if rv.type.width > 64 else b.zext(rv, builder.i64)
        b.ret(rv64)
    elif hasattr(rv, 'type') and isinstance(rv.type, ir.IntType) and rv.type.width == 64:
        b.ret(rv)
    else:
        b.ret(ir.Constant(builder.i64, 0))


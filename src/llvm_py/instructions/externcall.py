"""
ExternCall instruction lowering
Handles the minimal 5 runtime functions: print, error, panic, exit, now
"""

import llvmlite.ir as ir
from typing import Dict, List, Optional

# The 5 minimal external functions
EXTERN_FUNCS = {
    "print": {
        "ret": "void",
        "args": ["i8*"],  # String pointer
        "llvm_name": "ny_print"
    },
    "error": {
        "ret": "void", 
        "args": ["i8*"],  # Error message
        "llvm_name": "ny_error"
    },
    "panic": {
        "ret": "void",
        "args": ["i8*"],  # Panic message
        "llvm_name": "ny_panic"
    },
    "exit": {
        "ret": "void",
        "args": ["i64"],  # Exit code
        "llvm_name": "ny_exit"
    },
    "now": {
        "ret": "i64",
        "args": [],  # No arguments
        "llvm_name": "ny_now"
    }
}

def lower_externcall(
    builder: ir.IRBuilder,
    module: ir.Module,
    func_name: str,
    args: List[int],
    dst_vid: Optional[int],
    vmap: Dict[int, ir.Value],
    resolver=None
) -> None:
    """
    Lower MIR ExternCall instruction
    
    Args:
        builder: Current LLVM IR builder
        module: LLVM module
        func_name: External function name
        args: List of argument value IDs
        dst_vid: Optional destination for return value
        vmap: Value map
        resolver: Optional resolver for type handling
    """
    if func_name not in EXTERN_FUNCS:
        # Unknown extern function - treat as void()
        print(f"Warning: Unknown extern function: {func_name}")
        return
    
    extern_info = EXTERN_FUNCS[func_name]
    llvm_name = extern_info["llvm_name"]
    
    # Look up or declare function
    func = None
    for f in module.functions:
        if f.name == llvm_name:
            func = f
            break
    
    if not func:
        # Build function type
        i8 = ir.IntType(8)
        i64 = ir.IntType(64)
        void = ir.VoidType()
        
        # Return type
        if extern_info["ret"] == "void":
            ret_type = void
        elif extern_info["ret"] == "i64":
            ret_type = i64
        else:
            ret_type = void
        
        # Argument types
        arg_types = []
        for arg_type_str in extern_info["args"]:
            if arg_type_str == "i8*":
                arg_types.append(i8.as_pointer())
            elif arg_type_str == "i64":
                arg_types.append(i64)
        
        func_type = ir.FunctionType(ret_type, arg_types)
        func = ir.Function(module, func_type, name=llvm_name)
    
    # Prepare arguments
    call_args = []
    for i, arg_id in enumerate(args):
        if i >= len(extern_info["args"]):
            break  # Too many arguments
        
        expected_type_str = extern_info["args"][i]
        arg_val = vmap.get(arg_id)
        
        if not arg_val:
            # Default value
            if expected_type_str == "i8*":
                # Null string
                i8 = ir.IntType(8)
                arg_val = ir.Constant(i8.as_pointer(), None)
            elif expected_type_str == "i64":
                arg_val = ir.Constant(ir.IntType(64), 0)
        
        # Type conversion
        if expected_type_str == "i8*":
            # Need string pointer
            if hasattr(arg_val, 'type'):
                if isinstance(arg_val.type, ir.IntType):
                    # int to ptr
                    i8 = ir.IntType(8)
                    arg_val = builder.inttoptr(arg_val, i8.as_pointer())
                elif not arg_val.type.is_pointer:
                    # Need pointer type
                    i8 = ir.IntType(8)
                    arg_val = ir.Constant(i8.as_pointer(), None)
        elif expected_type_str == "i64":
            # Need i64
            if hasattr(arg_val, 'type'):
                if arg_val.type.is_pointer:
                    arg_val = builder.ptrtoint(arg_val, ir.IntType(64))
                elif arg_val.type != ir.IntType(64):
                    # Convert to i64
                    pass  # TODO: Handle other conversions
        
        call_args.append(arg_val)
    
    # Make the call
    if extern_info["ret"] == "void":
        builder.call(func, call_args)
        if dst_vid is not None:
            # Void return - store 0
            vmap[dst_vid] = ir.Constant(ir.IntType(64), 0)
    else:
        result = builder.call(func, call_args, name=f"extern_{func_name}")
        if dst_vid is not None:
            vmap[dst_vid] = result
#!/usr/bin/env python3
"""
Nyash LLVM Python Backend - Main Builder
Following the design principles in docs/LLVM_LAYER_OVERVIEW.md
"""

import json
import sys
from typing import Dict, Any, Optional
import llvmlite.ir as ir
import llvmlite.binding as llvm

class NyashLLVMBuilder:
    """Main LLVM IR builder for Nyash MIR"""
    
    def __init__(self):
        # Initialize LLVM
        llvm.initialize()
        llvm.initialize_native_target()
        llvm.initialize_native_asmprinter()
        
        # Module and basic types
        self.module = ir.Module(name="nyash_module")
        self.i64 = ir.IntType(64)
        self.i32 = ir.IntType(32)
        self.i8 = ir.IntType(8)
        self.i1 = ir.IntType(1)
        self.i8p = self.i8.as_pointer()
        self.f64 = ir.DoubleType()
        
    def build_from_mir(self, mir_json: Dict[str, Any]) -> str:
        """Build LLVM IR from MIR JSON"""
        # TODO: Implement MIR -> LLVM lowering
        # For now, create a simple ny_main that returns 0
        
        # ny_main: extern "C" fn() -> i32
        ny_main_ty = ir.FunctionType(self.i32, [])
        ny_main = ir.Function(self.module, ny_main_ty, name="ny_main")
        
        block = ny_main.append_basic_block(name="entry")
        builder = ir.IRBuilder(block)
        builder.ret(ir.Constant(self.i32, 0))
        
        return str(self.module)
    
    def compile_to_object(self, output_path: str):
        """Compile module to object file"""
        # Create target machine
        target = llvm.Target.from_default_triple()
        target_machine = target.create_target_machine()
        
        # Compile
        mod = llvm.parse_assembly(str(self.module))
        mod.verify()
        
        # Generate object code
        obj = target_machine.emit_object(mod)
        
        # Write to file
        with open(output_path, 'wb') as f:
            f.write(obj)

def main():
    if len(sys.argv) < 2:
        print("Usage: llvm_builder.py <input.mir.json> [-o output.o]")
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_file = "nyash_llvm_py.o"
    
    if "-o" in sys.argv:
        idx = sys.argv.index("-o")
        if idx + 1 < len(sys.argv):
            output_file = sys.argv[idx + 1]
    
    # Read MIR JSON
    with open(input_file, 'r') as f:
        mir_json = json.load(f)
    
    # Build LLVM IR
    builder = NyashLLVMBuilder()
    llvm_ir = builder.build_from_mir(mir_json)
    
    print(f"Generated LLVM IR:\n{llvm_ir}")
    
    # Compile to object
    builder.compile_to_object(output_file)
    print(f"Compiled to {output_file}")

if __name__ == "__main__":
    main()
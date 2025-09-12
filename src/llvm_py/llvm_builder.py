#!/usr/bin/env python3
"""
Nyash LLVM Python Backend - Main Builder
Following the design principles in docs/LLVM_LAYER_OVERVIEW.md
"""

import json
import sys
import os
from typing import Dict, Any, Optional, List, Tuple
import llvmlite.ir as ir
import llvmlite.binding as llvm

# Import instruction handlers
from instructions.const import lower_const
from instructions.binop import lower_binop
from instructions.jump import lower_jump
from instructions.branch import lower_branch
from instructions.ret import lower_return
from instructions.phi import lower_phi, defer_phi_wiring
from instructions.call import lower_call
from instructions.boxcall import lower_boxcall
from instructions.externcall import lower_externcall
from instructions.typeop import lower_typeop, lower_convert
from instructions.newbox import lower_newbox
from instructions.safepoint import lower_safepoint, insert_automatic_safepoint
from instructions.barrier import lower_barrier
from instructions.loopform import lower_while_loopform

from resolver import Resolver
from mir_reader import MIRReader

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
        self.void = ir.VoidType()
        
        # Value and block maps
        self.vmap: Dict[int, ir.Value] = {}  # value_id -> LLVM value
        self.bb_map: Dict[int, ir.Block] = {}  # block_id -> LLVM block
        
        # PHI deferrals for sealed block approach
        self.phi_deferrals: List[Tuple[int, List[Tuple[int, int]]]] = []
        
        # Resolver for unified value resolution
        self.resolver = Resolver(self.vmap, self.bb_map)
        
        # Statistics
        self.loop_count = 0
        
    def build_from_mir(self, mir_json: Dict[str, Any]) -> str:
        """Build LLVM IR from MIR JSON"""
        # Parse MIR
        reader = MIRReader(mir_json)
        functions = reader.get_functions()
        
        if not functions:
            # No functions - create dummy ny_main
            return self._create_dummy_main()
        
        # Process each function
        for func_data in functions:
            self.lower_function(func_data)
        
        # Wire deferred PHIs
        self._wire_deferred_phis()
        
        return str(self.module)
    
    def _create_dummy_main(self) -> str:
        """Create dummy ny_main that returns 0"""
        ny_main_ty = ir.FunctionType(self.i32, [])
        ny_main = ir.Function(self.module, ny_main_ty, name="ny_main")
        block = ny_main.append_basic_block(name="entry")
        builder = ir.IRBuilder(block)
        builder.ret(ir.Constant(self.i32, 0))
        return str(self.module)
    
    def lower_function(self, func_data: Dict[str, Any]):
        """Lower a single MIR function to LLVM IR"""
        name = func_data.get("name", "unknown")
        params = func_data.get("params", [])
        blocks = func_data.get("blocks", [])
        
        # Determine function signature
        if name == "ny_main":
            # Special case: ny_main returns i32
            func_ty = ir.FunctionType(self.i32, [])
        else:
            # Default: i64(i64, ...) signature
            param_types = [self.i64] * len(params)
            func_ty = ir.FunctionType(self.i64, param_types)
        
        # Create function
        func = ir.Function(self.module, func_ty, name=name)
        
        # Create all blocks first
        for block_data in blocks:
            bid = block_data.get("id", 0)
            block_name = f"bb{bid}"
            bb = func.append_basic_block(block_name)
            self.bb_map[bid] = bb
        
        # Process each block
        for block_data in blocks:
            bid = block_data.get("id", 0)
            bb = self.bb_map[bid]
            self.lower_block(bb, block_data, func)
    
    def lower_block(self, bb: ir.Block, block_data: Dict[str, Any], func: ir.Function):
        """Lower a single basic block"""
        builder = ir.IRBuilder(bb)
        instructions = block_data.get("instructions", [])
        
        # Process each instruction
        for inst in instructions:
            self.lower_instruction(builder, inst, func)
    
    def lower_instruction(self, builder: ir.IRBuilder, inst: Dict[str, Any], func: ir.Function):
        """Dispatch instruction to appropriate handler"""
        op = inst.get("op")
        
        if op == "const":
            dst = inst.get("dst")
            value = inst.get("value")
            lower_const(builder, self.module, dst, value, self.vmap)
            
        elif op == "binop":
            operation = inst.get("operation")
            lhs = inst.get("lhs")
            rhs = inst.get("rhs")
            dst = inst.get("dst")
            lower_binop(builder, self.resolver, operation, lhs, rhs, dst, 
                       self.vmap, builder.block)
            
        elif op == "jump":
            target = inst.get("target")
            lower_jump(builder, target, self.bb_map)
            
        elif op == "branch":
            cond = inst.get("cond")
            then_bid = inst.get("then")
            else_bid = inst.get("else")
            lower_branch(builder, cond, then_bid, else_bid, self.vmap, self.bb_map)
            
        elif op == "ret":
            value = inst.get("value")
            lower_return(builder, value, self.vmap, func.return_value.type)
            
        elif op == "phi":
            dst = inst.get("dst")
            incoming = inst.get("incoming", [])
            # Defer PHI wiring for now
            defer_phi_wiring(dst, incoming, self.phi_deferrals)
            
        elif op == "call":
            func_name = inst.get("func")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_call(builder, self.module, func_name, args, dst, self.vmap, self.resolver)
            
        elif op == "boxcall":
            box_vid = inst.get("box")
            method = inst.get("method")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_boxcall(builder, self.module, box_vid, method, args, dst, 
                         self.vmap, self.resolver)
            
        elif op == "externcall":
            func_name = inst.get("func")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_externcall(builder, self.module, func_name, args, dst, 
                            self.vmap, self.resolver)
            
        elif op == "newbox":
            box_type = inst.get("type")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_newbox(builder, self.module, box_type, args, dst, 
                        self.vmap, self.resolver)
            
        elif op == "typeop":
            operation = inst.get("operation")
            src = inst.get("src")
            dst = inst.get("dst")
            target_type = inst.get("target_type")
            lower_typeop(builder, operation, src, dst, target_type, 
                        self.vmap, self.resolver)
            
        elif op == "safepoint":
            live = inst.get("live", [])
            lower_safepoint(builder, self.module, live, self.vmap)
            
        elif op == "barrier":
            barrier_type = inst.get("type", "memory")
            lower_barrier(builder, barrier_type)
            
        elif op == "while":
            # Experimental LoopForm lowering
            cond = inst.get("cond")
            body = inst.get("body", [])
            self.loop_count += 1
            if not lower_while_loopform(builder, func, cond, body, 
                                       self.loop_count, self.vmap, self.bb_map):
                # Fallback to regular while
                self._lower_while_regular(builder, inst, func)
        else:
            if os.environ.get('NYASH_CLI_VERBOSE') == '1':
                print(f"[Python LLVM] Unknown instruction: {op}")
    
    def _lower_while_regular(self, builder: ir.IRBuilder, inst: Dict[str, Any], func: ir.Function):
        """Fallback regular while lowering"""
        # TODO: Implement regular while lowering
        pass
    
    def _wire_deferred_phis(self):
        """Wire all deferred PHI nodes"""
        # TODO: Implement PHI wiring after all blocks are created
        for dst_vid, incoming in self.phi_deferrals:
            # Find the block containing this PHI
            # Wire the incoming edges
            pass
    
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
#!/usr/bin/env python3
"""
Simple test for Nyash LLVM Python backend
Tests basic MIR -> LLVM compilation
"""

import json
from llvm_builder import NyashLLVMBuilder

# Simple MIR test case: function that returns 42
test_mir = {
    "functions": {
        "main": {
            "name": "main",
            "params": [],
            "return_type": "i64",
            "entry_block": 0,
            "blocks": {
                "0": {
                    "instructions": [
                        {
                            "kind": "Const",
                            "dst": 0,
                            "value": {"type": "i64", "value": 42}
                        }
                    ],
                    "terminator": {
                        "kind": "Return",
                        "value": 0
                    }
                }
            }
        }
    }
}

def test_basic():
    """Test basic MIR -> LLVM compilation"""
    builder = NyashLLVMBuilder()
    
    # Generate LLVM IR
    llvm_ir = builder.build_from_mir(test_mir)
    print("Generated LLVM IR:")
    print(llvm_ir)
    
    # Compile to object file
    builder.compile_to_object("test_simple.o")
    print("\nCompiled to test_simple.o")

if __name__ == "__main__":
    test_basic()
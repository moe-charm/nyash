#!/usr/bin/env python3
"""
Simple smoke for Nyash LLVM Python backend
Generates a minimal MIR(JSON) in the current schema and compiles it.
"""

from llvm_builder import NyashLLVMBuilder

# Minimal MIR(JSON): main() { ret 42 }
TEST_MIR = {
    "functions": [
        {
            "name": "main",
            "params": [],
            "blocks": [
                {
                    "id": 0,
                    "instructions": [
                        {"op": "const", "dst": 0, "value": {"type": "i64", "value": 42}},
                        {"op": "ret", "value": 0}
                    ]
                }
            ]
        }
    ]
}

def test_basic():
    builder = NyashLLVMBuilder()
    ir = builder.build_from_mir(TEST_MIR)
    print("Generated LLVM IR (truncated):\n", ir.splitlines()[0:8])
    builder.compile_to_object("test_simple.o")
    print("Compiled to test_simple.o")

if __name__ == "__main__":
    test_basic()

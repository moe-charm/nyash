"""
Unit tests for phi_wiring helpers

These tests construct a minimal function with two blocks and a PHI in the
second block. We verify that placeholders are created and incoming edges
are wired from the correct predecessor, using end-of-block snapshots.
"""

import sys
from pathlib import Path

# Ensure 'src' is importable when running this test directly
TEST_DIR = Path(__file__).resolve().parent
PKG_DIR = TEST_DIR.parent  # src/llvm_py
ROOT = PKG_DIR.parent      # src
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
if str(PKG_DIR) not in sys.path:
    sys.path.insert(0, str(PKG_DIR))

import llvmlite.ir as ir  # type: ignore

from phi_wiring import setup_phi_placeholders, finalize_phis  # type: ignore
import llvm_builder  # type: ignore


def _simple_mir_with_phi():
    """
    Build a minimal MIR JSON that compiles to:
      bb0: const v1=42; jump bb1
      bb1: phi v2=[(bb0,v1)] ; ret v2
    """
    return {
        "functions": [
            {
                "name": "main",
                "params": [],
                "blocks": [
                    {"id": 0, "instructions": [
                        {"op": "const", "dst": 1, "value": {"type": "int", "value": 42}},
                        {"op": "jump", "target": 1}
                    ]},
                    {"id": 1, "instructions": [
                        {"op": "phi", "dst": 2, "incoming": [[1, 0]]},
                        {"op": "ret", "value": 2}
                    ]}
                ]
            }
        ]
    }


def test_phi_placeholders_and_finalize_basic():
    mir = _simple_mir_with_phi()
    b = llvm_builder.NyashLLVMBuilder()
    # Build once to create function, blocks, preds; stop before finalize by calling internals like lower_function
    reader_functions = mir["functions"]
    assert reader_functions
    b.lower_function(reader_functions[0])
    # After lowering a function, finalize_phis is already called at the end of lower_function.
    # Verify via IR text that a PHI exists in bb1 with an incoming from bb0.
    ir_text = str(b.module)
    assert 'bb1' in ir_text
    assert 'phi  i64' in ir_text
    assert '[0, %"bb0"]' in ir_text or '[ i64 0, %"bb0"]' in ir_text

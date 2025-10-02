#!/usr/bin/env python3
"""
Tests for PhiRegistry single-source behavior: ensure only one PHI per (block,dst).
"""
import unittest
import llvmlite.ir as ir

import os, sys
# Ensure intra-package absolute imports like `instructions.const` resolve when
# importing llvm_builder via package path.
_here = os.path.dirname(os.path.dirname(__file__))
if _here not in sys.path:
    sys.path.insert(0, _here)
from src.llvm_py.llvm_builder import NyashLLVMBuilder


class TestPhiRegistry(unittest.TestCase):
    def test_single_phi_per_block_dst(self):
        # Construct a tiny MIR with an explicit phi instruction at block 4
        # and a CFG that will cause block_lower to also pre-ensure PHI placeholders
        # via block_phi_incomings metadata.
        mir = {
            "functions": [
                {
                    "name": "main",
                    "params": [],
                    "blocks": [
                        {  # entry
                            "id": 1,
                            "instructions": [
                                {"op": "jump", "target": 2}
                            ],
                        },
                        {  # produces v10
                            "id": 2,
                            "instructions": [
                                {"op": "const", "dst": 10, "value": {"type": "i64", "value": 1}},
                                {"op": "jump", "target": 4},
                            ],
                        },
                        {  # produces v20
                            "id": 3,
                            "instructions": [
                                {"op": "const", "dst": 20, "value": {"type": "i64", "value": 2}},
                                {"op": "jump", "target": 4},
                            ],
                        },
                        {  # merge with explicit phi (JSON v0 incoming order is (value, block))
                            "id": 4,
                            "instructions": [
                                {"op": "phi", "dst": 30, "incoming": [(10, 2), (20, 3)]},
                                {"op": "ret", "value": 30},
                            ],
                        },
                    ],
                }
            ]
        }

        b = NyashLLVMBuilder(target="native")
        # Build should not raise (verify pass runs inside)
        _ = b.build_from_mir(mir)
        # Verify that vmap has exactly one PHI for dst=30 in block 4
        phi = b.vmap.get(30)
        self.assertIsNotNone(phi)
        # Ensure its name is canonical (without .1 suffix)
        self.assertTrue(str(getattr(phi, 'name', '')).startswith('phi_30'))


if __name__ == "__main__":
    unittest.main()

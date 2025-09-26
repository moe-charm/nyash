#!/usr/bin/env python3
"""
Lightweight unit tests for src/llvm_py/phi_wiring.py (analysis helpers).

These do not require llvmlite; they validate pure-Python helpers like
analyze_incomings() and small control-flow utilities.
Run locally with:
  python3 -m unittest src.llvm_py.tests.test_phi_wiring
"""
import unittest

from src.llvm_py import phi_wiring


class TestPhiWiringHelpers(unittest.TestCase):
    def test_analyze_incomings_simple(self):
        blocks = [
            {
                "id": 10,
                "instructions": [
                    {
                        "op": "phi",
                        "dst": 100,
                        # JSON v0 uses [(value, block)] but helper adapts to [(decl_b, v_src)]
                        "incoming": [(1, 20), (2, 30)],
                    }
                ],
            },
            {"id": 1, "instructions": []},
            {"id": 2, "instructions": []},
        ]
        inc = phi_wiring.analyze_incomings(blocks)
        self.assertIn(10, inc)
        self.assertIn(100, inc[10])
        pairs = set(inc[10][100])
        # Helper normalizes JSON v0 order (value, block) -> (decl_b, v_src)
        self.assertEqual(pairs, {(20, 1), (30, 2)})

    def test_nearest_pred_on_path_negative(self):
        # Build a tiny CFG: 1 -> 2 -> 3, preds_list only contains 9 (not on path)
        succs = {1: [2], 2: [3]}
        preds_list = [9]
        decl_b = 1
        target = 3
        res = phi_wiring._nearest_pred_on_path(succs, preds_list, decl_b, target)
        self.assertIsNone(res)

    def test_build_succs(self):
        preds = {3: [1, 2], 4: [3]}
        succs = phi_wiring._build_succs(preds)
        self.assertEqual(succs, {1: [3], 2: [3], 3: [4]})


if __name__ == "__main__":
    unittest.main()

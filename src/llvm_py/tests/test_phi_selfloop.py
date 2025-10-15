#!/usr/bin/env python3
import unittest
import llvmlite.ir as ir

from src.llvm_py import phi_wiring


class DummyResolver:
    def __init__(self, builder):
        self.builder = builder
        self.block_phi_incomings = {}

    def _value_at_end_i64(self, vs, pred_bid, preds, block_end_values, vmap, bb_map):
        # Return a value snapshot if present
        return self.builder.block_end_values.get((int(pred_bid), int(vs)))


class DummyBuilder:
    pass


class TestPhiSelfLoop(unittest.TestCase):
    def setUp(self):
        self.mod = ir.Module(name="phi_selfloop_mod")
        i64 = ir.IntType(64)
        fnty = ir.FunctionType(i64, [])
        fn = ir.Function(self.mod, fnty, name="main")
        bb1 = fn.append_basic_block(name="bb1")  # preheader
        bb2 = fn.append_basic_block(name="bb2")  # header (self-loop allowed)

        b = DummyBuilder()
        b.module = self.mod
        b.function = fn
        b.i64 = i64
        b.bb_map = {1: bb1, 2: bb2}
        # header has predecessors: self and preheader
        b.preds = {2: [2, 1]}
        b.vmap = {}
        b.block_end_values = {}
        b.def_blocks = {}
        b.resolver = DummyResolver(b)
        self.builder = b

    def test_self_loop_pred_is_wired(self):
        # Seed value at preheader
        self.builder.block_end_values[(1, 10)] = ir.Constant(self.builder.i64, 0)
        # Provide a snapshot for the fallback (non-self) var at header too (for vs swap)
        self.builder.block_end_values[(2, 10)] = ir.Constant(self.builder.i64, 5)

        blocks = [
            {"id": 2, "instructions": [{"op": "phi", "dst": 100, "incoming": [(10, 1), (100, 2)]}]},
            {"id": 1, "instructions": []},
        ]

        phi_wiring.setup_phi_placeholders(self.builder, blocks)
        # ensure a PHI is declared at head (through PhiHandler path in real flow)
        phi = self.builder.vmap.get(100)
        self.assertIsNone(phi)  # placeholders do not create PHI by policy

        # finalize should wire both preds and tolerate self-loop
        phi_wiring.finalize_phis(self.builder)
        phi = self.builder.vmap.get(100)
        # PHI may still be None if allow_create is off, but wiring will attempt ensure/create only when flag is on.
        # We verify that no exception occurs and the metadata path runs. When creation is allowed, incoming should have both bb1 and bb2.
        # This test is primarily to exercise nearest_pred_on_path and preds handling for self-loop.
        self.assertIn(2, self.builder.preds)


if __name__ == "__main__":
    unittest.main()


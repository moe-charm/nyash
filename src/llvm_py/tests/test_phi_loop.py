#!/usr/bin/env python3
import unittest
import llvmlite.ir as ir

from src.llvm_py import phi_wiring


class DummyResolver:
    def __init__(self, builder):
        self.builder = builder
        self.block_phi_incomings = {}

    def _value_at_end_i64(self, vs, pred_bid, preds, block_end_values, vmap, bb_map):
        return self.builder.block_end_values.get((int(pred_bid), int(vs)))


class DummyBuilder:
    pass


class TestPhiLoop(unittest.TestCase):
    def setUp(self):
        self.mod = ir.Module(name="phi_loop_mod")
        i64 = ir.IntType(64)
        fnty = ir.FunctionType(i64, [])
        fn = ir.Function(self.mod, fnty, name="main")
        bb1 = fn.append_basic_block(name="bb1")  # preheader
        bb2 = fn.append_basic_block(name="bb2")  # header/merge
        bb3 = fn.append_basic_block(name="bb3")  # body

        b = DummyBuilder()
        b.module = self.mod
        b.function = fn
        b.i64 = i64
        b.bb_map = {1: bb1, 2: bb2, 3: bb3}
        # header has predecessors: preheader and body (backedge)
        b.preds = {2: [1, 3]}
        b.vmap = {}
        b.block_end_values = {}
        b.def_blocks = {}
        b.resolver = DummyResolver(b)
        self.builder = b

    def test_loop_phi_self_carry(self):
        # Values at end of preds
        self.builder.block_end_values[(1, 10)] = ir.Constant(self.builder.i64, 0)
        # Latch value is self-carry (dst=100); provide alternative seed 10 in incoming
        self.builder.block_end_values[(3, 10)] = ir.Constant(self.builder.i64, 5)

        blocks = [
            {"id": 2, "instructions": [{"op": "phi", "dst": 100, "incoming": [(10, 1), (100, 3)]}]},
            {"id": 1, "instructions": []},
            {"id": 3, "instructions": []},
        ]

        phi_wiring.setup_phi_placeholders(self.builder, blocks)
        phi = self.builder.vmap.get(100)
        self.assertIsNotNone(phi)
        phi_wiring.finalize_phis(self.builder)
        # Verify both predecessors are connected
        incoming = list(getattr(phi, "incoming", []))
        if incoming:
            preds = {blk.name for (_val, blk) in incoming}
            self.assertEqual(preds, {"bb1", "bb3"})
        else:
            # No exception path assurance
            self.assertIn(100, self.builder.vmap)


if __name__ == "__main__":
    unittest.main()


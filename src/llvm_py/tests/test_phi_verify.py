import unittest
from llvmlite import ir

from src.llvm_py.phi_wiring.verify import verify_phi_cfg


class DummyBuilder:
    pass


class TestPhiVerify(unittest.TestCase):
    def test_verify_simple_phi(self):
        m = ir.Module(name="m")
        i64 = ir.IntType(64)
        fty = ir.FunctionType(i64, [])
        fn = ir.Function(m, fty, name="f")
        entry = fn.append_basic_block("bb1")
        pred = fn.append_basic_block("bb0")
        bpred = ir.IRBuilder(pred)
        bpred.ret(ir.Constant(i64, 1))
        b = ir.IRBuilder(entry)
        b.position_at_start(entry)
        phi = b.phi(i64, name="phi_10")
        phi.add_incoming(ir.Constant(i64, 1), pred)

        builder = DummyBuilder()
        builder.module = m
        builder.i64 = i64
        builder.vmap = {10: phi}
        builder.bb_map = {1: entry, 0: pred}
        builder.preds = {1: [0]}
        builder.block_phi_incomings = {1: {10: [(0, 10)]}}
        builder.phi_wired = {(1, 10): {0}}

        probs = verify_phi_cfg(builder, strict=True)
        self.assertEqual(probs, [])


if __name__ == "__main__":
    unittest.main()


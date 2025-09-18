import unittest
import llvmlite.ir as ir

from src.llvm_py.phi_wiring import setup_phi_placeholders


class DummyResolver:
    def __init__(self):
        self.marked = set()

    def mark_string(self, vid: int):
        self.marked.add(int(vid))


class DummyBuilder:
    def __init__(self, bb_map):
        self.i64 = ir.IntType(64)
        self.vmap = {}
        self.def_blocks = {}
        self.resolver = DummyResolver()
        self.bb_map = bb_map


class TestPhiTagging(unittest.TestCase):
    def _mk_blocks_and_bbs(self):
        mod = ir.Module(name="m")
        fnty = ir.FunctionType(ir.VoidType(), [])
        fn = ir.Function(mod, fnty, name="f")
        b0 = fn.append_basic_block(name="b0")
        b1 = fn.append_basic_block(name="b1")
        return mod, {0: b0, 1: b1}

    def test_mark_by_dst_type(self):
        _mod, bb_map = self._mk_blocks_and_bbs()
        builder = DummyBuilder(bb_map)
        blocks = [
            {
                "id": 1,
                "instructions": [
                    {
                        "op": "phi",
                        "dst": 42,
                        "dst_type": {"kind": "handle", "box_type": "StringBox"},
                        "incoming": [[7, 0]],
                    }
                ],
            }
        ]
        setup_phi_placeholders(builder, blocks)
        self.assertIn(42, builder.resolver.marked)

    def test_mark_by_incoming_stringish(self):
        _mod, bb_map = self._mk_blocks_and_bbs()
        builder = DummyBuilder(bb_map)
        blocks = [
            {
                "id": 0,
                "instructions": [
                    {"op": "const", "dst": 7, "value": {"type": "string", "value": "hi"}}
                ],
            },
            {
                "id": 1,
                "instructions": [
                    {
                        "op": "phi",
                        "dst": 43,
                        # no dst_type string; inference should happen via incoming
                        "incoming": [[7, 0]],
                    }
                ],
            },
        ]
        setup_phi_placeholders(builder, blocks)
        self.assertIn(43, builder.resolver.marked)


if __name__ == "__main__":
    unittest.main()


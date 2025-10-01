"""
Instruction Context - 箱理論実装
命令処理に必要な全コンテキストを1つの箱に統一
"""

from typing import Dict, List, Optional, Any
from llvmlite import ir
from dataclasses import dataclass


@dataclass
class InstructionContext:
    """
    命令lowering処理に必要な全コンテキストを保持する箱

    箱理論の実践:
    - 「箱にする」: 散在する引数を1つの箱に統一
    - 「境界を作る」: 命令処理とコンテキスト管理を明確に分離
    - 「戻せる」: 従来の引数渡しにも対応可能
    - 「見える化」: コンテキストの内容が明確
    """

    # Core maps
    vmap: Dict[int, ir.Value]           # Value ID → LLVM Value
    bb_map: Dict[int, ir.Block]         # Block ID → LLVM Block

    # Analysis results
    preds: Dict[int, List[int]]         # Block predecessors
    block_end_values: Dict[int, Dict[int, ir.Value]]  # Block exit snapshots

    # Builders and module
    module: ir.Module                   # LLVM module
    builder: ir.IRBuilder               # Current IR builder
    current_block: ir.Block             # Current basic block

    # Optional components
    resolver: Optional[Any] = None      # Type resolver
    ctx: Optional[Any] = None           # Additional context

    # Metadata
    def_blocks: Optional[Dict[int, set]] = None  # Value definition blocks

    def __post_init__(self):
        """Initialize optional fields"""
        if self.def_blocks is None:
            self.def_blocks = {}

    @classmethod
    def from_owner(cls, owner, builder: ir.IRBuilder, current_block: ir.Block):
        """
        OwnerオブジェクトからInstructionContextを生成

        Args:
            owner: NyashLLVMBuilder instance
            builder: Current IR builder
            current_block: Current basic block

        Returns:
            InstructionContext instance
        """
        vmap_ctx = getattr(owner, '_current_vmap', owner.vmap)

        return cls(
            vmap=vmap_ctx,
            bb_map=owner.bb_map,
            preds=owner.preds,
            block_end_values=owner.block_end_values,
            module=owner.module,
            builder=builder,
            current_block=current_block,
            resolver=owner.resolver,
            ctx=getattr(owner, 'ctx', None),
            def_blocks=owner.def_blocks
        )

    def get_value(self, vid: int) -> Optional[ir.Value]:
        """Get LLVM value by ID"""
        return self.vmap.get(vid)

    def set_value(self, vid: int, value: ir.Value):
        """Set LLVM value for ID"""
        self.vmap[vid] = value

    def get_block(self, bid: int) -> Optional[ir.Block]:
        """Get LLVM block by ID"""
        return self.bb_map.get(bid)

    def get_predecessors(self, bid: int) -> List[int]:
        """Get predecessor block IDs"""
        return self.preds.get(bid, [])

    def get_block_end_value(self, bid: int, vid: int) -> Optional[ir.Value]:
        """Get value at end of block"""
        snap = self.block_end_values.get(bid, {})
        return snap.get(vid)

    def record_definition(self, vid: int, bid: int):
        """Record value definition in block"""
        if self.def_blocks is not None:
            self.def_blocks.setdefault(vid, set()).add(bid)

    def get_current_block_id(self) -> Optional[int]:
        """Get current block ID from block name"""
        try:
            return int(str(self.current_block.name).replace('bb', ''))
        except Exception:
            return None

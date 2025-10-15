"""
MIR14 instruction lowering modules
Each instruction has its own file, following Rust structure
"""

# Import all instruction handlers
from .const import lower_const
from .binop import lower_binop
from .compare import lower_compare
# controlflow
from .controlflow.jump import lower_jump
from .controlflow.branch import lower_branch
from .ret import lower_return
from .phi import lower_phi
from .typeop import lower_typeop
from .safepoint import lower_safepoint
from .barrier import lower_barrier

# LoopForm support
from .loopform import LoopFormContext, lower_while_loopform

__all__ = [
    'lower_const', 'lower_binop', 'lower_compare',
    'lower_jump', 'lower_branch', 'lower_return',
    'lower_phi', 'lower_typeop', 'lower_safepoint',
    'lower_barrier',
    'LoopFormContext', 'lower_while_loopform'
]

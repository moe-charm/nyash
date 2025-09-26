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
from .call import lower_call
from .boxcall import lower_boxcall
from .externcall import lower_externcall
from .typeop import lower_typeop
from .safepoint import lower_safepoint
from .barrier import lower_barrier
from .newbox import lower_newbox

# LoopForm support
from .loopform import LoopFormContext, lower_while_loopform

__all__ = [
    'lower_const', 'lower_binop', 'lower_compare',
    'lower_jump', 'lower_branch', 'lower_return',
    'lower_phi', 'lower_call', 'lower_boxcall',
    'lower_externcall', 'lower_typeop', 'lower_safepoint',
    'lower_barrier', 'lower_newbox',
    'LoopFormContext', 'lower_while_loopform'
]

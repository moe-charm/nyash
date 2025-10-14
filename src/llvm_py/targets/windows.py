"""
Windows (MSVC) Target Implementation
"""

from .base import BaseTarget
import llvmlite.binding as llvm


class WindowsTarget(BaseTarget):
    """
    Windows COFF (x86_64-pc-windows-msvc) target.

    Responsibilities:
    - Provide stable triple for COFF object emission
    - Use a target machine created from triple to emit .obj
    """

    def get_triple(self) -> str:
        return "x86_64-pc-windows-msvc"

    def configure_function(self, func) -> None:
        # Default OK
        pass

    def emit_object(self, module, output_path: str) -> None:
        llvm_ir = str(module)
        llvm_module = llvm.parse_assembly(llvm_ir)
        llvm_module.verify()
        target = llvm.Target.from_triple(self.get_triple())
        tm = target.create_target_machine()
        with open(output_path, 'wb') as f:
            f.write(tm.emit_object(llvm_module))

    def get_name(self) -> str:
        return "WindowsMSVC"


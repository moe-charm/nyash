"""
WASM Target Implementation
箱理論実践: WASM専用ロジックをここに集約
"""

from .base import BaseTarget
import llvmlite.binding as llvm


class WasmTarget(BaseTarget):
    """
    WASM (wasm32-unknown-wasi) target

    責任:
    - WASM triple設定
    - WASM関数linkage設定（external）
    - WASMバイナリ生成
    """

    def get_triple(self) -> str:
        """WASM target triple"""
        return "wasm32-unknown-wasi"

    def configure_function(self, func) -> None:
        """
        Configure function for WASM export

        WASM requires external linkage for exported functions
        """
        # Set external linkage for WASM export
        func.linkage = "external"
        # Optional: set visibility (not strictly required)
        # func.visibility = "default"

    def emit_object(self, module, output_path: str) -> None:
        """
        Emit WASM binary

        Uses llvmlite's built-in WASM backend
        No need for LLC/wasm-ld!
        """
        # Parse IR module
        llvm_ir = str(module)
        llvm_module = llvm.parse_assembly(llvm_ir)
        llvm_module.verify()

        # Create target machine for WASM
        target = llvm.Target.from_triple(self.get_triple())
        target_machine = target.create_target_machine()

        # Emit object file (WASM binary)
        with open(output_path, 'wb') as f:
            f.write(target_machine.emit_object(llvm_module))

    def get_name(self) -> str:
        """Return target name"""
        return "WASM"

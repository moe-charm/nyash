"""
Native Target Implementation
箱理論実践: Native専用ロジックをここに集約
"""

from .base import BaseTarget
import llvmlite.binding as llvm


class NativeTarget(BaseTarget):
    """
    Native (x86_64/ARM64) target

    責任:
    - System default triple設定
    - Native関数linkage設定（デフォルト）
    - Nativeオブジェクトファイル生成
    """

    def get_triple(self) -> str:
        """
        Native target triple (system default)

        Examples:
        - x86_64-unknown-linux-gnu (Linux)
        - x86_64-apple-darwin (macOS)
        - x86_64-pc-windows-msvc (Windows)
        """
        return llvm.get_default_triple()

    def configure_function(self, func) -> None:
        """
        Configure function for native

        Default linkage is fine for native
        """
        # No special configuration needed
        pass

    def emit_object(self, module, output_path: str) -> None:
        """
        Emit native object file

        Uses system default target machine
        """
        # Parse IR module
        llvm_ir = str(module)
        llvm_module = llvm.parse_assembly(llvm_ir)
        llvm_module.verify()

        # Create target machine for native
        target = llvm.Target.from_default_triple()
        target_machine = target.create_target_machine()

        # Emit object file
        with open(output_path, 'wb') as f:
            f.write(target_machine.emit_object(llvm_module))

    def get_name(self) -> str:
        """Return target name"""
        return "Native"

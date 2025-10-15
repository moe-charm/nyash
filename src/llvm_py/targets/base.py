"""
Base Target Interface
箱理論実践: ターゲット抽象化で責任分離
"""

from abc import ABC, abstractmethod
from typing import Optional


class BaseTarget(ABC):
    """
    LLVM target abstraction

    箱の境界:
    - get_triple(): ターゲットtriple定義
    - configure_module(): module設定
    - configure_function(): function設定
    - emit_object(): オブジェクトファイル生成
    """

    @abstractmethod
    def get_triple(self) -> str:
        """
        Return LLVM target triple

        Examples:
        - wasm32-unknown-wasi (WASM)
        - x86_64-unknown-linux-gnu (Native Linux)
        - x86_64-apple-darwin (Native macOS)
        """
        raise NotImplementedError

    def configure_module(self, module) -> None:
        """
        Configure LLVM module for this target

        Args:
            module: llvmlite IR module
        """
        # Default: set triple
        module.triple = self.get_triple()

    def configure_function(self, func) -> None:
        """
        Configure LLVM function for this target

        Args:
            func: llvmlite IR function

        Default: no-op (target-specific configuration in subclasses)
        """
        pass

    @abstractmethod
    def emit_object(self, module, output_path: str) -> None:
        """
        Emit object file for this target

        Args:
            module: llvmlite IR module
            output_path: output file path
        """
        raise NotImplementedError

    def get_name(self) -> str:
        """Return target name for logging"""
        return self.__class__.__name__

#!/usr/bin/env python3
"""
LinkingAnalyzer - 箱理論実装
WASM Linking Section解析の境界明確化

箱理論の実践:
- 「箱にする」: linking section解析を1つの箱に統一
- 「境界を作る」: シンボルテーブル解析を明確に分離
- 「戻せる」: 従来の巨大関数から箱経由に段階移行
- 「見える化」: 関数名→index対応が明確
"""

from dataclasses import dataclass
from typing import Dict, Optional
from wasm_section_parser import WasmSectionParser


@dataclass
class FunctionSymbol:
    """
    関数シンボル情報

    箱の境界: シンボル情報をカプセル化
    """
    name: str               # Function name
    index: int              # Function index
    flags: int              # Symbol flags


class LinkingAnalyzer:
    """
    WASM Linking Section解析器

    箱理論実践:
    - parse(): linking section解析のエントリーポイント
    - _parse_symbol_table(): シンボルテーブル解析の境界
    - get_function_index(): 関数名→index変換の境界

    内部実装（linking version, subsection type, LEB128）を隠蔽
    """

    # Linking subsection types
    SUBSECTION_SYMBOL_TABLE = 8

    # Symbol kinds
    SYMBOL_FUNCTION = 0
    SYMBOL_DATA = 1
    SYMBOL_GLOBAL = 2
    SYMBOL_SECTION = 3
    SYMBOL_EVENT = 4
    SYMBOL_TABLE = 5

    def __init__(self):
        """Initialize linking analyzer"""
        self.function_symbols: Dict[str, FunctionSymbol] = {}

    def parse(self, wasm_data: bytes) -> Dict[str, int]:
        """
        箱理論: linking section解析のエントリーポイント

        WASM binaryからlinking sectionを解析し、関数名→index対応を返す

        Args:
            wasm_data: WASM binary data

        Returns:
            dict mapping function name to index
        """
        # WasmSectionParser箱を使用（箱理論: 箱越しに処理）
        parser = WasmSectionParser(wasm_data)

        # Find "linking" custom section
        linking_data = parser.find_custom_section("linking")
        if linking_data is None:
            return {}

        # Parse linking section content
        return self._parse_linking_section(linking_data, parser)

    def _parse_linking_section(
        self,
        data: bytes,
        parser: WasmSectionParser
    ) -> Dict[str, int]:
        """
        箱理論: linking section内容解析の境界

        Args:
            data: Linking section payload
            parser: WasmSectionParser instance for LEB128 decoding

        Returns:
            dict mapping function name to index
        """
        if len(data) == 0:
            return {}

        offset = 0

        # Read linking version (usually 2)
        linking_version = data[offset]
        offset += 1

        # Parse subsections
        while offset < len(data):
            # Read subsection type
            if offset >= len(data):
                break

            subsection_type = data[offset]
            offset += 1

            # Read subsection size
            subsection_size, offset = parser.read_varuint(data, offset)

            subsection_end = offset + subsection_size

            if subsection_end > len(data):
                # Invalid subsection size
                break

            # Parse subsection based on type
            if subsection_type == self.SUBSECTION_SYMBOL_TABLE:
                # Symbol table subsection
                subsection_data = data[offset:subsection_end]
                self._parse_symbol_table(subsection_data, parser)

            # Move to next subsection
            offset = subsection_end

        # Return function name -> index mapping
        return {
            sym.name: sym.index
            for sym in self.function_symbols.values()
        }

    def _parse_symbol_table(
        self,
        data: bytes,
        parser: WasmSectionParser
    ) -> None:
        """
        箱理論: シンボルテーブル解析の境界

        シンボルテーブルsubsectionを解析し、関数シンボルを抽出

        Args:
            data: Symbol table subsection data
            parser: WasmSectionParser for LEB128 decoding
        """
        offset = 0

        # Read symbol count
        count, offset = parser.read_varuint(data, offset)

        # Read symbols
        for _ in range(count):
            if offset >= len(data):
                break

            # Read symbol kind
            symbol_kind = data[offset]
            offset += 1

            # Read symbol flags
            flags, offset = parser.read_varuint(data, offset)

            # Parse based on symbol kind
            if symbol_kind == self.SYMBOL_FUNCTION:
                # Function symbol: index + name
                func_index, offset = parser.read_varuint(data, offset)

                # Read function name
                name_len, offset = parser.read_varuint(data, offset)

                if offset + name_len > len(data):
                    break

                func_name = data[offset:offset+name_len].decode('utf-8', errors='ignore')
                offset += name_len

                # Store function symbol
                self.function_symbols[func_name] = FunctionSymbol(
                    name=func_name,
                    index=func_index,
                    flags=flags
                )
            else:
                # Skip non-function symbols (implementation detail hidden)
                # 境界: 他のシンボル種別の処理は将来拡張可能
                break

    def get_function_index(self, name: str) -> Optional[int]:
        """
        箱理論: 関数名→index変換の境界

        Args:
            name: Function name

        Returns:
            Function index or None if not found
        """
        symbol = self.function_symbols.get(name)
        return symbol.index if symbol is not None else None

    def list_functions(self) -> Dict[str, int]:
        """
        箱理論: 関数一覧取得の境界

        Returns:
            dict mapping function name to index
        """
        return {
            name: sym.index
            for name, sym in self.function_symbols.items()
        }

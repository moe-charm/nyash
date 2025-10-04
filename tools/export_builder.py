#!/usr/bin/env python3
"""
ExportBuilder - 箱理論実装
WASM Export Section生成の境界明確化

箱理論の実践:
- 「箱にする」: export section生成を1つの箱に統一
- 「境界を作る」: LEB128エンコード、セクション挿入を明確に分離
- 「戻せる」: 従来のバイナリ操作から箱経由に段階移行
- 「見える化」: export生成プロセスが明確
"""

from dataclasses import dataclass
from typing import List, Tuple
from wasm_section_parser import WasmSectionParser


@dataclass
class ExportEntry:
    """
    Export entry情報

    箱の境界: export情報をカプセル化
    """
    name: str               # Export name
    kind: int               # Export kind (0=func, 1=table, 2=mem, 3=global)
    index: int              # Export index

    # Export kind constants
    KIND_FUNC = 0
    KIND_TABLE = 1
    KIND_MEMORY = 2
    KIND_GLOBAL = 3


class ExportBuilder:
    """
    WASM Export Section生成器

    箱理論実践:
    - build_export_section(): export section生成のエントリーポイント
    - inject_export_section(): WASMバイナリへのexport section挿入の境界
    - _encode_varuint(): LEB128エンコードの境界

    内部実装（LEB128, section順序, バイナリ操作）を隠蔽
    """

    # WASM section IDs
    SECTION_EXPORT = 7
    SECTION_CODE = 10

    def __init__(self):
        """Initialize export builder"""
        self.entries: List[ExportEntry] = []

    def add_export(self, name: str, kind: int, index: int) -> None:
        """
        箱理論: export追加の境界

        Args:
            name: Export name
            kind: Export kind (0=func, 1=table, 2=mem, 3=global)
            index: Export index
        """
        self.entries.append(ExportEntry(
            name=name,
            kind=kind,
            index=index
        ))

    def build_export_section(self) -> bytes:
        """
        箱理論: export section生成の境界

        登録されたexport entriesからexport sectionバイナリを生成

        Returns:
            Export section binary (section ID + size + entries)
        """
        if len(self.entries) == 0:
            return b''

        # Build export entries
        entries_data = bytearray()

        # Entry count
        entries_data.extend(self._encode_varuint(len(self.entries)))

        # Encode each entry
        for entry in self.entries:
            # Export name
            entries_data.extend(self._encode_name(entry.name))

            # Export kind
            entries_data.append(entry.kind)

            # Export index
            entries_data.extend(self._encode_varuint(entry.index))

        # Build section: ID + size + entries
        section = bytearray()
        section.append(self.SECTION_EXPORT)  # Section ID: Export
        section.extend(self._encode_varuint(len(entries_data)))
        section.extend(entries_data)

        return bytes(section)

    def inject_export_section(self, wasm_data: bytes) -> bytes:
        """
        箱理論: export section挿入の境界

        既存のWASMバイナリにexport sectionを挿入
        挿入位置: Code section(10)の直前、または末尾

        Args:
            wasm_data: Original WASM binary

        Returns:
            Modified WASM binary with export section
        """
        # Generate export section
        export_section = self.build_export_section()
        if len(export_section) == 0:
            return wasm_data

        # Parse existing sections
        parser = WasmSectionParser(wasm_data)
        sections = []

        for header, body in parser.iter_sections():
            # Reconstruct section (ID + size + body)
            section_binary = bytearray()
            section_binary.append(header.section_id)
            section_binary.extend(self._encode_varuint(header.size))
            section_binary.extend(body)

            sections.append((header.section_id, bytes(section_binary)))

        # Rebuild WASM with export section inserted
        result = bytearray(wasm_data[0:8])  # magic + version

        export_inserted = False
        for section_id, section_data in sections:
            # Insert export section before Code(10) or higher
            if not export_inserted and section_id >= 8:
                result.extend(export_section)
                export_inserted = True

            result.extend(section_data)

        # If not inserted yet, add at end
        if not export_inserted:
            result.extend(export_section)

        return bytes(result)

    @staticmethod
    def _encode_varuint(value: int) -> bytes:
        """
        箱理論: LEB128エンコードの境界

        符号なし整数をLEB128形式でエンコード

        Args:
            value: Integer to encode

        Returns:
            LEB128 encoded bytes
        """
        result = bytearray()

        while True:
            byte = value & 0x7F
            value >>= 7

            if value != 0:
                byte |= 0x80  # Set continuation bit

            result.append(byte)

            if value == 0:
                break

        return bytes(result)

    @staticmethod
    def _encode_name(name: str) -> bytes:
        """
        箱理論: 名前エンコードの境界

        文字列を WASM name形式でエンコード (length + UTF-8)

        Args:
            name: String to encode

        Returns:
            Encoded name (length as varuint + UTF-8 bytes)
        """
        name_bytes = name.encode('utf-8')
        result = bytearray()
        result.extend(ExportBuilder._encode_varuint(len(name_bytes)))
        result.extend(name_bytes)
        return bytes(result)

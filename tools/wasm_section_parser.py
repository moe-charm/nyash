#!/usr/bin/env python3
"""
WasmSectionParser - 箱理論実装
WASM Section解析の境界明確化

箱理論の実践:
- 「箱にする」: セクション解析処理を1つの箱に統一
- 「境界を作る」: LEB128デコード、セクション走査を明確に分離
- 「戻せる」: 従来のバイナリ操作コードから箱経由に段階移行
- 「見える化」: セクション構造が明確
"""

from dataclasses import dataclass
from typing import Iterator, Optional, Tuple


@dataclass
class SectionHeader:
    """
    WASMセクションヘッダー

    箱の境界: セクション情報をカプセル化
    """
    section_id: int         # Section ID (0=custom, 1=type, etc.)
    size: int               # Section size in bytes
    offset: int             # Section body start offset

    def is_custom(self) -> bool:
        """Custom section判定"""
        return self.section_id == 0


class WasmSectionParser:
    """
    WASM Section解析器

    箱理論実践:
    - read_varuint(): LEB128デコードの境界
    - read_section_header(): セクションヘッダー解析の境界
    - iter_sections(): セクション走査の境界

    内部実装を隠蔽し、呼び出し側はWASMバイナリ構造の詳細を知らない
    """

    def __init__(self, wasm_data: bytes):
        """
        Args:
            wasm_data: WASM binary data
        """
        self.data = wasm_data

        # Validate magic and version
        if len(wasm_data) < 8:
            raise ValueError("Invalid WASM: too short")
        if wasm_data[0:4] != b'\x00asm':
            raise ValueError("Invalid WASM magic")

    @staticmethod
    def read_varuint(data: bytes, offset: int) -> Tuple[int, int]:
        """
        箱理論: LEB128デコードの境界

        LEB128符号なし整数をデコード
        内部実装（shift, mask, ループ）を隠蔽

        Args:
            data: Binary data
            offset: Read start offset

        Returns:
            (value, next_offset) tuple
        """
        value = 0
        shift = 0
        current = offset

        while current < len(data):
            byte = data[current]
            current += 1

            value |= (byte & 0x7F) << shift

            if (byte & 0x80) == 0:
                # Last byte (no continuation bit)
                break

            shift += 7

        return value, current

    def read_section_header(self, offset: int) -> Optional[SectionHeader]:
        """
        箱理論: セクションヘッダー解析の境界

        セクションID、サイズ、本体開始位置を読み取る

        Args:
            offset: Section header start offset

        Returns:
            SectionHeader or None if end of data
        """
        if offset >= len(self.data):
            return None

        # Read section ID
        section_id = self.data[offset]
        offset += 1

        # Read section size (varuint)
        size, offset = self.read_varuint(self.data, offset)

        return SectionHeader(
            section_id=section_id,
            size=size,
            offset=offset
        )

    def iter_sections(self) -> Iterator[Tuple[SectionHeader, bytes]]:
        """
        箱理論: セクション走査の境界

        全セクションを順次yield
        内部のオフセット計算を隠蔽

        Yields:
            (SectionHeader, section_body_data) tuples
        """
        offset = 8  # Skip magic (4 bytes) + version (4 bytes)

        while offset < len(self.data):
            header = self.read_section_header(offset)
            if header is None:
                break

            # Extract section body
            body_start = header.offset
            body_end = body_start + header.size

            if body_end > len(self.data):
                # Invalid section size
                break

            body = self.data[body_start:body_end]

            yield header, body

            # Move to next section
            offset = body_end

    def find_custom_section(self, name: str) -> Optional[bytes]:
        """
        箱理論: 名前によるカスタムセクション検索の境界

        指定名のカスタムセクションを検索

        Args:
            name: Custom section name (e.g., "linking", "name")

        Returns:
            Section body data or None if not found
        """
        for header, body in self.iter_sections():
            if not header.is_custom():
                continue

            # Read section name from custom section body
            if len(body) == 0:
                continue

            # Custom section format: name_len (varuint) + name + payload
            name_len, offset = self.read_varuint(body, 0)

            if offset + name_len > len(body):
                continue

            section_name = body[offset:offset+name_len].decode('utf-8', errors='ignore')

            if section_name == name:
                # Return payload (after name)
                return body[offset+name_len:]

        return None

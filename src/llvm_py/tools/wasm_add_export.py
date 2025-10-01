#!/usr/bin/env python3
"""Add export section to WASM binary"""

import sys
import struct

def write_uleb128(value):
    """Encode unsigned LEB128"""
    result = []
    while True:
        byte = value & 0x7F
        value >>= 7
        if value != 0:
            byte |= 0x80
        result.append(byte)
        if value == 0:
            break
    return bytes(result)

def add_export(wasm_path, output_path, func_name="ny_main", func_index=0):
    """Add export section to WASM binary"""

    with open(wasm_path, 'rb') as f:
        data = bytearray(f.read())

    # Check magic number
    if data[:4] != b'\x00asm':
        print("Error: Not a WASM binary!")
        sys.exit(1)

    # Find insertion point (before Code section, after Function section)
    # Section order: Type(1), Import(2), Function(3), ... Export(7), ... Code(10)
    offset = 8
    insert_pos = len(data)
    last_before_code = 8

    while offset < len(data):
        section_id = data[offset]
        section_start = offset
        offset += 1

        # Read section size
        size_start = offset
        section_size = 0
        shift = 0
        while True:
            byte = data[offset]
            offset += 1
            section_size |= (byte & 0x7F) << shift
            if (byte & 0x80) == 0:
                break
            shift += 7

        section_end = offset + section_size

        # Export (7) should be inserted before Code (10)
        # Track the position after the last section before Code
        if section_id < 7:  # Before Export
            last_before_code = section_end
        elif section_id >= 8:  # After Export (Start, Element, Code, Data)
            # Insert before this section
            insert_pos = section_start
            break

        offset = section_end

    # If we didn't find a section >= 8, insert at last_before_code
    if insert_pos == len(data):
        insert_pos = last_before_code

    # Build export section
    # Export section format:
    # - section_id: 7 (Export)
    # - count: 1 (one export)
    # - name_len: len(func_name)
    # - name: func_name bytes
    # - kind: 0 (function)
    # - index: func_index

    export_data = bytearray()
    export_data.extend(write_uleb128(1))  # count: 1 export
    export_data.extend(write_uleb128(len(func_name)))  # name length
    export_data.extend(func_name.encode('utf-8'))  # name
    export_data.append(0)  # kind: function
    export_data.extend(write_uleb128(func_index))  # function index

    # Wrap in section
    section = bytearray()
    section.append(7)  # section_id: Export
    section.extend(write_uleb128(len(export_data)))  # section size
    section.extend(export_data)

    # Insert section
    result = data[:insert_pos] + section + data[insert_pos:]

    # Write output
    with open(output_path, 'wb') as f:
        f.write(result)

    print(f"✅ Added export '{func_name}' (index {func_index}) to {output_path}")

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Usage: wasm_add_export.py <input.wasm> <output.wasm> [func_name] [func_index]")
        sys.exit(1)

    input_path = sys.argv[1]
    output_path = sys.argv[2]
    func_name = sys.argv[3] if len(sys.argv) > 3 else "ny_main"
    func_index = int(sys.argv[4]) if len(sys.argv) > 4 else 0

    add_export(input_path, output_path, func_name, func_index)

#!/usr/bin/env python3
"""
WASM Binary Inspector
Parse and display WASM module structure
"""

import sys
import struct

def read_varuint(data, offset):
    """Read LEB128 variable-length unsigned integer"""
    result = 0
    shift = 0
    while True:
        byte = data[offset]
        offset += 1
        result |= (byte & 0x7F) << shift
        if (byte & 0x80) == 0:
            break
        shift += 7
    return result, offset

def read_varint(data, offset):
    """Read LEB128 variable-length signed integer"""
    result = 0
    shift = 0
    byte = 0
    while True:
        byte = data[offset]
        offset += 1
        result |= (byte & 0x7F) << shift
        shift += 7
        if (byte & 0x80) == 0:
            break
    # Sign extend
    if shift < 64 and (byte & 0x40):
        result |= -(1 << shift)
    return result, offset

def read_name(data, offset):
    """Read UTF-8 name"""
    length, offset = read_varuint(data, offset)
    name = data[offset:offset+length].decode('utf-8')
    return name, offset + length

def inspect_wasm(filepath):
    """Inspect WASM binary file"""
    with open(filepath, 'rb') as f:
        data = f.read()

    print(f"=== WASM Binary Inspector ===")
    print(f"File: {filepath}")
    print(f"Size: {len(data)} bytes")
    print()

    # Check magic
    magic = data[0:4]
    if magic != b'\x00asm':
        print("❌ Invalid WASM magic")
        return
    print(f"✅ Magic: {magic.hex()} (valid)")

    # Check version
    version = struct.unpack('<I', data[4:8])[0]
    print(f"✅ Version: {version}")
    print()

    # Parse sections
    offset = 8
    section_names = {
        0: "Custom",
        1: "Type",
        2: "Import",
        3: "Function",
        4: "Table",
        5: "Memory",
        6: "Global",
        7: "Export",
        8: "Start",
        9: "Element",
        10: "Code",
        11: "Data",
        12: "DataCount"
    }

    print("=== Sections ===")
    sections = {}
    while offset < len(data):
        section_id = data[offset]
        offset += 1

        section_size, offset = read_varuint(data, offset)
        section_data = data[offset:offset+section_size]
        section_name = section_names.get(section_id, f"Unknown({section_id})")

        print(f"[{section_id}] {section_name}: {section_size} bytes")
        sections[section_id] = section_data

        offset += section_size

    print()

    # Analyze Export section
    if 7 in sections:
        print("=== Export Section ===")
        export_data = sections[7]
        export_offset = 0
        count, export_offset = read_varuint(export_data, export_offset)
        print(f"Export count: {count}")

        for i in range(count):
            name, export_offset = read_name(export_data, export_offset)
            kind = export_data[export_offset]
            export_offset += 1
            index, export_offset = read_varuint(export_data, export_offset)

            kind_name = {0: "func", 1: "table", 2: "mem", 3: "global"}.get(kind, "unknown")
            print(f"  {i}: '{name}' ({kind_name} {index})")
    else:
        print("⚠️  No Export section found!")

    print()

    # Analyze Function section
    if 3 in sections:
        print("=== Function Section ===")
        func_data = sections[3]
        func_offset = 0
        count, func_offset = read_varuint(func_data, func_offset)
        print(f"Function count: {count}")

        for i in range(count):
            type_idx, func_offset = read_varuint(func_data, func_offset)
            print(f"  func[{i}]: type {type_idx}")

    print()

    # Analyze Type section
    if 1 in sections:
        print("=== Type Section ===")
        type_data = sections[1]
        type_offset = 0
        count, type_offset = read_varuint(type_data, type_offset)
        print(f"Type count: {count}")

        for i in range(count):
            form = type_data[type_offset]
            type_offset += 1

            if form == 0x60:  # func type
                param_count, type_offset = read_varuint(type_data, type_offset)
                params = []
                for _ in range(param_count):
                    params.append(type_data[type_offset])
                    type_offset += 1

                result_count, type_offset = read_varuint(type_data, type_offset)
                results = []
                for _ in range(result_count):
                    results.append(type_data[type_offset])
                    type_offset += 1

                type_names = {0x7F: "i32", 0x7E: "i64", 0x7D: "f32", 0x7C: "f64"}
                param_str = ", ".join(type_names.get(p, f"0x{p:02x}") for p in params)
                result_str = ", ".join(type_names.get(r, f"0x{r:02x}") for r in results)
                print(f"  type[{i}]: ({param_str}) -> ({result_str})")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: wasm_inspector.py <wasm_file>")
        sys.exit(1)

    inspect_wasm(sys.argv[1])

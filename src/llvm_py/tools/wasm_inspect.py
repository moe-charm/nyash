#!/usr/bin/env python3
"""Simple WASM binary inspector"""

import sys
import struct

def read_uleb128(data, offset):
    """Read unsigned LEB128"""
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

def inspect_wasm(path):
    with open(path, 'rb') as f:
        data = f.read()

    # Check magic number
    if data[:4] != b'\x00asm':
        print("Not a WASM binary!")
        return

    version = struct.unpack('<I', data[4:8])[0]
    print(f"WASM version: {version}")

    imported_func_count = 0
    defined_func_count = 0

    offset = 8
    while offset < len(data):
        section_id = data[offset]
        offset += 1

        section_size, offset = read_uleb128(data, offset)
        section_end = offset + section_size

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
            11: "Data"
        }

        section_name = section_names.get(section_id, f"Unknown({section_id})")
        print(f"\nSection {section_id} ({section_name}): {section_size} bytes")

        # Parse Import section
        if section_id == 2:
            count, off = read_uleb128(data, offset)
            print(f"  Import count: {count}")
            for i in range(count):
                # Read module name length
                mod_len, off = read_uleb128(data, off)
                module_name = data[off:off+mod_len].decode('utf-8')
                off += mod_len
                # Read field name length
                field_len, off = read_uleb128(data, off)
                field_name = data[off:off+field_len].decode('utf-8')
                off += field_len
                # Read kind
                kind = data[off]
                off += 1
                if kind == 0:  # function
                    type_idx, off = read_uleb128(data, off)
                    imported_func_count += 1
                    print(f"  Import #{i}: {module_name}.{field_name} (func, type={type_idx})")
                else:
                    print(f"  Import #{i}: {module_name}.{field_name} (kind={kind})")

        # Parse Function section
        elif section_id == 3:
            count, off = read_uleb128(data, offset)
            defined_func_count = count
            print(f"  Function count: {count}")
            for i in range(count):
                type_idx, off = read_uleb128(data, off)
                actual_index = imported_func_count + i
                print(f"  Function #{i} (actual index={actual_index}): type={type_idx}")

        # Parse Export section
        elif section_id == 7:
            count, off = read_uleb128(data, offset)
            print(f"  Export count: {count}")

            for i in range(count):
                # Read name length
                name_len, off = read_uleb128(data, off)
                # Read name
                name = data[off:off+name_len].decode('utf-8')
                off += name_len
                # Read kind
                kind = data[off]
                off += 1
                # Read index
                idx, off = read_uleb128(data, off)

                kind_names = {0: "func", 1: "table", 2: "mem", 3: "global"}
                print(f"  Export #{i}: {name} (kind={kind_names.get(kind, kind)}, index={idx})")

        offset = section_end

    print("\n" + "="*60)
    print("📊 Summary:")
    print(f"  Imported functions: {imported_func_count}")
    print(f"  Defined functions: {defined_func_count}")
    print(f"  Total function space: {imported_func_count + defined_func_count}")
    print(f"  Function index range: 0-{imported_func_count + defined_func_count - 1}")
    print("="*60)
    print("\n✅ Inspection complete")

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: wasm_inspect.py <file.wasm>")
        sys.exit(1)

    inspect_wasm(sys.argv[1])

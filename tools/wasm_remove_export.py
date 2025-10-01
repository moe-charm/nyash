#!/usr/bin/env python3
"""Remove all export sections from WASM"""
import sys

def read_uleb128(data, offset):
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

def remove_exports(input_path, output_path):
    with open(input_path, 'rb') as f:
        data = bytearray(f.read())

    result = bytearray(data[:8])  # Keep header
    offset = 8

    while offset < len(data):
        section_id = data[offset]
        section_start = offset
        offset += 1

        size_start = offset
        section_size, offset = read_uleb128(data, offset)
        section_end = offset + section_size

        # Copy everything except Export sections
        if section_id != 7:
            result.extend(data[section_start:section_end])
        else:
            print(f"Removed Export section (size={section_size})")

        offset = section_end

    with open(output_path, 'wb') as f:
        f.write(result)

    print(f"✅ Wrote {output_path}")

if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Usage: remove_export.py <input.wasm> <output.wasm>")
        sys.exit(1)

    remove_exports(sys.argv[1], sys.argv[2])

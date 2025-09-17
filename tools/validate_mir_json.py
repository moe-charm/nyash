#!/usr/bin/env python3
"""
Validate a MIR JSON file against the Nyash JSON v0 schema.

Usage:
  python3 tools/validate_mir_json.py <file.json> [--schema docs/reference/mir/json_v0.schema.json]

Requires the 'jsonschema' Python package. Install via:
  python3 -m pip install jsonschema
"""

import argparse
import json
import sys
from pathlib import Path

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('json_file', help='MIR JSON file path')
    ap.add_argument('--schema', default='docs/reference/mir/json_v0.schema.json', help='Schema JSON path')
    args = ap.parse_args()

    try:
        import jsonschema  # type: ignore
    except Exception:
        print('[schema] error: Python package "jsonschema" not found.\n'
              'Install with: python3 -m pip install jsonschema', file=sys.stderr)
        return 2

    try:
        with open(args.json_file, 'r', encoding='utf-8') as f:
            data = json.load(f)
    except Exception as e:
        print(f'[schema] error: failed to read JSON: {e}', file=sys.stderr)
        return 3

    try:
        with open(args.schema, 'r', encoding='utf-8') as f:
            schema = json.load(f)
    except Exception as e:
        print(f'[schema] error: failed to read schema: {e}', file=sys.stderr)
        return 4

    try:
        jsonschema.validate(instance=data, schema=schema)
    except jsonschema.ValidationError as e:  # type: ignore
        # Show human-friendly context
        path = '/'.join([str(p) for p in e.path])
        print(f'[schema] validation failed at $.{path}: {e.message}', file=sys.stderr)
        return 5

    print('[schema] validation OK')
    return 0

if __name__ == '__main__':
    sys.exit(main())


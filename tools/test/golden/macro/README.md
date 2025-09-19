# Macro Goldens

- Compare expanded AST JSON (key-order insensitive) against expected.
- Structure mirrors apps/tests/macro categories.
- Keep case names aligned with test files for discoverability.

Helpers
- normalize_json: Python json.dumps with sort_keys=True
- *_user_macro_golden.sh scripts: run nyash --dump-expanded-ast-json and compare

# Optional CI Snippet (nyfmt PoC)

This is a documentation‑only snippet showing how to wire a non‑blocking nyfmt PoC check in CI. Do not enable until the PoC exists.

```yaml
# .github/workflows/nyfmt-poc.yml (example; disabled by default)
name: nyfmt-poc
on:
  workflow_dispatch: {}
  # push: { branches: [ never-enable-by-default ] }

jobs:
  nyfmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Print nyfmt PoC smoke
        run: |
          chmod +x tools/nyfmt_smoke.sh
          NYFMT_POC=1 ./tools/nyfmt_smoke.sh
```

Notes
- Keep this job opt‑in (workflow_dispatch) until the formatter PoC exists.
- The smoke script only echoes guidance; it does not fail the build.

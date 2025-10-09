Quick Selfhost — focused subset of selfhost/using tests

Purpose
- Run selfhost/using-related smokes independently from the main quick profile.
- Allows tighter env and easier triage without impacting core quick.

Notes
- These wrappers simply invoke the original quick/selfhost tests.
- Use `tools/smokes/v2/run.sh --profile quick-selfhost`.


# VM Stats Cookbook

Collect VM instruction stats (JSON) to guide optimization and instruction set diet.

## Prerequisites
- Build: `cargo build --release -j32`
- Ensure plugins are configured in `nyash.toml` if your program uses them.

## Quick Start
```bash
# Human-readable
./target/release/nyash --backend vm --vm-stats local_tests/vm_stats_http_ok.nyash

# JSON for tooling
./target/release/nyash --backend vm --vm-stats --vm-stats-json local_tests/vm_stats_http_ok.nyash > vm_stats_ok.json

# Or via helper script
tools/run_vm_stats.sh local_tests/vm_stats_http_ok.nyash vm_stats_ok.json
```

## Sample Programs
- `local_tests/vm_stats_http_ok.nyash` — Server responds "OK" to a client GET.
- `local_tests/vm_stats_http_err.nyash` — Client GET to an unreachable port (Result Err path).
- `local_tests/vm_stats_http_404.nyash` — Server returns 404/"NF"; transport成功＋アプリ層エラーの代表例。
- `local_tests/vm_stats_http_500.nyash` — Server returns 500/"ERR"; 同上。
- `local_tests/vm_stats_filebox.nyash` — FileBox open/write/copyFrom/read.

## Tips
- Enable plugin debugging when needed:
  - `NYASH_DEBUG_PLUGIN=1` — Show VM→Plugin TLV header preview.
  - `NYASH_NET_LOG=1 NYASH_NET_LOG_FILE=net_plugin.log` — Net plugin logs.
- Env alternative to CLI flags:
  - `NYASH_VM_STATS=1` and `NYASH_VM_STATS_JSON=1`.

## Next Steps
- Collect stats for normal and error flows (OK/404/500/unreachable, FileBox)。
- Compare hot instructions across scenarios（BoxCall/Const/NewBox の比率、Branch/Jump/Phi の有無）。
- Feed findings into the 26-instruction diet discussion（コア維持・メタ降格・型折りたたみ）。

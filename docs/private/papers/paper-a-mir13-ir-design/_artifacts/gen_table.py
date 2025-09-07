#!/usr/bin/env python3
import sys, os, csv, statistics

def scan_results(path):
    rows = []
    for name in sorted(os.listdir(path)):
        if not name.endswith('.csv'):
            continue
        fpath = os.path.join(path, name)
        with open(fpath, newline='') as f:
            rdr = csv.reader(f)
            vals = []
            label = None
            for r in rdr:
                if not r:
                    continue
                label = r[0]
                try:
                    vals.append(float(r[1]))
                except Exception:
                    pass
            if label and vals:
                rows.append((name, label, len(vals), statistics.median(vals), statistics.mean(vals)))
    return rows

def main():
    if len(sys.argv) < 2:
        print("usage: gen_table.py <results_dir>", file=sys.stderr)
        sys.exit(1)
    res = scan_results(sys.argv[1])
    if not res:
        print("no CSVs found", file=sys.stderr)
        sys.exit(2)
    print("| File | Label | N | Median (ms) | Mean (ms) |")
    print("|------|-------|---|-------------:|----------:|")
    for name, label, n, med, mean in res:
        print(f"| {name} | {label} | {n} | {med:.1f} | {mean:.1f} |")

if __name__ == '__main__':
    main()


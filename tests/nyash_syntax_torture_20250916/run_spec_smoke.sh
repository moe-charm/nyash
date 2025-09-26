
#!/usr/bin/env bash
set -euo pipefail

NYASH_BIN=${NYASH_BIN:-./target/release/nyash}
BACKENDS=${BACKENDS:-"interp vm llvm"}   # choose subset: "interp vm", etc.
OUTDIR=${OUTDIR:-_out}
mkdir -p "$OUTDIR"

fail=0

normalize_file () {
  local infile="$1"; local outfile="$2"
  # Drop runner/interpreter noise; keep only program prints
  local tmp="$outfile.tmp"
  rg -v -e '^📝' -e '^🚀' -e '^✅' -e '^🔍' -e '^🔌' \
        -e '^\[plugin-loader\]' -e '^\[Bridge\]' \
        -e '^Result(Type)?' -e '^MIR Module' \
        -e '^; ' -e '^;MIR' "$infile" > "$tmp" || true
  # Keep last non-empty line as canonical output
  awk 'NF{last=$0} END{if (last) print last;}' "$tmp" > "$outfile" || true
  rm -f "$tmp"
}

run_one () {
  local file="$1"
  local mode="$2"
  local out="$OUTDIR/$(basename "$file").$mode.out"
  case "$mode" in
    interp) "$NYASH_BIN" "$file" > "$out" ;;
    vm)     "$NYASH_BIN" --backend vm "$file" > "$out" ;;
    llvm)
      # Requires LLVM features and NYASH_LLVM_OBJ_OUT env; emit and run
      local obj="$OUTDIR/nyash_llvm_temp.o"
      NYASH_LLVM_OBJ_OUT="$obj" "$NYASH_BIN" --backend llvm "$file" >/dev/null
      cc "$obj" -L crates/nyrt/target/release -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o "$OUTDIR/app_$(basename "$file" .nyash)"
      "$OUTDIR/app_$(basename "$file" .nyash)" > "$out"
      ;;
    *) echo "Unknown mode: $mode" >&2; return 2;;
  esac
  # Normalize
  normalize_file "$out" "$out.norm"
}

compare_modes () {
  local file="$1"
  local modes=($BACKENDS)
  local ref="${modes[0]}"
  for m in "${modes[@]}"; do
    run_one "$file" "$m"
  done
  local refout="$OUTDIR/$(basename "$file").$ref.out.norm"
  for m in "${modes[@]:1}"; do
    local out="$OUTDIR/$(basename "$file").$m.out.norm"
    if ! diff -u "$refout" "$out"; then
      echo "[DIFF] $(basename "$file") ($ref vs $m) differs" >&2
      fail=1
    fi
  done
}

main () {
  local tests=( $(ls -1 tests/syntax_torture/*.nyash 2>/dev/null || true) )
  if [ ${#tests[@]} -eq 0 ]; then
    # fallback: assume running inside this folder
    tests=( $(ls -1 ./*.nyash) )
  fi

  for t in "${tests[@]}"; do
    echo "==> $(basename "$t")"
    compare_modes "$t"
  done

  if [ $fail -ne 0 ]; then
    echo "Some tests differ across backends." >&2
    exit 1
  fi
  echo "All tests matched across selected backends."
}

main "$@"

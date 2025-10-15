#!/usr/bin/env bash
set -euo pipefail

# Generate a simple ny_main glue in C and compile to an object.
# Usage:
#   tools/aot/gen_ny_main.sh --ret N               # ny_main() { return N; }
#   tools/aot/gen_ny_main.sh --concat-len A B      # returns len(A+B) using NyRT dotted exports
#   tools/aot/gen_ny_main.sh --array-len N         # returns N after pushing N ints into a new Array (nyash.array.*)
#   tools/aot/gen_ny_main.sh --map-size N          # returns N after inserting N kv-pairs into a new Map (nyash.map.*)
# Env:
#   OUT_C   : output C path (default: build/ny_main_glue.c)
#   OUT_OBJ : output object path (default: build/ny_main_glue.o)

OUT_C=${OUT_C:-build/ny_main_glue.c}
OUT_OBJ=${OUT_OBJ:-build/ny_main_glue.o}

mkdir -p "$(dirname "$OUT_C")"

mode=""
arg1=""; arg2=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --ret) mode=ret; arg1="$2"; shift 2 ;;
    --concat-len) mode=concat; arg1="$2"; arg2="$3"; shift 3 ;;
    --array-len) mode=array; arg1="$2"; shift 2 ;;
    --map-size) mode=map; arg1="$2"; shift 2 ;;
    *) echo "Unknown option: $1" >&2; exit 2 ;;
  esac
done

if [ -z "$mode" ]; then
  echo "Usage: $0 [--ret N | --concat-len A B]" >&2
  exit 2
fi

case "$mode" in
  ret)
cat > "$OUT_C" <<C
#include <stdint.h>
__attribute__((visibility("default"))) long long ny_main(void){ return (long long)($arg1); }
C
;;
  concat)
cat > "$OUT_C" <<C
#include <stdint.h>
// Use clang/GCC asm alias to link dotted exports
extern long long nyash_box_from_i8_string(const char*) asm("nyash.box.from_i8_string");
extern long long nyash_string_concat_hh(long long,long long) asm("nyash.string.concat_hh");
extern long long nyash_string_len_h(long long) asm("nyash.string.len_h");
__attribute__((visibility("default"))) long long ny_main(void){
  long long a=nyash_box_from_i8_string("$arg1");
  long long b=nyash_box_from_i8_string("$arg2");
  long long c=nyash_string_concat_hh(a,b);
  return nyash_string_len_h(c);
}
C
;;
  array)
cat > "$OUT_C" <<C
#include <stdint.h>
// Array helpers (dotted exports)
extern long long nyash_array_new_h(void) asm("nyash.array.new_h");
extern long long nyash_array_push_h(long long, long long) asm("nyash.array.push_h");
extern long long nyash_array_len_h(long long) asm("nyash.array.len_h");
__attribute__((visibility("default"))) long long ny_main(void){
  long long h = nyash_array_new_h();
  if (!h) return -1;
  long long n = (long long)($arg1);
  for (long long i=0; i<n; ++i) { (void)nyash_array_push_h(h, i+1); }
  return nyash_array_len_h(h);
}
C
;;
  map)
cat > "$OUT_C" <<C
#include <stdint.h>
// Map helpers (dotted exports)
extern long long nyash_map_birth_h(void) asm("nyash.map.birth_h");
extern long long nyash_map_set_h(long long, long long, long long) asm("nyash.map.set_h");
extern long long nyash_map_size_h(long long) asm("nyash.map.size_h");
__attribute__((visibility("default"))) long long ny_main(void){
  long long m = nyash_map_birth_h();
  if (!m) return -1;
  long long n = (long long)($arg1);
  for (long long i=0; i<n; ++i) { (void)nyash_map_set_h(m, i+1, i+100); }
  return nyash_map_size_h(m);
}
C
;;
esac

echo "[gen] wrote $OUT_C" >&2

# Compile to object (Linux clang default)
clang -c "$OUT_C" -o "$OUT_OBJ"
echo "[gen] object: $OUT_OBJ" >&2

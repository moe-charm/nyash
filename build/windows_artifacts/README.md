# Windows Artifacts - Phase 15.77

Generated: 2025-10-14 11:34 JST
Status: Both routes COMPLETE ✅

## Files

### Executables
- **test_msvc.exe** (724 KB) - MSVC static runtime build
- **test_min.exe** (7.4 MB) - MinGW static runtime build

### Source Code
- **ny_main_win.c** (930 bytes) - Test program that calls NyRT runtime functions

## Verification

Both executables produce the same output:

```
C:\> test_msvc.exe
Result: 6

C:\> test_min.exe
Result: 6
```

## Build Details

See: `../WINDOWS_EXECUTION_LOG.txt` for complete build commands and execution logs.

## NyRT Runtime Functions Verified

✅ nyash.box.from_i8_string   - String box creation
✅ nyash.string.concat_hh      - String concatenation  
✅ nyash.string.len_h          - String length

## Checksums

MSVC EXE:
- MD5: 5c4815642cb8fe73682d6666c385d37a
- Size: 741,888 bytes

MinGW EXE:
- Size: 7,749,632 bytes

## Phase 15.77 Milestone

✅ MinGW Route - PASS (WSL-only build)
✅ MSVC Route - PASS (WSL + Windows tools)
✅ Both routes produce correct output: Result: 6

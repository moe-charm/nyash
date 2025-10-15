# Windows Profile

Windows-specific smoke tests that require WSL + Windows PowerShell interop.

## Prerequisites

- **WSL 2** (Windows Subsystem for Linux)
- **PowerShell** available via `powershell.exe`
- **LLVM** installed on Windows side
- **build_llvm.ps1** configured in `tools/` directory

## Test Suites

### Quick Profile (`quick/`)

Fast-running tests for basic Windows functionality:

- **`core/windows_exe_basic.sh`**: Basic Windows .exe build and execution test
  - Builds a minimal Nyash program to Windows .exe using PowerShell
  - Verifies exit code propagation (expects exit code 3)
  - Timeout: 180 seconds (Windows builds are slower)

### Full Profile (`full/`)

Comprehensive Windows test suite (future expansion).

## Running Tests

```bash
# Run quick Windows tests
./tools/smokes/v2/run.sh --profile windows-quick

# Run all Windows tests (future)
./tools/smokes/v2/run.sh --profile windows-full
```

## Configuration

See `tools/smokes/v2/configs/windows/env.sh` for Windows-specific environment variables.

## Architecture Notes

### Why Separate Windows Profile?

Windows tests have unique requirements:
- Cross-platform execution (WSL → Windows)
- Different timeout requirements
- PowerShell dependency
- Windows LLVM toolchain

### Test Isolation

Windows tests are isolated to:
- Avoid platform-specific failures in CI
- Allow optional execution on WSL-enabled systems
- Provide clear skip messages when prerequisites missing

## Troubleshooting

### "powershell.exe not found"

WSL Windows interop is not enabled. Check:
```bash
# Should return path to powershell.exe
which powershell.exe
```

### "build_llvm.ps1 failed"

Windows LLVM toolchain not configured. Verify:
- LLVM installed on Windows
- `tools/build_llvm.ps1` exists and is executable
- PowerShell execution policy allows script execution

### "Windows EXE run failed"

Check:
- Antivirus not blocking .exe execution
- Required DLLs available on Windows PATH
- Nyash runtime kernel available on Windows side

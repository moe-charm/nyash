#!/usr/bin/env bash
# migrate_windows_tests.sh — Migrate Windows smoke tests to new structure
#
# Purpose: Safely migrate existing Windows tests from:
#   tools/smokes/v2/profiles/quick/llvm/windows/
# To:
#   tools/smokes/v2/profiles/windows/quick/core/
#
# Usage:
#   ./tools/migrate_windows_tests.sh [--dry-run] [--help]

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Dry run mode
DRY_RUN="${DRY_RUN:-0}"

# Backup directory
BACKUP_DIR=""

# ============================================================================
# Logging
# ============================================================================

log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $*"; }

# ============================================================================
# Execution wrapper
# ============================================================================

execute() {
    if [ "$DRY_RUN" = "1" ]; then
        log_info "[DRY RUN] $*"
        return 0
    else
        log_info "Executing: $*"
        "$@"
    fi
}

# ============================================================================
# Safety functions
# ============================================================================

backup_before_migration() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    BACKUP_DIR="$PROJECT_ROOT/tools/smokes/v2/profiles/.backup_windows_$timestamp"

    log_step "Creating backup: $BACKUP_DIR"

    if [ "$DRY_RUN" = "0" ]; then
        mkdir -p "$BACKUP_DIR"
        local src_dir="$PROJECT_ROOT/tools/smokes/v2/profiles/quick/llvm/windows"
        if [ -d "$src_dir" ]; then
            cp -r "$src_dir" "$BACKUP_DIR/"
            log_info "Backup created: $BACKUP_DIR"
        else
            log_warn "Source directory not found, skipping backup: $src_dir"
        fi
    else
        log_info "[DRY RUN] Would create backup: $BACKUP_DIR"
    fi
}

rollback() {
    log_warn "ERROR detected! Rolling back changes..."

    if [ "$DRY_RUN" = "1" ]; then
        log_info "[DRY RUN] Would rollback git changes"
        return 0
    fi

    cd "$PROJECT_ROOT"

    # Check if there are uncommitted changes
    if ! git diff --quiet || ! git diff --cached --quiet; then
        log_info "Resetting uncommitted changes..."
        git reset --hard HEAD
        log_info "Rollback complete"
    else
        log_info "No uncommitted changes to rollback"
    fi

    # Restore from backup if available
    if [ -n "$BACKUP_DIR" ] && [ -d "$BACKUP_DIR" ]; then
        log_info "Restoring from backup: $BACKUP_DIR"
        local dest_dir="$PROJECT_ROOT/tools/smokes/v2/profiles/quick/llvm/windows"
        rm -rf "$dest_dir"
        cp -r "$BACKUP_DIR/windows" "$dest_dir"
        log_info "Backup restored"
    fi
}

# ============================================================================
# Validation functions
# ============================================================================

validate_preconditions() {
    log_step "Validating preconditions..."

    # Check git repository
    if [ ! -d "$PROJECT_ROOT/.git" ]; then
        log_error "Not in a git repository: $PROJECT_ROOT"
        return 1
    fi

    # Check source file exists
    local src_file="$PROJECT_ROOT/tools/smokes/v2/profiles/quick/llvm/windows/wsl_windows_exe_smoke.sh"
    if [ ! -f "$src_file" ]; then
        log_error "Source file not found: $src_file"
        return 1
    fi

    # Check git is clean (unless dry-run)
    if [ "$DRY_RUN" = "0" ]; then
        cd "$PROJECT_ROOT"
        if ! git diff --quiet || ! git diff --cached --quiet; then
            log_error "Git working directory is not clean. Please commit or stash changes first."
            log_error "Run 'git status' to see uncommitted changes."
            return 1
        fi
    fi

    log_info "All preconditions satisfied"
    return 0
}

# ============================================================================
# Migration functions
# ============================================================================

create_directory_structure() {
    log_step "Step 1/7: Creating new directory structure..."

    execute mkdir -p "$PROJECT_ROOT/tools/smokes/v2/profiles/windows/quick/core"
    execute mkdir -p "$PROJECT_ROOT/tools/smokes/v2/profiles/windows/full"
    execute mkdir -p "$PROJECT_ROOT/tools/smokes/v2/configs/windows"

    log_info "Directory structure created"
}

move_test_file() {
    log_step "Step 2/7: Moving test file..."

    local src="$PROJECT_ROOT/tools/smokes/v2/profiles/quick/llvm/windows/wsl_windows_exe_smoke.sh"
    local dst="$PROJECT_ROOT/tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh"

    if [ ! -f "$src" ]; then
        log_error "Source file not found: $src"
        return 1
    fi

    execute git mv "$src" "$dst"

    log_info "Test file moved: windows_exe_basic.sh"
}

fix_relative_paths() {
    log_step "Step 3/7: Fixing relative paths..."

    local dst="$PROJECT_ROOT/tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh"

    if [ "$DRY_RUN" = "0" ]; then
        # Old: ../../../../lib/test_runner.sh (from quick/llvm/windows/)
        # New: ../../../../../lib/test_runner.sh (from windows/quick/core/)
        sed -i 's|../../../../lib/|../../../../../lib/|g' "$dst"
        log_info "Relative paths updated"
    else
        log_info "[DRY RUN] Would update: ../../../../lib/ -> ../../../../../lib/"
    fi
}

create_readme() {
    log_step "Step 4/7: Creating README.md..."

    local readme="$PROJECT_ROOT/tools/smokes/v2/profiles/windows/README.md"

    if [ "$DRY_RUN" = "0" ]; then
        cat > "$readme" << 'EOF'
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
EOF
        log_info "README.md created"
    else
        log_info "[DRY RUN] Would create README.md"
    fi
}

create_config_files() {
    log_step "Step 5/7: Creating config files..."

    local config_env="$PROJECT_ROOT/tools/smokes/v2/configs/windows/env.sh"

    if [ "$DRY_RUN" = "0" ]; then
        cat > "$config_env" << 'EOF'
#!/usr/bin/env bash
# Windows profile environment configuration

# Extended timeout for Windows builds (cross-platform overhead)
export SMOKES_DEFAULT_TIMEOUT=${SMOKES_DEFAULT_TIMEOUT:-180}

# Windows-specific paths (overridable)
export WINDOWS_LLVM_ROOT="${WINDOWS_LLVM_ROOT:-C:\Program Files\LLVM}"
export WINDOWS_BUILD_SCRIPT="${WINDOWS_BUILD_SCRIPT:-tools\build_llvm.ps1}"

# WSL interop check
if ! command -v powershell.exe >/dev/null 2>&1; then
    export WINDOWS_AVAILABLE=0
else
    export WINDOWS_AVAILABLE=1
fi
EOF
        chmod +x "$config_env"
        log_info "Config files created"
    else
        log_info "[DRY RUN] Would create configs/windows/env.sh"
    fi
}

cleanup_old_directory() {
    log_step "Step 6/7: Cleaning up old directory..."

    local old_dir="$PROJECT_ROOT/tools/smokes/v2/profiles/quick/llvm/windows"

    if [ -d "$old_dir" ]; then
        # Check if directory is empty
        if [ -z "$(ls -A "$old_dir")" ]; then
            execute git rm -r "$old_dir"
            log_info "Removed empty directory: $old_dir"
        else
            log_warn "Directory not empty, manual cleanup required: $old_dir"
            if [ "$DRY_RUN" = "0" ]; then
                ls -la "$old_dir"
            fi
        fi
    else
        log_info "Old directory already removed or doesn't exist"
    fi
}

verify_migration() {
    log_step "Step 7/7: Verifying migration..."

    if [ "$DRY_RUN" = "1" ]; then
        log_info "[DRY RUN] Skipping file existence checks (files not created)"
        log_info "✓ Dry run verification passed"
        return 0
    fi

    local new_file="$PROJECT_ROOT/tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh"
    local readme="$PROJECT_ROOT/tools/smokes/v2/profiles/windows/README.md"
    local config="$PROJECT_ROOT/tools/smokes/v2/configs/windows/env.sh"

    local all_ok=1

    # Check new test file
    if [ -f "$new_file" ]; then
        log_info "✓ Test file exists: windows_exe_basic.sh"
    else
        log_error "✗ Test file missing: $new_file"
        all_ok=0
    fi

    # Check README
    if [ -f "$readme" ]; then
        log_info "✓ README exists"
    else
        log_error "✗ README missing: $readme"
        all_ok=0
    fi

    # Check config
    if [ -f "$config" ]; then
        log_info "✓ Config exists"
    else
        log_error "✗ Config missing: $config"
        all_ok=0
    fi

    # Check git status
    cd "$PROJECT_ROOT"
    log_info ""
    log_info "Git status:"
    git status --short
    log_info ""

    if [ $all_ok -eq 1 ]; then
        log_info "✓ Migration verification passed"
        return 0
    else
        log_error "✗ Migration verification failed"
        return 1
    fi
}

# ============================================================================
# Main migration
# ============================================================================

main() {
    cd "$PROJECT_ROOT"

    log_info "========================================"
    log_info "Windows Smoke Tests Migration"
    log_info "========================================"
    log_info ""

    if [ "$DRY_RUN" = "1" ]; then
        log_warn "DRY RUN mode enabled - no changes will be made"
        log_info ""
    fi

    # Validate preconditions
    if ! validate_preconditions; then
        log_error "Precondition check failed. Aborting."
        exit 1
    fi

    # Create backup
    backup_before_migration

    # Execute migration steps
    create_directory_structure
    move_test_file
    fix_relative_paths
    create_readme
    create_config_files
    cleanup_old_directory
    verify_migration

    # Success message
    log_info ""
    log_info "========================================"
    log_info "Migration complete!"
    log_info "========================================"
    log_info ""

    if [ "$DRY_RUN" = "1" ]; then
        log_info "This was a DRY RUN. No changes were made."
        log_info "Run without --dry-run to execute migration:"
        log_info "  ./tools/migrate_windows_tests.sh"
    else
        log_info "Next steps:"
        log_info "  1. Review changes: git status"
        log_info "  2. Review diff: git diff --cached"
        log_info "  3. Test migration: ./tools/smokes/v2/run.sh --profile windows-quick"
        log_info "  4. Commit changes: git commit -m 'refactor(smokes): Migrate Windows tests to new structure'"
        log_info ""
        log_info "Backup location (safe to delete after commit):"
        log_info "  $BACKUP_DIR"
    fi
}

# ============================================================================
# Error handling
# ============================================================================

# Trap errors and rollback
trap 'rollback; exit 1' ERR

# ============================================================================
# CLI argument parsing
# ============================================================================

show_help() {
    cat << EOF
Usage: $0 [OPTIONS]

Migrate Windows smoke tests from old structure to new profile-based structure.

From: tools/smokes/v2/profiles/quick/llvm/windows/
To:   tools/smokes/v2/profiles/windows/quick/core/

Options:
  --dry-run    Show what would be done without making changes
  --help       Show this help message

Examples:
  # Preview migration
  ./tools/migrate_windows_tests.sh --dry-run

  # Execute migration
  ./tools/migrate_windows_tests.sh

Safety Features:
  - Automatic backup before migration
  - Git status validation (must be clean)
  - Rollback on error
  - Dry run mode for preview

EOF
}

# Parse arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --dry-run)
            DRY_RUN=1
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
    shift
done

# ============================================================================
# Execute
# ============================================================================

main

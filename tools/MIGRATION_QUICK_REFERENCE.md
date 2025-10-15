# Windows Tests Migration - Quick Reference

## 🚀 Quick Start (3 Commands)

```bash
# 1. Preview what will happen (safe, no changes)
./tools/migrate_windows_tests.sh --dry-run

# 2. Execute migration (requires clean git status)
./tools/migrate_windows_tests.sh

# 3. Verify and commit
git status
git commit -m "refactor(smokes): Migrate Windows tests to new structure"
```

---

## 📁 What Gets Moved

| Before | After |
|--------|-------|
| `quick/llvm/windows/wsl_windows_exe_smoke.sh` | `windows/quick/core/windows_exe_basic.sh` |
| *(no README)* | `windows/README.md` *(new)* |
| *(no config)* | `configs/windows/env.sh` *(new)* |

---

## ✅ Success Indicators

After running migration script, you should see:

1. ✅ **7 steps complete** (Step 1/7 → Step 7/7)
2. ✅ **Backup created**: `.backup_windows_YYYYMMDD_HHMMSS/`
3. ✅ **Git status shows**:
   - Renamed: `wsl_windows_exe_smoke.sh` → `windows_exe_basic.sh`
   - New: `windows/README.md`
   - New: `configs/windows/env.sh`
4. ✅ **Verification passed**: "✓ Migration verification passed"

---

## 🧪 Quick Test Commands

```bash
# Test 1: Syntax check
bash -n tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh

# Test 2: Direct run
tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh

# Test 3: Verify relative path fix
grep "lib/test_runner.sh" tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh
# Should show: ../../../../../lib/test_runner.sh (6 levels)
```

---

## 🚨 Troubleshooting

### "Git working directory is not clean"
```bash
# Check what's uncommitted
git status

# Either commit or stash
git stash
./tools/migrate_windows_tests.sh
git stash pop
```

### "Source file not found"
Migration already done or file moved manually. Check:
```bash
ls tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh
```

### Want to rollback?
```bash
# Automatic (if migration script still running)
# Ctrl+C triggers automatic rollback via trap ERR

# Manual (after migration completed)
git reset --hard HEAD

# Restore from backup
BACKUP_DIR=$(ls -td tools/smokes/v2/profiles/.backup_windows_* | head -1)
rm -rf tools/smokes/v2/profiles/quick/llvm/windows
cp -r "$BACKUP_DIR/windows" tools/smokes/v2/profiles/quick/llvm/windows
```

---

## 📋 Full Checklist

For comprehensive step-by-step verification, see:
- **[MIGRATION_CHECKLIST.md](./MIGRATION_CHECKLIST.md)** (detailed)

---

## 🛠️ Script Options

```bash
# Show help
./tools/migrate_windows_tests.sh --help

# Dry run (preview only, no changes)
./tools/migrate_windows_tests.sh --dry-run

# Actual execution
./tools/migrate_windows_tests.sh
```

---

## 🎯 Expected Changes Summary

**Files Created**: 3
- `windows/quick/core/windows_exe_basic.sh` (moved)
- `windows/README.md` (new)
- `configs/windows/env.sh` (new)

**Files Deleted**: 1
- `quick/llvm/windows/wsl_windows_exe_smoke.sh` (moved from)

**Directories Created**: 4
- `windows/`
- `windows/quick/core/`
- `windows/full/`
- `configs/windows/`

**Directories Deleted**: 1
- `quick/llvm/windows/` (if empty)

**Code Changes**: 1
- Relative path: `../../../../lib/` → `../../../../../lib/`

---

## ⏱️ Time Estimate

- **Dry run**: ~10 seconds
- **Actual migration**: ~30 seconds
- **Verification**: ~2 minutes
- **Total**: **~3 minutes**

---

## 🔒 Safety Features

1. **Backup**: Automatic before any changes
2. **Git validation**: Requires clean working directory
3. **Rollback**: Automatic on error (via `trap ERR`)
4. **Dry run**: Preview mode available
5. **Verification**: Post-migration checks

---

## 📞 Quick Help

**Script location**: `tools/migrate_windows_tests.sh`

**Documentation**:
- Quick reference (this file)
- Detailed checklist: `tools/MIGRATION_CHECKLIST.md`
- Migration script help: `./tools/migrate_windows_tests.sh --help`

**Support**:
- Review existing structure: `tools/smokes/v2/profiles/`
- Check test runner: `tools/smokes/v2/lib/test_runner.sh`
- Windows profile README (after migration): `tools/smokes/v2/profiles/windows/README.md`

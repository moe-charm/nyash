# Windows Tests Migration Checklist

**Migration Date**: __________ (YYYY-MM-DD)
**Executed By**: __________
**Status**: [ ] Not Started / [ ] In Progress / [ ] Complete

---

## 📋 Pre-Migration Checklist

### Environment Verification
- [ ] Git repository is clean (`git status` shows no uncommitted changes)
- [ ] Current branch is correct (e.g., `selfhost` or feature branch)
- [ ] All existing tests pass before migration
- [ ] Backup strategy confirmed

### Test Execution Baseline
```bash
# Run existing Windows tests to establish baseline
cd /home/tomoaki/git/hakorune-selfhost
./tools/smokes/v2/profiles/quick/llvm/windows/wsl_windows_exe_smoke.sh

# Record results:
Exit Code: ____
Output: ____________________
```

- [ ] Baseline test results recorded
- [ ] Known failures documented (if any)

---

## 🔄 Migration Execution

### Step 1: Dry Run
```bash
./tools/migrate_windows_tests.sh --dry-run
```

**Expected Output**:
- [x] 7 steps shown (Step 1/7 through Step 7/7)
- [x] No errors reported
- [x] "Dry run verification passed" message
- [x] Files would be moved (not actually moved)

**Verification**:
- [ ] Dry run completed successfully
- [ ] No error messages
- [ ] Output looks correct

### Step 2: Actual Migration
```bash
./tools/migrate_windows_tests.sh
```

**Expected Output**:
- [x] Backup created with timestamp
- [x] Directory structure created
- [x] Test file moved via `git mv`
- [x] Relative paths updated
- [x] README.md created
- [x] Config files created
- [x] Old directory cleaned up
- [x] Verification passed

**Verification**:
- [ ] Migration script completed successfully (exit code 0)
- [ ] No error messages during execution
- [ ] Backup directory created at: `tools/smokes/v2/profiles/.backup_windows_YYYYMMDD_HHMMSS/`

---

## ✅ Post-Migration Verification

### File Structure Verification
```bash
# Check new structure exists
ls -la tools/smokes/v2/profiles/windows/quick/core/
ls -la tools/smokes/v2/profiles/windows/
ls -la tools/smokes/v2/configs/windows/
```

**Expected Files**:
- [ ] `tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh` (executable)
- [ ] `tools/smokes/v2/profiles/windows/README.md`
- [ ] `tools/smokes/v2/configs/windows/env.sh` (executable)
- [ ] `tools/smokes/v2/profiles/windows/full/` (directory, empty)

### Old Structure Cleanup
```bash
# Verify old directory is gone or empty
ls tools/smokes/v2/profiles/quick/llvm/windows/ 2>&1
```

**Expected**:
- [ ] Directory does not exist OR is empty
- [ ] No files left behind

### Git Status Verification
```bash
git status
```

**Expected Changes**:
- [ ] **Renamed**: `tools/smokes/v2/profiles/quick/llvm/windows/wsl_windows_exe_smoke.sh` → `tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh`
- [ ] **New file**: `tools/smokes/v2/profiles/windows/README.md`
- [ ] **New file**: `tools/smokes/v2/configs/windows/env.sh`
- [ ] **Deleted**: `tools/smokes/v2/profiles/quick/llvm/windows/` (if empty)

### Content Verification
```bash
# Check relative path was updated
grep -n "lib/test_runner.sh" tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh
```

**Expected**:
- [ ] Line contains: `../../../../../lib/test_runner.sh` (6 levels up)
- [ ] NO lines contain: `../../../../lib/test_runner.sh` (old 4 levels)

---

## 🧪 Functional Testing

### Test 1: Source the Moved Test File
```bash
# This tests the relative path fix
bash -n tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh
echo "Syntax check: $?"
```

**Expected**:
- [ ] Exit code 0 (syntax valid)
- [ ] No error messages

### Test 2: Run Via New Profile (if profile exists)
```bash
# This will test integration with profile system
./tools/smokes/v2/run.sh --profile windows-quick 2>&1 | tee migration_test.log
```

**Expected**:
- [ ] Test executes (may skip if prerequisites missing)
- [ ] No "file not found" errors
- [ ] Proper skip message if WSL/PowerShell not available

### Test 3: Direct Execution
```bash
# Direct test execution
cd tools/smokes/v2/profiles/windows/quick/core
bash windows_exe_basic.sh
```

**Expected**:
- [ ] Test runs (or properly skips with clear message)
- [ ] Exit code appropriate for result (0 = pass, 2 = skip, 1 = fail)

### Test 4: Compare with Baseline
Compare results with pre-migration baseline:
- [ ] Exit code matches baseline
- [ ] Output matches baseline (modulo path changes)
- [ ] Skip/pass/fail behavior unchanged

---

## 📊 Documentation Review

### README.md Quality Check
```bash
cat tools/smokes/v2/profiles/windows/README.md
```

**Verify**:
- [ ] Prerequisites section clear
- [ ] Test suite descriptions accurate
- [ ] Running instructions correct
- [ ] Troubleshooting section helpful
- [ ] No typos or formatting issues

### Config Quality Check
```bash
cat tools/smokes/v2/configs/windows/env.sh
```

**Verify**:
- [ ] Environment variables documented
- [ ] Defaults reasonable
- [ ] WSL interop check present
- [ ] File is executable (`-x` permission)

---

## 🎯 Final Commit

### Commit Preparation
```bash
# Review all changes
git diff --cached

# Check file modes
git ls-files --stage | grep windows
```

**Verify**:
- [ ] All expected files in staging area
- [ ] No unexpected changes
- [ ] File permissions correct (scripts executable)
- [ ] No temporary/backup files accidentally staged

### Commit Message
```bash
git commit -m "refactor(smokes): Migrate Windows tests to new structure

- Move wsl_windows_exe_smoke.sh → windows/quick/core/windows_exe_basic.sh
- Create Windows profile README with prerequisites and troubleshooting
- Add Windows-specific config (env.sh) with timeout defaults
- Fix relative paths for new directory depth (4→6 levels)
- Establish windows/quick/ and windows/full/ profile structure

Migration executed via tools/migrate_windows_tests.sh
All tests verified: [PASS/SKIP status]

Related: Windows test organization refactoring
"
```

**Verify**:
- [ ] Commit message clear and comprehensive
- [ ] Includes rationale
- [ ] Mentions migration tool
- [ ] Notes test verification status

### Post-Commit Actions
```bash
# Verify commit
git show --stat

# Optional: Delete backup after successful commit
rm -rf tools/smokes/v2/profiles/.backup_windows_*
```

- [ ] Commit created successfully
- [ ] Backup directory can be safely deleted
- [ ] Git log looks correct

---

## 🚨 Rollback Procedure (If Needed)

If migration fails or issues discovered:

### Automatic Rollback (During Migration)
If error occurs during script execution, rollback is automatic via `trap ERR`.

### Manual Rollback (After Migration)
```bash
# Reset git changes
git reset --hard HEAD

# Restore from backup
BACKUP_DIR="tools/smokes/v2/profiles/.backup_windows_YYYYMMDD_HHMMSS"  # Use actual timestamp
rm -rf tools/smokes/v2/profiles/quick/llvm/windows
cp -r "$BACKUP_DIR/windows" tools/smokes/v2/profiles/quick/llvm/windows

# Verify restoration
git status
./tools/smokes/v2/profiles/quick/llvm/windows/wsl_windows_exe_smoke.sh
```

**After Rollback**:
- [ ] Backup restored successfully
- [ ] Git working directory clean
- [ ] Original tests work again
- [ ] Issue documented for investigation

---

## 📈 Migration Statistics

**File Counts**:
- Files moved: ____
- Files created: ____
- Files deleted: ____
- Total lines changed: ____

**Test Results**:
- Pre-migration: [PASS/SKIP/FAIL]
- Post-migration: [PASS/SKIP/FAIL]
- Behavior match: [YES/NO]

**Time Spent**:
- Dry run: ____ minutes
- Actual migration: ____ minutes
- Verification: ____ minutes
- Documentation: ____ minutes
- **Total**: ____ minutes

---

## ✅ Sign-Off

**Migration Completed By**: __________
**Date**: __________
**Verified By**: __________ (if pair/review)

**Notes**:
_______________________________________________
_______________________________________________
_______________________________________________

**Overall Status**: [ ] SUCCESS / [ ] FAILED / [ ] ROLLED BACK

---

## 📚 References

- Migration script: `tools/migrate_windows_tests.sh`
- New profile location: `tools/smokes/v2/profiles/windows/`
- Backup location: `tools/smokes/v2/profiles/.backup_windows_*/`
- Windows profile README: `tools/smokes/v2/profiles/windows/README.md`

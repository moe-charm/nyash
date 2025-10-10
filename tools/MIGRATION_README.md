# Windows Tests Migration Tools

Automated migration system for safely moving Windows smoke tests to new profile structure.

## Quick Start

```bash
# 1. Preview what will happen (no changes made)
./tools/migrate_windows_tests.sh --dry-run

# 2. Execute migration (requires clean git)
./tools/migrate_windows_tests.sh

# 3. Verify and commit
git status
git commit -m "refactor(smokes): Migrate Windows tests to new structure"
```

**Total time: ~3 minutes**

---

## Files in This Directory

### Executable Scripts
- **`migrate_windows_tests.sh`** (511 lines, 15 KB)
  - Main migration automation script
  - Dry run mode, automatic backup, rollback on error
  - Usage: `./migrate_windows_tests.sh [--dry-run] [--help]`

### Documentation
- **`MIGRATION_README.md`** (this file)
  - Quick overview and navigation guide

- **`MIGRATION_QUICK_REFERENCE.md`** (171 lines, 4 KB)
  - 3-command quick start
  - Troubleshooting guide
  - Success indicators
  - **Use this for**: Quick execution without reading manuals

- **`MIGRATION_CHECKLIST.md`** (319 lines, 8 KB)
  - Comprehensive step-by-step verification
  - Pre/post migration checklists
  - Rollback procedures
  - **Use this for**: Thorough verification and sign-off

- **`MIGRATION_IMPLEMENTATION_SUMMARY.md`** (388 lines, 9.4 KB)
  - Complete implementation details
  - Architecture and design decisions
  - Metrics and statistics
  - **Use this for**: Understanding how the migration works

---

## What Gets Migrated

### Source Location
```
tools/smokes/v2/profiles/quick/llvm/windows/
└── wsl_windows_exe_smoke.sh
```

### Destination Location
```
tools/smokes/v2/profiles/windows/
├── README.md                          (NEW)
├── quick/
│   └── core/
│       └── windows_exe_basic.sh       (MOVED & RENAMED)
└── full/                              (EMPTY, future expansion)

tools/smokes/v2/configs/windows/
└── env.sh                             (NEW)
```

### Changes Applied
1. **Move**: `git mv` preserves file history
2. **Rename**: `wsl_windows_exe_smoke.sh` → `windows_exe_basic.sh`
3. **Path fix**: Relative path updated (4→6 directory levels)
4. **Documentation**: Comprehensive README with prerequisites
5. **Configuration**: Windows-specific environment settings

---

## Safety Features

### 1. Pre-Validation
- Git repository check
- Source file existence verification
- Clean working directory required (no uncommitted changes)

### 2. Automatic Backup
- Created before any changes
- Timestamped: `.backup_windows_YYYYMMDD_HHMMSS/`
- Full copy of source directory

### 3. Rollback on Error
- Automatic via `trap ERR` (during migration)
- Manual instructions provided (after migration)
- Backup preserved for recovery

### 4. Dry Run Mode
- Preview all operations without changes
- Test migration safety before execution
- Verify expected outcomes

### 5. Post-Migration Verification
- File existence checks
- Git status validation
- Executable permissions
- Path correctness

---

## Usage Scenarios

### Scenario 1: First-Time Migration (Recommended Path)
```bash
# Step 1: Preview (dry run)
./tools/migrate_windows_tests.sh --dry-run

# Step 2: Review output carefully
# Check that steps look correct

# Step 3: Execute migration
./tools/migrate_windows_tests.sh

# Step 4: Verify results
git status
git diff --cached

# Step 5: Test migrated file
bash -n tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh
tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh

# Step 6: Commit
git commit -m "refactor(smokes): Migrate Windows tests to new structure"

# Step 7: Clean up backup (after successful commit)
rm -rf tools/smokes/v2/profiles/.backup_windows_*
```

### Scenario 2: Migration with Uncommitted Changes
```bash
# Stash changes first
git stash

# Run migration
./tools/migrate_windows_tests.sh

# Restore stashed changes
git stash pop
```

### Scenario 3: Troubleshooting Mode
```bash
# Enable bash debug mode
bash -x ./tools/migrate_windows_tests.sh --dry-run

# Check specific step manually
source tools/migrate_windows_tests.sh
validate_preconditions
```

---

## Documentation Navigation

### I want to...

**Execute migration quickly**
→ Read: `MIGRATION_QUICK_REFERENCE.md` (3 commands)

**Verify each step thoroughly**
→ Use: `MIGRATION_CHECKLIST.md` (comprehensive checklist)

**Understand implementation details**
→ Read: `MIGRATION_IMPLEMENTATION_SUMMARY.md` (technical docs)

**Get help with script options**
→ Run: `./migrate_windows_tests.sh --help`

**Troubleshoot an issue**
→ See: `MIGRATION_QUICK_REFERENCE.md` → Troubleshooting section

---

## Expected Outcomes

### Successful Migration Indicators
✅ Script exit code 0
✅ "Migration verification passed" message
✅ Backup created: `.backup_windows_YYYYMMDD_HHMMSS/`
✅ Git status shows:
  - Renamed: `wsl_windows_exe_smoke.sh` → `windows_exe_basic.sh`
  - New: `windows/README.md`
  - New: `configs/windows/env.sh`
  - Deleted: `quick/llvm/windows/` (if empty)
✅ Test file works: `bash -n windows_exe_basic.sh` (exit 0)

### Common Issues

**"Git working directory is not clean"**
```bash
# Solution: Commit or stash changes first
git status
git stash  # or git commit
```

**"Source file not found"**
```bash
# Solution: Verify file exists or already migrated
ls tools/smokes/v2/profiles/quick/llvm/windows/wsl_windows_exe_smoke.sh
ls tools/smokes/v2/profiles/windows/quick/core/windows_exe_basic.sh
```

**Want to undo migration**
```bash
# Solution: Manual rollback
git reset --hard HEAD
# Restore from backup if needed
```

---

## Migration Steps (7 Steps)

The script executes these steps automatically:

1. **Create directory structure** - New profile directories
2. **Move test file** - Via `git mv` (preserves history)
3. **Fix relative paths** - Update `lib/test_runner.sh` path
4. **Create README** - Windows profile documentation
5. **Create config files** - Windows-specific settings
6. **Cleanup old directory** - Remove empty source directory
7. **Verify migration** - Validate all changes

Each step provides clear logging and error handling.

---

## Time Estimates

- **Dry run**: ~10 seconds
- **Actual migration**: ~30 seconds
- **Verification**: ~2 minutes
- **Total**: **~3 minutes**

---

## Statistics

### Implementation Metrics
- **Total lines**: 1,389 lines
  - Migration script: 511 lines (37%)
  - Documentation: 878 lines (63%)
- **Total size**: ~36 KB
- **Files created**: 4 files
- **Safety features**: 5 mechanisms
- **Verification checks**: 7 steps

### Migration Scope
- **Files moved**: 1 test file
- **Files created**: 2 (README, config)
- **Directories created**: 4
- **Code changes**: 1 (relative path)

---

## Support

### Getting Help

**Quick questions**: See `MIGRATION_QUICK_REFERENCE.md`

**Detailed verification**: Use `MIGRATION_CHECKLIST.md`

**Technical details**: Read `MIGRATION_IMPLEMENTATION_SUMMARY.md`

**Script options**: Run `./migrate_windows_tests.sh --help`

### Reporting Issues

If migration fails:
1. Check error message in output
2. Verify git status is clean
3. Try dry run mode first
4. Check rollback instructions
5. Review backup directory

### Contributing Improvements

Found an issue or improvement?
- Document in migration summary
- Update checklist if needed
- Submit issue/PR with details

---

## License & Credits

Part of the Hakorune selfhost project.

**Migration system created**: 2025-10-10
**Purpose**: Safe, automated Windows test migration
**Maintainer**: See project contributors

---

## Next Steps

1. ✅ **Read this README** (you are here)
2. 📖 **Review**: `MIGRATION_QUICK_REFERENCE.md`
3. 🧪 **Test**: `./migrate_windows_tests.sh --dry-run`
4. ✅ **Execute**: `./migrate_windows_tests.sh`
5. 🔍 **Verify**: Use checklist
6. 💾 **Commit**: Git commit changes

**Ready?** Start with the Quick Reference guide!

# docs/ Directory Cleanup Plan (Ultrathink Analysis)

**Date**: 2025-10-11
**Status**: Proposal
**Goal**: Simplify `docs/` root structure by consolidating small directories

---

## 📊 Current State Analysis

### Root-level Files (7 files)
| File | Lines | Status | Proposed Action |
|------|-------|--------|-----------------|
| `INDEX.md` | 269 | ✅ Keep | Entry point for all docs |
| `README.md` | 116 | ✅ Keep | Docs homepage |
| `changelog.md` | 15 | ⚠️ Misplaced | → `/CHANGELOG.md` (project root) |
| `claude_task.md` | 429 | ⚠️ Old | → `development/current/archive/` or delete |
| `directory-as-namespace.md` | 456 | ⚠️ Proposal | → `development/proposals/` |
| `mapbox-design-analysis.md` | 690 | ⚠️ Analysis | → `development/analysis/` |
| `phi_routes_explanation.md` | 100 | ⚠️ Guide | → `reference/mir/` or `guides/` |

---

### Subdirectories (19 dirs)

| Directory | Files | Status | Proposed Action |
|-----------|-------|--------|-----------------|
| **Large (Keep)** | | | |
| `development/` | 484 | ✅ Keep | Main development docs |
| `archive/` | 489 | ✅ Keep | Historical docs |
| `private/` | 778 | ⚠️ Review | Archive or delete? |
| `reference/` | 103 | ✅ Keep | API/language reference |
| `guides/` | 95 | ✅ Keep | User guides |
| `papers/` | 4 | ✅ Keep | Research papers |
| **Small (Consolidate)** | | | |
| `abi/` | 1 | ❌ Merge | → `reference/abi/` |
| `architecture/` | 3 | ❌ Merge | → `development/architecture/` |
| `benchmarks/` | 2 | ❌ Merge | → `development/benchmarks/` |
| `bugs/` | 1 | ❌ Merge | → `development/issues/` |
| `config/` | 1 | ❌ Merge | → `reference/config/` or `guides/` |
| `cookbook/` | 1 | ❌ Merge | → `guides/` (quick-tips) |
| `design/` | 5 | ❌ Merge | → `development/design/` |
| `investigation/` | 2 | ❌ Merge | → `development/analysis/` |
| `mir/` | 1 | ❌ Merge | → `reference/mir/` |
| `runtime/` | 2 | ❌ Merge | → `reference/runtime/` or `development/` |
| `smokes/` | 1 | ❌ Merge | → `tools/smokes/docs/` |
| `tools/` | 11 | ⚠️ Split | Distribute to appropriate locations |
| `assets/` | 1 | ✅ Keep | Images/diagrams |

---

## 🎯 Cleanup Goals

### Goal 1: Reduce Root-level Clutter
**Before**: 7 markdown files in `docs/`
**After**: 2 markdown files (`INDEX.md`, `README.md`)

**Actions**:
1. Move `changelog.md` → `/CHANGELOG.md` (project root)
2. Move `claude_task.md` → `development/current/archive/claude_task_20251011.md`
3. Move `directory-as-namespace.md` → `development/proposals/`
4. Move `mapbox-design-analysis.md` → `development/analysis/`
5. Move `phi_routes_explanation.md` → `reference/mir/phi-routes.md`

---

### Goal 2: Consolidate Small Directories
**Before**: 19 subdirectories (many with 1-5 files)
**After**: 8-10 main directories

**Consolidation Plan**:

#### 1. `docs/abi/` (1 file) → `docs/reference/abi/`
```bash
mkdir -p docs/reference/abi
mv docs/abi/vm-kernel.md docs/reference/abi/
rmdir docs/abi
```

#### 2. `docs/architecture/` (3 files) → `docs/development/architecture/`
```bash
mv docs/architecture/* docs/development/architecture/
rmdir docs/architecture
```

#### 3. `docs/benchmarks/` (2 files) → `docs/development/benchmarks/`
```bash
mkdir -p docs/development/benchmarks
mv docs/benchmarks/* docs/development/benchmarks/
rmdir docs/benchmarks
```

#### 4. `docs/bugs/` (1 file) → `docs/development/issues/`
```bash
mv docs/bugs/mapbox-get-null-comparison-bug.md docs/development/issues/
rmdir docs/bugs
```

#### 5. `docs/config/` (1 file) → `docs/guides/configuration.md`
```bash
mv docs/config/env.md docs/guides/configuration.md
rmdir docs/config
```

#### 6. `docs/cookbook/` (1 file) → `docs/guides/quick-tips.md`
```bash
mv docs/cookbook/quick-tips.md docs/guides/
rmdir docs/cookbook
```

#### 7. `docs/design/` (5 files) → `docs/development/design/`
```bash
mv docs/design/* docs/development/design/
rmdir docs/design
```

#### 8. `docs/investigation/` (2 files) → `docs/development/analysis/`
```bash
mv docs/investigation/* docs/development/analysis/
rmdir docs/investigation
```

#### 9. `docs/mir/externs` → `docs/reference/mir/externs`
```bash
mv docs/mir/externs docs/reference/mir/
rmdir docs/mir
```

#### 10. `docs/runtime/` (2 dirs) → `docs/reference/runtime/`
```bash
mkdir -p docs/reference/runtime
mv docs/runtime/provider_box docs/reference/runtime/
mv docs/runtime/static_plugins docs/reference/runtime/
rmdir docs/runtime
```

#### 11. `docs/smokes/` (1 file) → `tools/smokes/docs/`
```bash
mkdir -p tools/smokes/docs
mv docs/smokes/quick-gates.md tools/smokes/docs/
rmdir docs/smokes
```

#### 12. `docs/tools/` (11 files) → Distribute
```bash
# CLI/tool docs → docs/guides/
mv docs/tools/cli-options.md docs/guides/
mv docs/tools/modules-cli.md docs/guides/
mv docs/tools/nyash-help.md docs/guides/

# LLVM/build docs → docs/guides/
mv docs/tools/llvm-build.md docs/guides/

# VSCode/IDE → docs/guides/
mv docs/tools/vscode-hako.md docs/guides/

# Claude-specific → docs/development/
mv docs/tools/claude-issues.md docs/development/

# Others → docs/guides/ or docs/reference/
mv docs/tools/using-quickstart.md docs/guides/
mv docs/tools/vm-stats-cookbook.md docs/guides/
mv docs/tools/codex-android-setup.md docs/guides/

# nyfmt → docs/reference/tools/ (if needed)
mkdir -p docs/reference/tools
mv docs/tools/nyfmt docs/reference/tools/

rmdir docs/tools
```

---

## 📁 Proposed Final Structure

```
docs/
├── INDEX.md                   # Entry point
├── README.md                  # Docs homepage
├── development/               # 484 files (+ 20 from consolidation)
│   ├── architecture/          # ← from docs/architecture/
│   ├── analysis/              # ← from docs/investigation/
│   ├── benchmarks/            # ← from docs/benchmarks/
│   ├── design/                # ← from docs/design/
│   ├── issues/                # ← from docs/bugs/
│   ├── proposals/             # ← + directory-as-namespace.md, mapbox-design-analysis.md
│   ├── current/
│   │   └── archive/           # ← claude_task.md
│   └── roadmap/
├── reference/                 # 103 files (+ 5 from consolidation)
│   ├── abi/                   # ← from docs/abi/
│   ├── mir/                   # ← from docs/mir/, + phi_routes_explanation.md
│   ├── runtime/               # ← from docs/runtime/
│   ├── tools/                 # ← nyfmt
│   ├── language/
│   └── boxes-system/
├── guides/                    # 95 files (+ 9 from consolidation)
│   ├── configuration.md       # ← from docs/config/env.md
│   ├── quick-tips.md          # ← from docs/cookbook/
│   ├── cli-options.md         # ← from docs/tools/
│   ├── modules-cli.md         # ← from docs/tools/
│   ├── nyash-help.md          # ← from docs/tools/
│   ├── llvm-build.md          # ← from docs/tools/
│   ├── vscode-hako.md         # ← from docs/tools/
│   ├── using-quickstart.md    # ← from docs/tools/
│   ├── vm-stats-cookbook.md   # ← from docs/tools/
│   └── codex-android-setup.md # ← from docs/tools/
├── archive/                   # 489 files (keep as-is)
├── private/                   # 778 files (review later)
├── papers/                    # 4 files (keep)
└── assets/                    # 1 file (keep)
```

**Result**:
- **Before**: 19 subdirectories
- **After**: 8 main directories (development, reference, guides, archive, private, papers, assets, [deprecated])

---

## 🚀 Execution Plan

### Phase 1: Root-level Files (5 min)
```bash
# 1. Move changelog to project root
mv docs/changelog.md CHANGELOG.md

# 2. Archive Claude task
mkdir -p docs/development/current/archive
mv docs/claude_task.md docs/development/current/archive/claude_task_20251011.md

# 3. Move proposals
mv docs/directory-as-namespace.md docs/development/proposals/
mv docs/mapbox-design-analysis.md docs/development/analysis/

# 4. Move MIR guide
mv docs/phi_routes_explanation.md docs/reference/mir/phi-routes.md
```

### Phase 2: Small Directory Consolidation (10 min)
Run the consolidation commands above (1-12).

### Phase 3: Update INDEX.md (5 min)
Update `docs/INDEX.md` to reflect new structure.

### Phase 4: Verification (5 min)
```bash
# Check for broken links
grep -r "\](docs/" docs/ | grep -v "\.git"

# Verify file counts
find docs/ -maxdepth 1 -type f | wc -l  # Should be 2 (INDEX.md, README.md)
find docs/ -maxdepth 1 -type d | wc -l  # Should be ~10
```

---

## ⚠️ Risks & Mitigation

### Risk 1: Broken Links
**Mitigation**: Run link checker after consolidation
```bash
# Find all markdown links
grep -r "\](docs/" docs/ --include="*.md"
```

### Risk 2: Loss of History
**Mitigation**: Commit BEFORE cleanup
```bash
git add docs/
git commit -m "docs: snapshot before consolidation"
```

### Risk 3: Concurrent Edits
**Mitigation**: Coordinate with ChatGPT, do cleanup in single session

---

## 📊 Expected Benefits

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Root-level MD files | 7 | 2 | **-71%** |
| Subdirectories | 19 | 8-10 | **-47-58%** |
| Empty/near-empty dirs | 12 | 0 | **-100%** |
| Clarity | ⚠️ Confusing | ✅ Clear | **Much better** |

---

## 🎯 Next Steps

1. **Review Plan**: Get user confirmation
2. **Create Backup**: `git commit -m "docs: pre-cleanup snapshot"`
3. **Execute Phase 1-2**: Run consolidation commands
4. **Update Links**: Fix broken references
5. **Commit**: `git commit -m "docs: consolidate small directories"`

---

**Version**: 1.0
**Author**: Claude (Ultrathink Analysis, 2025-10-11)

# Documentation Cleanup Plan (2025-10-04)

## 🚨 Current State (深刻な状況)

調査日: 2025-10-04
調査者: Claude Code

### 📊 Statistics (統計)

**Total**:
- Markdown files: **1,161 files**
- Directories: **558 directories**
- README.md: **153 files** (過剰)

**By Directory**:
| Directory | Files | Percentage |
|-----------|-------|------------|
| `docs/archive/` | 431 | 37% |
| `docs/development/` | 321 | 28% |
| `docs/private/` | 223 | 19% |
| **Top 3 Total** | **975** | **84%** |
| Other | 186 | 16% |

### 🔍 Discovered Problems (発見した問題)

#### 1. **Double Nesting (二重ネスト)** - 7 cases

```
docs/archive/phases/phase-10.1/phase-10.1/
docs/archive/phases/phase-10.5/phase-10.5/
docs/archive/phases/phase-10.6/phase-10.6/
docs/archive/phases/phase-10.7/phase-10.7/
docs/archive/phases/phase-11.5/phase-11.5/
docs/archive/phases/phase-12.5/phase-12.5/
docs/archive/phases/phase-15/phase-15.1/
```

**Impact**: Confusion, hard to navigate
**Priority**: P1 (High) - Easy to fix

#### 2. **Excessive README.md (過剰なREADME.md)** - 153 files

**Examples**:
- Every phase has multiple READMEs (各フェーズに複数)
- Nested READMEs in subdirectories
- Some empty or duplicate content

**Impact**: Hard to find the right README
**Priority**: P2 (Medium) - Need careful review

#### 3. **Duplicate Filenames (重複ファイル名)** - Many cases

Same filename in different directories creates confusion.

**Examples** (partial list):
- `00_MASTER_ROADMAP.md` (multiple locations)
- `AI_CONFERENCE_*.md` (scattered)
- `BLUEPRINT_MIN.md` (duplicates)

**Impact**: Which one is current? Which to use?
**Priority**: P2 (Medium) - Need consolidation

#### 4. **Archive Size (アーカイブが大きすぎ)** - 431 files (37%)

**Observation**:
- All files timestamp: 2025-09-06 11:10:41 (bulk move)
- Contains old phases (phase-6 to phase-14)
- Some may still be referenced

**Impact**: Slow navigation, git overhead
**Priority**: P3 (Low) - Consider separate repo

#### 5. **Similar Directory Names (似たディレクトリ名)**

Potential duplication/confusion:
- `docs/architecture/` vs `docs/development/architecture/`
- `docs/design/` vs `docs/development/design/`
- `docs/archive/proposals/` vs `docs/development/proposals/`

**Impact**: Where to put new docs? Confusion
**Priority**: P1 (High) - Need clear guidelines

---

## 🎯 Cleanup Strategy (整理戦略)

### Phase 1: Quick Wins (即座に実行可能) ✅

**Priority**: P0 (Immediate)
**Risk**: Very Low
**Impact**: High visibility

#### 1.1 Fix Double Nesting (二重ネスト解消)

```bash
# Example for phase-10.5
mv docs/archive/phases/phase-10.5/phase-10.5/* docs/archive/phases/phase-10.5/
rmdir docs/archive/phases/phase-10.5/phase-10.5/
```

**Affected**: 7 directories
**Time**: 10 minutes
**Commit**: One commit per fix (細かくコミット)

#### 1.2 Create Documentation Index (ドキュメントインデックス作成)

Create `docs/INDEX.md` with:
- Overview of directory structure
- Guidelines for where to put new docs
- Links to most important docs

**Time**: 30 minutes
**Commit**: Single commit

---

### Phase 2: Consolidation (統合) ⚠️

**Priority**: P1 (High)
**Risk**: Medium (需要確認必要)
**Impact**: Better organization

#### 2.1 Merge Similar Directories (似たディレクトリの統合)

**Proposal**:
- Keep `docs/development/` as primary
- Move `docs/architecture/` → `docs/development/architecture/`
- Move `docs/design/` → `docs/development/design/`

**Before**:
```
docs/
  architecture/
  development/
    architecture/
  design/
  development/
    design/
```

**After**:
```
docs/
  development/
    architecture/
    design/
```

**Prerequisites**:
1. Check for duplicate files
2. Merge or choose which to keep
3. Update links

**Time**: 1-2 hours
**Commits**: One per directory merge

#### 2.2 Consolidate Duplicate Files (重複ファイル統合)

**Process**:
1. Identify duplicates: `find docs -name "*.md" -exec basename {} \; | sort | uniq -d`
2. For each duplicate:
   - Compare content: `diff file1 file2`
   - Keep newer/better one
   - Delete or archive old one
   - Update links

**Time**: 2-3 hours (depends on count)
**Commits**: One per file group

---

### Phase 3: Archive Optimization (アーカイブ最適化) 🗄️

**Priority**: P3 (Low)
**Risk**: Low
**Impact**: Reduced clutter

#### 3.1 Compress Old Phases (古いフェーズの圧縮)

**Proposal**: Move very old phases to separate repo or zip

**Candidates**:
- phase-6 through phase-10 (completed 2024-2025)
- AI conference logs (historical value only)
- Build logs (can regenerate if needed)

**Options**:
1. **Separate repo**: `hakorune-archive`
2. **Zip files**: `archive-phase-6-to-10.tar.gz`
3. **Keep as-is**: If referenced frequently

**Decision**: Requires user input

**Time**: 1-2 hours (if decided to move)
**Commits**: One big archive commit

#### 3.2 README Cleanup (README整理)

**Proposal**:
- One README.md per phase
- Remove empty READMEs
- Merge duplicate READMEs

**Process**:
1. List all READMEs: `find docs -name "README.md"`
2. Check each: empty, duplicate, or useful?
3. Consolidate

**Time**: 3-4 hours
**Commits**: One per directory cleaned

---

## 📋 Execution Order (実行順序)

### Week 1 (今週)

✅ **Day 1**: Phase 1.1 - Fix double nesting (7 directories)
✅ **Day 2**: Phase 1.2 - Create INDEX.md
⚠️ **Day 3-4**: Phase 2.1 - Merge similar directories (with review)

### Week 2 (来週)

⚠️ **Day 1-2**: Phase 2.2 - Consolidate duplicate files
📊 **Day 3**: Review progress, decide on Phase 3
🗄️ **Day 4-5**: Phase 3 (if decided)

---

## 🛡️ Safety Measures (安全対策)

### Before Any Change

1. ✅ **Git commit**: Working tree clean
2. ✅ **Backup**: Create branch `docs-cleanup-backup`
3. ✅ **Test**: Check important links still work

### During Change

1. ✅ **Small commits**: One change at a time
2. ✅ **Clear messages**: Explain what and why
3. ✅ **Reversible**: Can `git revert` easily

### After Change

1. ✅ **Link check**: Run link checker (if available)
2. ✅ **Smoke test**: Open important docs
3. ✅ **Document**: Update INDEX.md if needed

---

## 🎯 Success Criteria (成功基準)

### Phase 1 (Quick Wins)

- [ ] No double-nested directories
- [ ] INDEX.md exists and accurate
- [ ] All commits clean (no broken links)

### Phase 2 (Consolidation)

- [ ] No duplicate top-level directories
- [ ] Duplicate files reduced by 50%+
- [ ] Clear guidelines for new docs

### Phase 3 (Archive)

- [ ] Archive size reduced (if applicable)
- [ ] Old content still accessible
- [ ] README count < 80 (from 153)

---

## 📚 Related Documents

- `dead-code-analysis.md` - Code cleanup analysis
- `todo-fixme-inventory.md` - TODO tracking
- Future: `docs/INDEX.md` - Master documentation index

---

## 🚨 Risks & Mitigation (リスクと対策)

### Risk 1: Broken Links

**Likelihood**: High
**Impact**: High
**Mitigation**:
- Use `grep -r` to find references before moving
- Update links in same commit as move
- Test major docs after change

### Risk 2: Lost Content

**Likelihood**: Low (with git)
**Impact**: High
**Mitigation**:
- Never `rm`, always `git rm` (stays in history)
- Create backup branch
- Review diffs before commit

### Risk 3: User Confusion

**Likelihood**: Medium
**Impact**: Medium
**Mitigation**:
- Clear commit messages
- Update INDEX.md
- Announce changes in CURRENT_TASK.md

---

## 🤝 Next Steps (次のステップ)

### User Decision Required

1. **Archive Strategy**: Keep, move to separate repo, or compress?
2. **Merge Priority**: Which directories to merge first?
3. **Timeline**: Aggressive (1 week) or conservative (2-3 weeks)?

### Immediate Actions (User Approved)

1. ✅ Fix double nesting (7 directories) - **Low Risk**
2. ✅ Create INDEX.md - **Low Risk**
3. ⏸️ Wait for approval on Phase 2 & 3

---

Generated by: Claude Code (Anthropic)
Date: 2025-10-04
Status: Proposal (Awaiting user approval)

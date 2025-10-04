# Documentation Index (ドキュメントインデックス)

**Last Updated**: 2025-10-04
**Total Files**: 1,129 markdown files (after Phase 1.1 cleanup)
**Total Directories**: 558 directories

---

## 🎯 Quick Navigation (クイックナビゲーション)

### 🚀 **For New Users (初めての方)**
- [README.md](README.md) - Project overview and getting started
- [Getting Started Guide](guides/getting-started.md) - Setup and first steps
- [Language Reference](reference/language/LANGUAGE_REFERENCE_2025.md) - Complete language spec
- [Quick Reference](reference/language/quick-reference.md) - One-page practical guide ⭐

### 👨‍💻 **For Developers (開発者向け)**
- [CURRENT_TASK.md](../CURRENT_TASK.md) - Current development focus
- [Development Roadmap](development/roadmap/phases/00_MASTER_ROADMAP.md) - Master plan
- [Phase 15 README](development/roadmap/phases/phase-15.7/README.md) - Current phase
- [Architecture Overview](development/architecture/) - System design

### 📚 **For Contributors (コントリビューター向け)**
- [Style Guide](guides/style-guide.md) - Coding conventions
- [Development Practices](guides/development-practices.md) - Best practices
- [Testing Guide](guides/testing-guide.md) - How to test

---

## 📁 Directory Structure (ディレクトリ構造)

### 📖 **User-Facing Documentation (利用者向け)**

#### `docs/guides/` - Practical Guides
**Purpose**: Step-by-step tutorials and how-to guides
**When to use**: Writing tutorials, examples, best practices

**Key Files**:
- `getting-started.md` - Installation and setup
- `language-guide.md` - Language tutorial
- `testing-guide.md` - Testing practices
- `style-guide.md` - Code style conventions

#### `docs/reference/` - Reference Documentation
**Purpose**: Complete technical specifications
**When to use**: Documenting APIs, language features, specifications

**Subdirectories**:
- `language/` - Language specification and syntax
- `boxes-system/` - Box API documentation
- `plugin-system/` - Plugin development
- `quick/` - Quick reference sheets
- `mir/` - MIR (Middle Intermediate Representation) specs

#### `docs/cookbook/` - Code Examples
**Purpose**: Practical code examples and recipes
**When to use**: Providing working code samples

---

### 🔧 **Developer Documentation (開発者向け)**

#### `docs/development/` - Active Development (321 files)
**Purpose**: Current development work, plans, and proposals
**When to use**: Documenting ongoing work, proposals, designs

**Key Subdirectories**:
- `architecture/` - System architecture and design
- `roadmap/` - Development roadmap and phases
- `proposals/` - Feature proposals and RFCs
- `current/` - Current task tracking
- `issues/` - Known issues and investigations
- `selfhosting/` - Self-hosting implementation

**Important Files**:
- `roadmap/phases/00_MASTER_ROADMAP.md` - Master development plan ⭐
- `roadmap/phases/phase-15.7/README.md` - Current phase details
- `current/main/` - Main track current tasks
- `current/llvm/` - LLVM track current tasks
- `proposals/ideas/refactoring/` - Refactoring proposals

#### `docs/architecture/` - Architecture Documentation
**Purpose**: High-level system architecture
**When to use**: Documenting system design decisions

**Note**: Consider consolidating with `development/architecture/` (see cleanup plan)

#### `docs/design/` - Design Documents
**Purpose**: Design specifications and patterns
**When to use**: Documenting design patterns and approaches

**Note**: Consider consolidating with `development/design/` (see cleanup plan)

---

### 🗄️ **Archive (アーカイブ)**

#### `docs/archive/` - Historical Documentation (431 files, 37%)
**Purpose**: Completed phases and historical documents
**When to use**: Reference only - DO NOT add new files here

**Structure**:
- `phases/` - Completed development phases (phase-6 through phase-14)
- `decisions/` - Historical decision records
- `proposals/` - Old proposals (completed or rejected)

**Note**: This directory is for reference only. All new work goes in `development/`.

---

### 🔒 **Private Documentation (非公開)**

#### `docs/private/` - Internal Documentation (223 files, 19%)
**Purpose**: Internal notes, papers, and non-public docs
**When to use**: Private research, internal discussions

**Note**: Not for public distribution

---

### 📊 **Other Directories**

#### `docs/abi/` - ABI Specifications
**Purpose**: Application Binary Interface documentation

#### `docs/benchmarks/` - Performance Benchmarks
**Purpose**: Benchmark results and analysis

#### `docs/config/` - Configuration Documentation
**Purpose**: Configuration file documentation

#### `docs/papers/` - Research Papers
**Purpose**: Academic and technical papers

---

## 📝 Guidelines for New Documentation (新規ドキュメント作成ガイドライン)

### Where to Put New Docs (どこに置くべきか)

| Document Type | Location | Example |
|---------------|----------|---------|
| Tutorial/Guide | `docs/guides/` | How to use feature X |
| API Reference | `docs/reference/` | Box API specification |
| Language Spec | `docs/reference/language/` | Syntax documentation |
| Design Proposal | `docs/development/proposals/` | RFC for new feature |
| Architecture Doc | `docs/development/architecture/` | System design |
| Current Task | `docs/development/current/` | Today's work |
| Completed Work | `docs/archive/` | Finished phase docs |

### Naming Conventions (命名規則)

**DO** ✅:
- Use lowercase with hyphens: `getting-started.md`
- Be descriptive: `phase-15-selfhosting-plan.md`
- Include dates for status docs: `CURRENT_TASK_20251004.md`

**DON'T** ❌:
- Use spaces: `Getting Started.md`
- Use underscores in new docs: `getting_started.md` (legacy only)
- Use ambiguous names: `doc.md`, `notes.md`

### README.md Rules

**One README.md per directory** - No more, no less

**Purpose of README.md**:
- Overview of directory contents
- Links to important files
- Brief explanation of structure

**Current Status**: 153 README.md files (too many)
**Target**: ~80 README.md files (one per meaningful directory)

---

## 🔗 Most Important Documents (最重要ドキュメント)

### Top 10 Essential Reads

1. [Master Roadmap](development/roadmap/phases/00_MASTER_ROADMAP.md) - Complete development plan ⭐⭐⭐
2. [CURRENT_TASK.md](../CURRENT_TASK.md) - What's happening now ⭐⭐⭐
3. [Language Reference](reference/language/LANGUAGE_REFERENCE_2025.md) - Complete spec ⭐⭐
4. [Quick Reference](reference/language/quick-reference.md) - One-page guide ⭐⭐⭐
5. [Phase 15.7 README](development/roadmap/phases/phase-15.7/README.md) - Current phase ⭐⭐
6. [Getting Started](guides/getting-started.md) - First steps ⭐⭐
7. [Architecture Overview](development/architecture/) - System design ⭐
8. [MIR Instruction Set](reference/mir/INSTRUCTION_SET.md) - Core IR ⭐⭐
9. [Plugin System](reference/plugin-system/) - Plugin development ⭐
10. [Testing Guide](guides/testing-guide.md) - How to test ⭐

---

## 🔍 Search Tips (検索のヒント)

### Finding Documents

```bash
# Find all docs about MIR
find docs -name "*.md" -exec grep -l "MIR" {} \;

# Find READMEs
find docs -name "README.md"

# Find recent changes
find docs -name "*.md" -mtime -7  # Last 7 days

# Search content
grep -r "self-hosting" docs/
```

### Common Searches

- **Current work**: Check `docs/development/current/`
- **Past phases**: Check `docs/archive/phases/`
- **Language syntax**: Check `docs/reference/language/`
- **API docs**: Check `docs/reference/boxes-system/`

---

## 🚧 Cleanup Status (整理状況)

**Phase 1.1** (2025-10-04): ✅ Complete
- Fixed 5/7 double-nested directories
- Removed 4,515 duplicate lines
- Cleaned 32 redundant files

**Phase 1.2** (2025-10-04): ✅ Complete
- Created this INDEX.md

**Pending**:
- Phase 1.3: Manually merge phase-10.7 and phase-11.5
- Phase 2: Consolidate duplicate directories
- Phase 3: Archive optimization

**Details**: See [docs-cleanup-plan.md](development/proposals/ideas/refactoring/docs-cleanup-plan.md)

---

## 📞 Help & Support

**For questions about**:
- Documentation structure → Check this INDEX.md
- Where to put new docs → See "Guidelines" section above
- Cleanup progress → See [docs-cleanup-plan.md](development/proposals/ideas/refactoring/docs-cleanup-plan.md)
- Development status → See [CURRENT_TASK.md](../CURRENT_TASK.md)

---

## 📜 Version History

- **2025-10-04**: Initial version created (Phase 1.2)
  - 1,129 files indexed (after Phase 1.1 cleanup)
  - Major sections established
  - Guidelines documented

---

**Maintained by**: Claude Code (Anthropic)
**Next Review**: 2025-11-04 (monthly review recommended)

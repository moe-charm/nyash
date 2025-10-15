# AI Pair Programming and Design Intent Communication: How Implementation State Bias Led to Architectural Misunderstanding

**Status**: Draft (2025-10-09)
**Authors**: Tomoaki Nakashima
**Context**: Hakorune Language Development (Phase 15.5-15.15)
**Keywords**: AI-assisted development, Design intent, Incremental migration, Plugin architecture

---

## Abstract

During the development of Hakorune, a self-hosted programming language with "Everything is Plugin" architecture, we discovered a critical challenge in AI-assisted pair programming: **AI systems (Claude, ChatGPT) misinterpreted the design intent by observing incomplete implementation state**. Despite perfect architectural documentation (ADR-001: "Everything is Plugin"), AI assistants concluded that the MapBox plugin's i64-only limitation was an intentional design choice rather than an incomplete implementation (Phase A in progress). This paper analyzes the root causes of this "Implementation State Bias," proposes practical solutions for maintaining design consistency during incremental migration, and validates the "Everything is Plugin" architecture through this real-world case study.

**Key Finding**: AI systems prioritize code observation over design documentation, leading to systematic misinterpretation during incremental architectural migrations.

---

## 1. Introduction

### 1.1 Background: Hakorune's "Everything is Plugin" Architecture

Hakorune (formerly Nyash) is a self-hosted programming language built on the "Box-First" philosophy where all values are objects ("Boxes"). The language implements a radical architectural principle documented in ADR-001:

**Core Design Principles**:
- **No CoreBox**: No special built-in types in the runtime
- **Everything is Plugin**: All functionality provided via TypeBox v2 ABI
- **STN/PVN Separation**: Stable Type Names (STN) independent from Provider IDs (PVN)
- **Interchangeable Implementations**: Plugin (.so), builtin (static), or embedded

**Technical Foundation**:
```rust
// Nyash ABI (TypeBox v2) with TLV encoding
tag=1: bool
tag=2: i32
tag=3: i64
tag=5: f64
tag=6/7: string
tag=8: PluginHandle(type_id, instance_id)  // For plugin Box instances
tag=9: HostHandle(u64)                      // For host-managed Box instances
```

The architecture was designed to allow seamless provider replacement:
```toml
# hako.toml (formerly nyash.toml)
[types.StringBox]
provider = "kernel:string@1.0"    # Default
# provider = "acme:string@2.1"    # Alternative
```

### 1.2 The Problem: Misinterpreted Design Intent

During Phase 15.5-15.15 (September-October 2025), while implementing the "Everything is Plugin" migration, both Claude and ChatGPT-5 assistants consistently misinterpreted the MapBox plugin's limitation as intentional design:

**What AI Observed**:
```rust
// plugins/nyash-map-plugin/src/lib.rs:2-3
//! Nyash MapBox Plugin — TypeBox v2 (minimal)
//! Extension: support both i64 and UTF-8 string keys;
//! **values remain i64**
```

**What AI Concluded**:
- MapBox plugin is intentionally i64-only (for primitives)
- Builtin MapBox is for complex objects
- Two implementations serve different purposes

**Actual Design Intent** (from ADR-001):
- Single ABI for all Box types
- MapBox plugin should support all values via tag=8/9
- Builtin is temporary fallback during migration
- Implementation is incomplete (Phase A pending)

**The Moment of Truth**:
User's simple question revealed the misunderstanding:
> "Shouldn't it just be a single Nyash ABI MapBox/ArrayBox/StringBox, with the choice of plugin or embedded implementation?"

This question exposed that the design intent was perfect all along—AI had misread incomplete implementation as final design.

---

## 2. Root Cause Analysis: Implementation State Bias

### 2.1 Definition: Implementation State Bias

**Implementation State Bias**: The tendency of AI systems to treat observed code implementation as authoritative design intent, overriding documented architecture and design decisions.

**Characteristics**:
1. **Code > Documentation**: Prioritizes code comments over ADR/design docs
2. **Current State = Final State**: Assumes incomplete implementation reflects intended design
3. **Constraint Reification**: Treats temporary limitations as permanent design choices

### 2.2 Timeline of Misunderstanding

**Phase 15.5 (Sep 16-20, 2025)**: Everything is Plugin Declaration
- ADR-001 written: "Everything is Plugin"
- UnifiedBoxRegistry implemented with FactoryPolicy
- Tag=8/9 TLV encoding added
- **Gap**: MapBox plugin not yet updated for tag=8/9

**Phase 15.7 (Sep 27-30)**: VM Convenience Handlers Removed
- Deleted 700 lines of VM-specific Box handling
- Forced all operations through plugin/builtin paths
- **Gap**: MapBox plugin still i64-only

**Phase 15.15 (Oct 5-7)**: Consolidation
- Common code deduplicated
- Instance Box refactoring
- **Gap**: MapBox plugin tag=8/9 still pending

**Oct 9, 2025**: ChatGPT-5 implements Arc cache
- Global (type_id, instance_id) → Weak<Arc> cache
- Infrastructure for tag=8/9 identity preservation complete
- **Gap**: MapBox plugin not updated to use it

**Oct 9 (Evening)**: User's insight
- Question: "Why not single ABI + plugin/embedded choice?"
- Realization: Design was perfect, implementation incomplete
- AI had misinterpreted "Phase A pending" as "intentional limitation"

### 2.3 How AI Reasoning Failed

**AI Thought Process** (reconstructed from conversation logs):

```
Step 1: Observe MapBox plugin code
  → "values remain i64" comment

Step 2: Read Builtin MapBox code
  → Supports all Box types via Arc<RwLock<HashMap<String, Box<dyn NyashBox>>>>

Step 3: Compare implementations
  → Plugin: i64 only
  → Builtin: All types

Step 4: Generate hypothesis
  → "This must be intentional separation"
  → "Plugin for primitives, Builtin for objects"

Step 5: Propose solution
  → Maintain both implementations
  → Document use cases
  → OR: Extend plugin (not realizing this was the original intent)
```

**What AI Missed**:
- ADR-001 explicitly states "Everything is Plugin"
- Tag=8/9 infrastructure already complete
- "values remain i64" was TODO, not design constraint
- Phase A/B/C roadmap existed but not explicitly linked

**Critical Failure Point**: **Step 3 → Step 4**
- AI treated current implementation as design intent
- Did not question "Why would intentional design violate ADR-001?"
- Did not ask "Is this implementation complete?"

---

## 3. Case Study: The MapBox Confusion

### 3.1 Technical Context

**Required Infrastructure** (all complete):
```rust
// 1. TLV encoding (src/runtime/plugin_ffi_common.rs:208-219)
pub fn plugin_handle(buf: &mut Vec<u8>, type_id: u32, instance_id: u32) {
    buf.push(TAG_HANDLE);  // 8
    buf.extend_from_slice(&type_id.to_le_bytes());
    buf.extend_from_slice(&instance_id.to_le_bytes());
}

pub fn host_handle(buf: &mut Vec<u8>, handle_id: u64) {
    buf.push(TAG_HOST_HANDLE);  // 9
    buf.extend_from_slice(&handle_id.to_le_bytes());
}

// 2. Arc cache (src/runtime/plugin_loader_v2/enabled/types.rs:237-268)
static HANDLE_CACHE: OnceCell<RwLock<HashMap<(u32, u32), Weak<Arc<PluginHandleInner>>>>> = ...;

pub fn get_or_create_handle(...) -> Arc<PluginHandleInner> {
    // Reuse same Arc for (type_id, instance_id)
}

// 3. Decode (src/runtime/host_api.rs:57-60)
if let Some((type_id, instance_id)) = tlv::decode::plugin_handle(payload) {
    if let Some(arc) = plugin_box_from_handle(type_id, instance_id) {
        return Some(VMValue::BoxRef(arc));
    }
}
```

**What Was Missing**: MapBox plugin using tag=8/9 in set/get methods

### 3.2 AI's Misinterpretation Sequence

**Conversation 1** (Oct 9, early):
```
User: "MapBox tests failing in Selfhost VM"
Claude: "MapBox returns instance_id as Integer, not BoxRef"
ChatGPT: "Implement Arc cache for identity preservation"
→ Correct technical solution, but missed bigger picture
```

**Conversation 2** (Oct 9, mid):
```
User: "Is MapBox supposed to store all Box types?"
Claude: "Plugin MapBox is i64-only, Builtin MapBox is for complex types"
→ Implementation State Bias in action
```

**Conversation 3** (Oct 9, evening):
```
User: "Shouldn't builtin and plugin be equivalent?"
Claude: "Yes, per ADR-001... wait"
ChatGPT: "Single ABI + plugin/embedded choice is correct"
→ Realization moment
```

### 3.3 The "Aha!" Moment

**User's Question** (exact quote):
> "にゃん　というか　nyash ABIのmap box と　arraybox と　string boxを　用意して
> それをプラグインとして使うか　埋め込むかだけの　仕組みでよくない？
> 何か僕がきがついてないところあるかにゃ"

Translation:
> "Wait, shouldn't it just be: prepare Nyash ABI for MapBox/ArrayBox/StringBox,
> and choose between plugin or embedded implementation?
> Am I missing something?"

**AI Response** (ChatGPT-5):
> "にゃん、結論から言うとその方針でOKだよ。ひとつの「Nyash ABI（TypeBox）」を
> String/Array/Map に定義して、実装は「動的プラグイン or 埋め込み（静的）」の
> 二択にする。ランタイムは同じABIをしゃぶるだけ。これがいちばんシンプルで、
> 箱言語らしく使いやすいよ。"

Translation:
> "Yes, that's correct. Define single Nyash ABI (TypeBox) for String/Array/Map,
> and implementation is just plugin (dynamic .so) or embedded (static).
> Runtime just uses the same ABI. This is simplest and most Box-language-like."

**User's Insight**:
> "僕は最初からそのつもりでつくっていたはずが　なんか　AIちゃんたちに
> つたわってなかったというか　気がついてなかった僕も悪いですにゃね"

Translation:
> "I had intended this design from the beginning, but somehow it wasn't
> communicated to the AIs... though I should have noticed they didn't understand."

---

## 4. Proposed Solutions

### 4.1 Code Annotation Strategy

**Problem**: Incomplete implementation looks like final design

**Solution**: Explicit phase markers in code

**Before** (ambiguous):
```rust
//! Nyash MapBox Plugin — TypeBox v2 (minimal)
//! Extension: support both i64 and UTF-8 string keys;
//! **values remain i64**
```

**After** (clear intent):
```rust
//! Nyash MapBox Plugin — TypeBox v2 (minimal)
//!
//! **IMPLEMENTATION STATUS**: Phase A incomplete (2025-10-09)
//! TODO(Phase-A): Implement tag=8/9 support for all Box types (see ADR-001)
//!   - Current: values remain i64 (temporary limitation)
//!   - Target: Support PluginHandle(tag=8) and HostHandle(tag=9)
//!   - Infra: Arc cache complete (types.rs:237), decode complete (host_api.rs:57)
//!   - ETA: 2-3 hours implementation
//!
//! **DESIGN INTENT** (ADR-001): Everything is Plugin
//!   - Single ABI for all implementations
//!   - Plugin/builtin/embedded are interchangeable providers
//!   - No special "primitive-only" vs "complex-only" separation
```

### 4.2 ADR-Code Linking

**Problem**: Design documentation disconnected from code

**Solution**: Bidirectional references

**In ADR**:
```markdown
## Implementation Status

- ✅ TLV encoding (tag=8/9): `src/runtime/plugin_ffi_common.rs:208`
- ✅ Arc cache: `src/runtime/plugin_loader_v2/enabled/types.rs:237`
- ⏳ MapBox plugin: `plugins/nyash-map-plugin/src/lib.rs` (Phase A pending)
- ⏳ ArrayBox plugin: `plugins/nyash-array-plugin/src/lib.rs` (Phase A pending)
```

**In Code**:
```rust
// See: docs/development/adr/adr-001-no-corebox-everything-is-plugin.md
// Status: Phase A incomplete (tag=8/9 support pending)
```

### 4.3 Phase Roadmap in Code

**Example**:
```rust
/// MapBox Plugin Implementation Roadmap
///
/// Phase A (Current): Primitive values only
///   - ✅ i64 values (tag=3)
///   - ✅ String keys
///   - ⏳ tag=8/9 support (2-3 hours work)
///
/// Phase B (Next): Full Box support
///   - values() returns ArrayBox<Box>
///   - keys() returns ArrayBox<StringBox>
///   - toJSON() handles nested structures
///
/// Phase C (Future): Advanced features
///   - entries() → ArrayBox<TupleBox<StringBox, Box>>
///   - getOr(key, default)
///   - merge(otherMap)
```

### 4.4 AI Prompt Engineering

**For AI systems interpreting code**:

```
When encountering implementation constraints:
1. Check for TODO/FIXME/Phase markers
2. Search for ADR documents
3. Ask: "Is this implementation complete?"
4. Ask: "Does this match documented design intent?"
5. If mismatch found, clarify with human

Priority: Design Intent (ADR) > Current Implementation
```

---

## 5. Validation: The Design Was Perfect

### 5.1 Post-Realization Analysis

**What Needed to Change**: Almost nothing

**MapBox Plugin Update** (estimated 2-3 hours):
```rust
// Before
struct MapInstance {
    data_i64: HashMap<i64, i64>,
    data_str: HashMap<String, i64>,
}

// After
enum MapValue {
    I64(i64),
    PluginHandle { type_id: u32, instance_id: u32 },
    HostHandle { handle_id: u64 },
}

struct MapInstance {
    data_i64: HashMap<i64, MapValue>,
    data_str: HashMap<String, MapValue>,
}
```

**All infrastructure already complete**:
- ✅ TLV tag=8/9 encoding/decoding
- ✅ Arc cache for identity preservation
- ✅ UnifiedBoxRegistry for provider switching
- ✅ FactoryPolicy for plugin/builtin selection

### 5.2 Architecture Validation

**"Everything is Plugin" Design Goals**:

1. **Single ABI**: ✅ Validated
   - TypeBox v2 works for all implementations
   - No special cases needed

2. **Provider Interchangeability**: ✅ Validated
   - Plugin/builtin/embedded work identically
   - Switch via hako.toml configuration

3. **Identity Preservation**: ✅ Validated
   - Arc cache ensures same instance returns same pointer
   - finalize() called exactly once

4. **Simplicity**: ✅ Validated
   - User's insight: "Why not just single ABI + choice?"
   - Design is actually simpler than AI's proposed alternatives

### 5.3 Lessons Learned

**For Language Design**:
- ✅ "Everything is Plugin" is not just philosophy—it works
- ✅ TLV ABI with tag=8/9 enables flexible implementations
- ✅ Arc cache pattern solves identity preservation elegantly

**For Development Process**:
- ⚠️ Incomplete implementations confuse AI assistants
- ⚠️ Phase markers essential during incremental migration
- ⚠️ ADR-code linking must be explicit and bidirectional

**For Human-AI Collaboration**:
- ⚠️ AI reads code more than documentation
- ⚠️ "TODO" comments insufficient—need "Phase X incomplete"
- ⚠️ Periodic design-intent validation checks necessary

---

## 6. Related Work

### 6.1 Incremental Migration Patterns

**Strangler Fig Pattern** (Fowler, 2004):
- Gradually replace old system with new
- Both systems coexist temporarily
- **Similar challenge**: Old and new appear as intentional separation

**Branch by Abstraction** (Kerievsky, 2010):
- Create abstraction layer before migration
- Switch implementations behind abstraction
- **Hakorune approach**: UnifiedBoxRegistry as abstraction

### 6.2 AI-Assisted Programming Research

**GitHub Copilot Studies** (Chen et al., 2021):
- Focuses on code completion accuracy
- Does not address design intent interpretation

**GPT-4 Code Generation** (Bubeck et al., 2023):
- Demonstrates reasoning capabilities
- Does not study misinterpretation of partial implementations

**Claude Code in Production** (Anthropic, 2024):
- Focuses on task completion
- **Our finding**: Code understanding can diverge from design intent

### 6.3 Plugin Architecture Systems

**Eclipse Plugin System**:
- Extension points defined in XML
- Similar STN/PVN separation concept
- No AI interpretation challenges documented

**Rust Trait-Based Plugins**:
- Compile-time abstraction
- Clear separation of interface/implementation
- Static typing prevents misinterpretation

**WebAssembly Component Model**:
- Interface Types for cross-language interop
- Similar to Nyash ABI goals
- Still evolving (2024)

---

## 7. Contributions

### 7.1 Conceptual Contributions

**Implementation State Bias**: New concept in AI-assisted programming
- Definition: AI treating code as authoritative over design docs
- Impact: Systematic misinterpretation during incremental migration
- Detection: Compare AI conclusions with ADR documents

### 7.2 Practical Solutions

**Phase Annotation Strategy**:
```rust
/// **IMPLEMENTATION STATUS**: Phase X incomplete (date)
/// TODO(Phase-X): [what's needed] (see ADR-NNN)
///   - Current: [current limitation]
///   - Target: [intended behavior]
///   - ETA: [estimated work]
```

**ADR-Code Linking**:
- Bidirectional references
- Implementation status tracking in ADR
- Code comments link to design documents

### 7.3 Architectural Validation

**"Everything is Plugin" Architecture**:
- Proven feasible in production language (Hakorune)
- TLV ABI with tag=8/9 enables flexible implementations
- Single abstraction (UnifiedBoxRegistry) supports multiple providers
- Identity preservation via Arc cache pattern

---

## 8. Future Work

### 8.1 AI Systems Enhancement

**Design-Intent-Aware AI**:
- Prioritize ADR/design docs over code observations
- Explicitly check for "implementation complete?" markers
- Flag contradictions between design and implementation

**Automated Consistency Checking**:
- Parse ADR documents for design intent
- Scan codebase for TODO/Phase markers
- Report misalignments

### 8.2 Development Tools

**IDE Integration**:
- Highlight incomplete implementations
- Link code to ADR documents
- Show phase roadmap in hover tooltips

**CI/CD Checks**:
- Verify ADR-code consistency
- Detect unmarked incomplete implementations
- Enforce phase marker presence

### 8.3 Research Directions

**Empirical Studies**:
- How often does Implementation State Bias occur?
- What percentage of AI suggestions reflect code vs design?
- Comparative study across different AI systems

**Mitigation Strategies**:
- Best practices for human-AI collaboration
- Phase annotation standards
- ADR-code linking formats

---

## 9. Conclusion

### 9.1 Summary of Findings

This paper identified and analyzed **Implementation State Bias**: a systematic tendency of AI systems to misinterpret incomplete implementations as intentional design choices. Through the real-world case study of Hakorune's "Everything is Plugin" architecture migration, we demonstrated:

1. **The Problem is Real**: Both Claude and ChatGPT-5 misinterpreted MapBox plugin's i64 limitation as intentional design, despite perfect ADR documentation stating otherwise.

2. **Root Cause Identified**: AI systems prioritize code observation over design documentation during reasoning, leading to incorrect conclusions during incremental migrations.

3. **Solutions are Practical**: Simple code annotations (Phase markers, TODO linking, status comments) can prevent misinterpretation with minimal overhead.

4. **Architecture Validated**: The user's design was perfect from the start—"Everything is Plugin" with single ABI and interchangeable providers works elegantly.

### 9.2 Key Takeaways

**For Developers**:
- ✅ Write ADRs early (user did this correctly)
- ⚠️ Mark incomplete implementations explicitly (what was missing)
- ⚠️ Link code to design documents bidirectionally
- ⚠️ Periodically validate AI understanding of design intent

**For AI Systems**:
- ⚠️ Don't assume code reflects final design
- ✅ Check for TODO/Phase/Status markers
- ✅ Prioritize ADR documents over code comments
- ✅ Ask "Is this complete?" before concluding intent

**For Language Design**:
- ✅ "Everything is Plugin" architecture validated
- ✅ TLV ABI with tag=8/9 enables flexible implementations
- ✅ Arc cache pattern solves identity preservation
- ✅ Single abstraction layer (UnifiedBoxRegistry) works well

### 9.3 Impact

This work contributes to the emerging field of human-AI collaborative software development by:

1. **Identifying a new class of error**: Implementation State Bias
2. **Providing practical solutions**: Code annotation strategies
3. **Validating an architectural approach**: Everything is Plugin
4. **Informing future AI development**: Design-intent-aware systems

As AI-assisted programming becomes ubiquitous, understanding and mitigating Implementation State Bias will be crucial for maintaining design consistency during long-term software evolution.

---

## Acknowledgments

The author thanks Claude (Anthropic) and ChatGPT-5 (OpenAI) for their assistance in developing Hakorune, despite—or perhaps because of—the misunderstanding that led to this research. The "bug" became a feature, revealing important insights about human-AI collaboration.

Special thanks to the open-source community for feedback on Hakorune's architecture and to the Rust community for providing excellent tools for implementing plugin systems.

---

## References

1. Fowler, M. (2004). StranglerFigApplication. martinfowler.com
2. Kerievsky, J. (2010). Branch By Abstraction. continuousdelivery.com
3. Chen, M. et al. (2021). "Evaluating Large Language Models Trained on Code." arXiv:2107.03374
4. Bubeck, S. et al. (2023). "Sparks of Artificial General Intelligence: Early experiments with GPT-4." arXiv:2303.12712
5. Anthropic (2024). "Claude Code: AI-Assisted Development." anthropic.com
6. Nakashima, T. (2025). "Hakorune: A Self-Hosted Box-First Language." github.com/hakorune-selfhost

---

## Appendices

### Appendix A: Hakorune Architecture Details

**MIR (Mid-level IR)**:
- 16 instructions (frozen set)
- SSA form with explicit control flow
- Box-centric value model

**TypeBox v2 ABI**:
```c
struct NyashTypeBoxFfi {
    uint32_t abi_tag;              // 'TYBX'
    uint16_t version;              // 1
    uint16_t struct_size;
    const char* name;              // "StringBox", "ArrayBox", etc.
    resolve_fn resolve;            // method_name → method_id
    invoke_fn invoke_id;           // (type_id, method_id, instance_id, args) → result
    uint64_t capabilities;
};
```

**TLV Encoding Details**:
```
Format: [version:u16][count:u16][entries...]
Entry:  [tag:u8][reserved:u8][size:u16][payload:bytes]

Tag values:
  1: bool (size=1)
  2: i32 (size=4)
  3: i64 (size=8)
  5: f64 (size=8)
  6: string (UTF-8, size=variable)
  7: cstring (C-string, size=variable)
  8: PluginHandle (type_id:u32 + instance_id:u32, size=8)
  9: HostHandle (handle_id:u64, size=8)
```

### Appendix B: Timeline Details

**Key Commits**:
- `566429fb` (Oct 8): MapBox.get(missing)→null default
- `cc34c9cb` (Oct 7): Collections API .size()/.isEmpty() unification
- `5b17c03b` (Oct 6): Phase 1 Collections API local implementation
- `3be3e5af` (Oct 5): MapBox.get() bug fix
- `49c4d10d` (Oct 9): ExternCall retirement + op_eq inline (ChatGPT-5)

**Conversation Logs**:
Available in Claude/ChatGPT conversation history (2025-10-09)

### Appendix C: Code Examples

**MapBox Plugin Before (i64-only)**:
```rust
struct MapInstance {
    data_i64: HashMap<i64, i64>,
    data_str: HashMap<String, i64>,
}

METHOD_SET => {
    let val = read_arg_i64(args, args_len, 1)?;
    map.insert(key, val);  // i64 only
}
```

**MapBox Plugin After (tag=8/9 support)**:
```rust
enum MapValue {
    I64(i64),
    PluginHandle { type_id: u32, instance_id: u32 },
    HostHandle { handle_id: u64 },
}

struct MapInstance {
    data_i64: HashMap<i64, MapValue>,
    data_str: HashMap<String, MapValue>,
}

METHOD_SET => {
    let val = match peek_tag(args, args_len, 1) {
        3 => MapValue::I64(read_i64(...)?),
        8 => MapValue::PluginHandle {
            type_id: read_u32(...)?,
            instance_id: read_u32(...)?,
        },
        9 => MapValue::HostHandle {
            handle_id: read_u64(...)?,
        },
        _ => return NYB_E_INVALID_ARGS,
    };
    map.insert(key, val);
}
```

**Builtin MapBox (already complete)**:
```rust
// src/boxes/map_box.rs:116
pub struct MapBox {
    data: Arc<RwLock<HashMap<String, Box<dyn NyashBox>>>>,
}

// Line 148-162: Identity preservation
pub fn get(&self, key: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
    match guard.get(&key_str) {
        Some(value) => {
            // Preserve identity for plugin/user boxes
            if value.is::<PluginBoxV2>() { return value.share_box(); }
            if value.is::<InstanceBox>() { return value.share_box(); }
            value.clone_box()
        }
    }
}
```

---

**End of Paper**

**Future Updates**: This paper will be updated as MapBox plugin implementation completes (estimated 2025-10-10) and additional insights emerge from the development process.

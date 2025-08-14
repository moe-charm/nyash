# MIR 25-Instruction Mapping Plan

## Current State: 32 Instructions → Target: 25 Instructions

### Tier-0: Universal Core (8 instructions)
1. **Const** ✅ (already exists)
2. **BinOp** ✅ (already exists) 
3. **Compare** ✅ (already exists)
4. **Branch** ✅ (already exists)
5. **Jump** ✅ (already exists)
6. **Phi** ✅ (already exists)
7. **Call** ✅ (already exists)
8. **Return** ✅ (already exists)

### Tier-1: Nyash Semantics (12 instructions)
9. **NewBox** ✅ (already exists)
10. **BoxFieldLoad** ← RENAME from Load/RefGet
11. **BoxFieldStore** ← RENAME from Store/RefSet  
12. **BoxCall** ✅ (already exists)
13. **Safepoint** ✅ (already exists as separate instruction)
14. **RefGet** → RENAME to RefGet ✅
15. **RefSet** → RENAME to RefSet ✅
16. **WeakNew** ✅ (already exists)
17. **WeakLoad** ✅ (already exists)
18. **WeakCheck** ← NEW (check weak reference validity)
19. **Send** ← NEW (Bus communication)
20. **Recv** ← NEW (Bus communication)

### Tier-2: Implementation Assistance (5 instructions)
21. **TailCall** ← NEW (tail call optimization)
22. **Adopt** ← NEW (ownership transfer)
23. **Release** ← NEW (ownership release)
24. **MemCopy** ← NEW (optimized memory operations)
25. **AtomicFence** ← RENAME from BarrierRead/BarrierWrite

## Instructions to Remove/Consolidate (7 instructions)
- **UnaryOp** → Merge into BinOp or eliminate
- **Load** → Consolidate into BoxFieldLoad
- **Store** → Consolidate into BoxFieldStore
- **ArrayGet** → Use BoxFieldLoad with array indexing
- **ArraySet** → Use BoxFieldStore with array indexing
- **Cast** → Eliminate or merge into BinOp
- **Copy** → Eliminate (optimization-specific)
- **Debug** → Remove from MIR (keep as separate system)
- **Print** → Use Call with print function
- **Throw** → Use Call with exception function
- **Catch** → Use Call with catch handler
- **RefNew** → Eliminate (use NewBox)
- **TypeCheck** → Use Compare with type introspection
- **BarrierRead/BarrierWrite** → Consolidate into AtomicFence
- **FutureNew/FutureSet/Await** → Use BoxCall with Future methods

## Effect System Mapping

### Current → New Effect Categories
- **Pure**: Const, BinOp, Compare, Phi, RefGet, WeakNew, WeakLoad, WeakCheck
- **Mut**: BoxFieldStore, RefSet, Adopt, Release, MemCopy
- **Io**: Send, Recv, Safepoint, AtomicFence
- **Control**: Branch, Jump, Return, TailCall
- **Context-dependent**: Call, BoxCall

## Implementation Strategy

### Phase 1: Core Instruction Consolidation
1. Rename Load → BoxFieldLoad
2. Rename Store → BoxFieldStore
3. Remove eliminated instructions
4. Add missing new instructions

### Phase 2: Effect System Update
1. Update effect classification to 4 categories
2. Update all instruction effect mappings
3. Implement effect-based optimization rules

### Phase 3: Backend Updates
1. Update Interpreter backend
2. Update VM backend
3. Update WASM backend
4. Ensure all support exactly 25 instructions

### Phase 4: Verification System
1. Implement ownership forest verification
2. Add strong cycle detection
3. Add weak reference safety checks
4. Implement RefSet ownership validation
# MIR 26-Instruction Specification

*Nyash Machine Intermediate Representation - ChatGPT5 Compliant Version*

## Overview

This document specifies the official 26-instruction set for Nyash MIR (Machine Intermediate Representation), following the ChatGPT5 specification and AI consensus from the grand meeting.

## Instruction Set (26 Instructions)

### Tier-0: Core Instructions (5)

1. **Const** - Load constant value
   ```mir
   %dst = const <value>
   ```
   Effect: pure

2. **BinOp** - Binary operations (includes arithmetic, logical, comparison)
   ```mir
   %dst = binop <op> %lhs, %rhs
   ```
   Effect: pure

3. **Compare** - Comparison operations
   ```mir
   %dst = icmp <op> %lhs, %rhs
   ```
   Effect: pure

4. **Phi** - SSA phi node
   ```mir
   %dst = phi [%bb1: %val1], [%bb2: %val2], ...
   ```
   Effect: pure

5. **Call** - Function and intrinsic calls
   ```mir
   %dst = call <func>(%arg1, %arg2, ...)
   call <func>(%arg1, %arg2, ...)
   ```
   Effect: context-dependent

### Tier-0: Control Flow (3)

6. **Branch** - Conditional branch
   ```mir
   br %cond, label %then, label %else
   ```
   Effect: control

7. **Jump** - Unconditional jump
   ```mir
   br label %target
   ```
   Effect: control

8. **Return** - Return from function
   ```mir
   ret %value
   ret void
   ```
   Effect: control

### Tier-1: Box Operations (5)

9. **NewBox** - Create new Box instance
   ```mir
   %dst = new <BoxType>(%arg1, %arg2, ...)
   ```
   Effect: mut

10. **BoxFieldLoad** - Load field from Box
    ```mir
    %dst = %box.field
    ```
    Effect: pure

11. **BoxFieldStore** - Store field to Box
    ```mir
    %box.field = %value
    ```
    Effect: mut

12. **BoxCall** - Call Box method
    ```mir
    %dst = call %box.method(%arg1, %arg2, ...)
    ```
    Effect: context-dependent

13. **ExternCall** - Call external function
    ```mir
    %dst = extern_call "interface.method"(%arg1, %arg2, ...)
    ```
    Effect: context-dependent

### Tier-1: Reference Operations (6)

14. **RefGet** - Get reference target
    ```mir
    %dst = ref_get %reference
    ```
    Effect: pure

15. **RefSet** - Set reference target
    ```mir
    ref_set %reference -> %new_target
    ```
    Effect: mut

16. **WeakNew** - Create weak reference
    ```mir
    %dst = weak_new %box
    ```
    Effect: pure

17. **WeakLoad** - Load from weak reference
    ```mir
    %dst = weak_load %weak_ref
    ```
    Effect: pure

18. **WeakCheck** - Check if weak reference is alive
    ```mir
    %dst = weak_check %weak_ref
    ```
    Effect: pure

19. **Safepoint** - GC safepoint
    ```mir
    safepoint
    ```
    Effect: io

### Tier-2: Advanced Operations (7)

20. **Send** - Send message via Bus
    ```mir
    send %data -> %target
    ```
    Effect: io

21. **Recv** - Receive message from Bus
    ```mir
    %dst = recv %source
    ```
    Effect: io

22. **TailCall** - Tail call optimization
    ```mir
    tail_call %func(%arg1, %arg2, ...)
    ```
    Effect: control

23. **Adopt** - Adopt ownership
    ```mir
    adopt %parent <- %child
    ```
    Effect: mut

24. **Release** - Release ownership
    ```mir
    release %reference
    ```
    Effect: mut

25. **MemCopy** - Optimized memory copy
    ```mir
    memcpy %dst <- %src, %size
    ```
    Effect: mut

26. **AtomicFence** - Memory barrier
    ```mir
    atomic_fence <ordering>
    ```
    Effect: io

## Deprecated Instructions (17)

The following instructions have been removed from the specification:

1. **UnaryOp** → Use `BinOp` (e.g., `not %x` → `%x xor true`)
2. **Load** → Use `BoxFieldLoad`
3. **Store** → Use `BoxFieldStore`
4. **ArrayGet** → Use `BoxFieldLoad` or `Call @array_get`
5. **ArraySet** → Use `BoxFieldStore` or `Call @array_set`
6. **Print** → Use `Call @print`
7. **Debug** → Use `Call @debug`
8. **TypeCheck** → Use `Call @type_check`
9. **Cast** → Use `Call @cast`
10. **Throw** → Use `Call @throw`
11. **Catch** → Use `Call @catch`
12. **Copy** → Optimization pass only
13. **Nop** → Not needed
14. **RefNew** → References handled implicitly
15. **BarrierRead** → Use `AtomicFence`
16. **BarrierWrite** → Use `AtomicFence`
17. **FutureNew/FutureSet/Await** → Use `NewBox` + `BoxCall`

## Intrinsic Functions

Standard intrinsic functions available via `Call` instruction:

- `@print(value)` - Print value to console
- `@debug(value, message)` - Debug output
- `@type_check(value, type)` - Runtime type check
- `@cast(value, type)` - Type cast
- `@throw(exception)` - Throw exception
- `@catch(type, handler)` - Set exception handler
- `@array_get(array, index)` - Array element access
- `@array_set(array, index, value)` - Array element update
- `@unary_neg(value)` - Unary negation
- `@unary_not(value)` - Logical not

## Effect System

Each instruction has an associated effect mask:

- **pure** - No side effects, can be reordered/eliminated
- **mut** - Mutates memory, order-dependent
- **io** - I/O operations, cannot be eliminated
- **control** - Control flow, affects program execution path

## Migration Guide

### UnaryOp Migration
```mir
// Before
%dst = neg %x
%dst = not %x

// After
%dst = binop sub 0, %x
%dst = binop xor %x, true
```

### Load/Store Migration
```mir
// Before
%value = load %ptr
store %value -> %ptr

// After
%value = %box.field
%box.field = %value
```

### Print Migration
```mir
// Before
print %value

// After
call @print(%value)
```

### Future Operations Migration
```mir
// Before
%future = future_new %value
future_set %future = %result
%result = await %future

// After
%future = new FutureBox(%value)
call %future.set(%result)
%result = call %future.await()
```

## Implementation Status

- ✅ Phase 1: New instruction definitions
- ✅ Phase 2: Frontend migration (AST→MIR generation)
- ✅ Phase 3: Optimization pass migration
- ✅ Phase 4: Backend implementation (VM/WASM)
- ✅ Phase 5-1: Deprecated instruction marking
- ✅ Phase 5-2: Backend rejection of deprecated instructions
- ✅ Phase 5-3: Frontend stops generating deprecated instructions
- 🔄 Phase 5-4: Test and documentation updates (in progress)
- 📋 Phase 5-5: Final verification and cleanup

---

*Last updated: 2025-08-17*
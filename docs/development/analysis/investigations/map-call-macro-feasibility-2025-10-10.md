# Map.call() as Macro: Feasibility Investigation

**Date**: 2025-10-10
**Investigator**: Claude (AI Assistant)
**User's Question**: "Could Map.call() be implemented as a Hakorune macro instead of native Rust code?"

---

## Executive Summary

**Verdict**: ❌ **NOT CURRENTLY FEASIBLE** (but could be in future Phase 20+)

**Key Finding**: Hakorune's current macro system (Phase 19) can only generate **new** Box declarations and static methods, but **cannot inject methods into existing Boxes** like MapBox.

**Current Status**: Map.call() must remain a native Rust method (slot 210).

**Future Possibility**: A "method macro" system could enable this, but requires significant language design work.

---

## 1. Survey: Existing Macro System Capabilities

### 1.1 What Macros Exist?

Based on investigation of `/home/tomoaki/git/hakorune-selfhost/src/macro/engine.rs`:

| Macro | Type | Capability | Implemented |
|-------|------|------------|-------------|
| `@enum` | Declaration | Generates **new** Box + static box | ✅ Phase 19 |
| `@match` | Expression | Pattern matching desugaring | ✅ Phase 19 |
| `@derive` | Attribute | Auto-generates equals()/toString() | ✅ (implicit) |

### 1.2 What Can Macros Do?

From `src/macro/engine.rs:54-118`:

```rust
fn expand_node(&mut self, node: &ASTNode) -> ASTNode {
    match node.clone() {
        ASTNode::Program { statements, span } => {
            // Can transform entire program
            let new_stmts = statements.into_iter().flat_map(|n| {
                let expanded = self.expand_node(&n);
                // Can expand 1 node → multiple nodes
                match expanded {
                    ASTNode::Program { statements: inner, .. } => inner,
                    other => vec![other],
                }
            }).collect();
            ASTNode::Program { statements: new_stmts, span }
        }
        ASTNode::BoxDeclaration { name, fields, methods, ... } => {
            // Can ADD methods to Box being declared
            // (e.g., @derive adds equals/toString)
            let mut new_methods = methods.clone();
            new_methods.insert("equals".to_string(), build_equals_method(...));
            ASTNode::BoxDeclaration { methods: new_methods, ... }
        }
        ASTNode::EnumDeclaration { name, variants, span } => {
            // Can generate NEW boxes
            expand_enum_to_boxes(&name, &variants, span)
        }
        other => other,
    }
}
```

**Capabilities**:
- ✅ Transform AST nodes during compilation
- ✅ Generate **new** Box declarations
- ✅ Add methods to Boxes **during their declaration**
- ✅ Multi-node expansion (1 @enum → Box + StaticBox)

**Limitations**:
- ❌ Cannot modify **existing** Boxes (like built-in MapBox)
- ❌ No "extension methods" concept
- ❌ Macro expansion happens **before** type resolution

---

## 2. @enum Macro Analysis: Method Injection Capabilities

### 2.1 How @enum Adds Methods

From `src/macro/engine.rs:218-302`, the `expand_enum_to_boxes()` function:

```rust
fn expand_enum_to_boxes(name: &str, variants: &[EnumVariant], span: Span) -> Vec<ASTNode> {
    let box_name = format!("{}Box", name);

    // Creates NEW BoxDeclaration with methods
    let mut methods: HashMap<String, ASTNode> = HashMap::new();
    methods.insert("birth".to_string(), build_enum_birth_method(...));
    methods.insert("is_Ok".to_string(), build_enum_is_method(...));
    methods.insert("as_Ok".to_string(), build_enum_as_method(...));

    let box_decl = ASTNode::BoxDeclaration {
        name: box_name,
        methods,  // ← Methods added during creation
        ...
    };

    // Also creates static box with constructors
    let static_box_decl = ASTNode::BoxDeclaration {
        name: name.to_string(),
        methods: static_methods,
        is_static: true,
        ...
    };

    vec![box_decl, static_box_decl]  // Returns NEW boxes
}
```

**Key Insight**: @enum generates **brand new** ResultBox and Result static box. It doesn't modify existing types.

### 2.2 Example: @enum Result

**Input**:
```hakorune
@enum Result {
    Ok(value)
    Err(error)
}
```

**Output** (generated at compile time):
```hakorune
box ResultBox {
    _tag: StringBox
    value: any
    error: any

    birth() { /* null init */ }
    is_Ok() { return me._tag == "Ok" }
    as_Ok() { return me.value }
    // ... more methods
}

static box Result {
    Ok(value) { /* constructor */ }
    Err(error) { /* constructor */ }
}
```

**Conclusion**: @enum creates new types, doesn't extend existing ones.

---

## 3. Map.call() Current Implementation

### 3.1 Native Implementation

From `/home/tomoaki/git/hakorune-selfhost/src/runtime/method_router_box/mod.rs:410-427`:

```rust
"MapBox" => {
    if let Some(mp) = bx.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
        match slot as u32 {
            210 => { // call(key, argsArray)
                if args.len() != 2 { return Err(...); }

                let key_box = args[0].to_nyash_box();
                let callee = mp.get(key_box);  // Get CallableBox from map

                if let Some(cb) = callee.as_any().downcast_ref::<CallableBox>() {
                    let recv_vm = VMValue::BoxRef(...);
                    let argv: Vec<VMValue> = hako_core_callable::flatten_argv(...);

                    // Actual call: receiver.method(argv)
                    crate::runtime::method_router_box::route(
                        _interp,
                        &recv_vm,
                        &cb.method,
                        &argv
                    )
                } else {
                    Err(VMError::InvalidInstruction("Map.call: value is not CallableBox".into()))
                }
            }
        }
    }
}
```

### 3.2 What Map.call() Does

**Semantics**: `map.call(key, args)` desugars to `map.get(key).call(args)`

**Steps**:
1. Look up `key` in MapBox → get CallableBox
2. Extract receiver + method from CallableBox
3. Flatten args array
4. Call receiver.method(flattenedArgs)

**Why it's complex**:
- Type checking (is value a CallableBox?)
- Argument flattening (ArrayBox → flat VMValue list)
- Interpreter context passing (_interp)
- Error handling

---

## 4. Hypothetical Macro Implementation

### 4.1 Syntax Design

**Ideal syntax** (if macros could extend existing types):
```hakorune
// Hypothetical "extension macro" (doesn't exist)
@extend MapBox {
    call(key, args) {
        local callee = me.get(key)
        return callee.call(args)
    }
}
```

### 4.2 Why This Doesn't Work

**Problem 1: No @extend Macro**
- Current macros can only generate **new** declarations
- No mechanism to "reopen" existing Box definition
- MapBox is defined in `/home/tomoaki/git/hakorune-selfhost/src/boxes/map_box.rs` (Rust)

**Problem 2: Macro Expansion Timing**
```
Parse → Macro Expansion → Type Resolution → MIR Generation
           ↑
    Can't see existing MapBox here
```
- Macros run **before** type resolution
- Built-in MapBox isn't visible during macro expansion
- Can't query "what methods does MapBox have?"

**Problem 3: No Access to Interpreter Context**
```hakorune
@extend MapBox {
    call(key, args) {
        local callee = me.get(key)
        return callee.call(args)  // ← Needs _interp context!
    }
}
```
- Map.call() needs `MirInterpreter` reference for recursive calls
- User-level Hakorune code can't access interpreter internals

**Problem 4: Type Safety**
```hakorune
@extend MapBox {
    call(key, args) {
        local callee = me.get(key)
        // ← How to check if callee is CallableBox?
        // ← User code can't do downcast_ref
        return callee.call(args)
    }
}
```
- No runtime type introspection in user code
- Can't check `if callee is CallableBox`

---

## 5. Comparison: Macro vs Native

### 5.1 Feature Comparison

| Aspect | Native Method (Current) | Macro-based (Hypothetical) |
|--------|------------------------|---------------------------|
| **Implementation** | Rust code in method_router_box | Hakorune code generated by macro |
| **Type checking** | Runtime downcast_ref | ❌ Not available in user code |
| **Performance** | Native speed | Potentially slower (extra call) |
| **Maintainability** | Centralized in method_router | Distributed (every Box?) |
| **Plugin support** | Requires native slot 210/211 | ✅ Would work automatically |
| **Extensibility** | ❌ Hard to override | ✅ Easy to customize |

### 5.2 Plugin Problem

**Current Issue**: Plugin MapBox implementations must provide slot 210/211

From docs:
> **Plugin Implications**:
> - If Map.call() were a macro, would plugins need to implement it?
> - Current problem: Plugin MapBox needs native slot 210/211
> - **With macro: Plugin just needs get() → macro handles the rest?**

**Analysis**: This is the **strongest argument** for macro-based implementation!

**However**: Plugins would still need to support:
- `get(key)` returning CallableBox
- CallableBox.call() implementation

So the benefit is marginal (plugin still needs CallableBox support).

---

## 6. What Would Be Needed for Macro Implementation?

### 6.1 Required Language Features

#### Feature 1: Extension Macros
```hakorune
@extend MapBox {
    call(key, args) { /* body */ }
}
```
- Allow macros to **add methods to existing types**
- Requires type registry query during macro expansion
- Needs method conflict resolution

#### Feature 2: Trait/Interface System
```hakorune
interface Callable {
    call(args) -> any
}

box MapBox {
    call(key, args) {
        local callee = me.get(key)
        // Type-safe check: does callee implement Callable?
        if callee implements Callable {
            return callee.call(args)
        }
        panic("Value is not callable")
    }
}
```
- Type-safe interface checking
- No need for Rust-level downcast

#### Feature 3: Method Macro
```hakorune
@method_sugar
box MapBox {
    call(key, args) = me.get(key).call(args)
}
```
- Single-line method desugaring
- Expands to full implementation

### 6.2 Architecture Changes

**Current**:
```
User Code → Parser → Macro Expansion → MIR Builder → Method Router (Rust)
                                                           ↓
                                                      Native Methods
```

**With Extension Macros**:
```
User Code → Parser → Macro Expansion → Type Registry Query
                          ↓                    ↓
                   Generate Methods      Check Conflicts
                          ↓
                    MIR Builder (user methods)
```

**Challenges**:
1. Macro expansion needs type information (circular dependency)
2. Method conflict resolution (what if native and macro both define call()?)
3. Performance implications (extra indirection)

---

## 7. Alternative Approaches

### 7.1 Option A: Keep Native (Recommended)

**Pros**:
- ✅ Works today
- ✅ Best performance
- ✅ Type-safe (Rust guarantees)
- ✅ Centralized implementation

**Cons**:
- ❌ Plugin MapBox must implement slots 210/211
- ❌ Hard to extend/customize

**Verdict**: Current approach is solid.

### 7.2 Option B: Trait-Based Delegation

```hakorune
// Future design (Phase 25+ with trait system)
interface Callable {
    call(args) -> any
}

box MapBox {
    call(key, args) {
        local callee = me.get(key)
        return callee.call(args)  // ← Trait dispatch
    }
}
```

**Pros**:
- ✅ Type-safe
- ✅ No macro complexity
- ✅ Plugin-friendly (just implement Callable)

**Cons**:
- ❌ Requires trait system (Phase 25+)
- ❌ Still needs native implementation for get()

### 7.3 Option C: Method Alias Macro

```hakorune
@method_alias MapBox.invoke = get(key).call(args)
```

**Simpler than full extension macros**:
- Just syntax sugar for chained calls
- No type checking needed
- Could be implemented in Phase 20

**Pros**:
- ✅ Simpler than @extend
- ✅ No type system changes needed
- ✅ Clear semantics

**Cons**:
- ❌ Still can't add to existing MapBox
- ❌ Only works for new definitions

### 7.4 Option D: Default Method in Interface

```hakorune
// Phase 25+
interface Collection {
    get(key) -> any

    // Default implementation
    call(key, args) {
        return me.get(key).call(args)
    }
}

box MapBox implements Collection {
    // Inherits call() automatically
}
```

**Pros**:
- ✅ Idiomatic OOP solution
- ✅ Automatic inheritance
- ✅ Plugin-friendly

**Cons**:
- ❌ Requires interface system
- ❌ Still needs CallableBox support

---

## 8. Design Precedents

### 8.1 Why Was Map.call() Implemented Natively?

From `/home/tomoaki/git/hakorune-selfhost/docs/development/current/main/callable_box_implementation_plan.md`:

**Design Rationale**:
1. **Method registry pattern**: Centralized dispatch in method_router_box
2. **Performance**: Native code faster than interpreter
3. **Type safety**: Rust guarantees CallableBox downcast
4. **Complexity**: Argument flattening needs interpreter context

**Similar "sugar methods"**:
- None! Map.call() is unique in being pure syntactic sugar.
- Most methods have non-trivial semantics (e.g., Array.slice, String.substring)

### 8.2 Could Other Methods Be Macros?

| Method | Could Be Macro? | Why/Why Not |
|--------|----------------|-------------|
| `Array.slice(start, end)` | ❌ No | Needs native memory manipulation |
| `String.substring(s, e)` | ❌ No | UTF-8 byte arithmetic |
| `Map.get(key)` | ❌ No | Hash table lookup |
| `Map.call(key, args)` | ✅ Yes | Just sugar for get().call() |
| `Array.methodRef(name, arity)` | ❌ No | Creates CallableBox (needs introspection) |

**Conclusion**: Map.call() is the **only** method that's truly pure sugar.

---

## 9. Pros/Cons: Macro vs Native

### 9.1 Macro-Based Implementation

**Pros**:
- ✅ **Extensibility**: Users could customize behavior
- ✅ **Plugin-friendly**: Automatic for any Box with get()
- ✅ **Clarity**: Implementation visible in language
- ✅ **Consistency**: Less special-case native code

**Cons**:
- ❌ **Not currently possible**: Needs @extend or trait system
- ❌ **Type safety**: Harder to guarantee CallableBox type
- ❌ **Performance**: Extra call indirection
- ❌ **Complexity**: New macro system features needed

### 9.2 Native Implementation (Current)

**Pros**:
- ✅ **Works today**: No language changes needed
- ✅ **Type-safe**: Rust-level guarantees
- ✅ **Performance**: Zero-cost abstraction
- ✅ **Centralized**: Easy to find implementation

**Cons**:
- ❌ **Plugin burden**: Must implement slots 210/211
- ❌ **Not extensible**: Hard to customize
- ❌ **Hidden implementation**: Not visible to users

### 9.3 Verdict

**Current**: Keep native (no choice, macros can't do it)
**Future**: Consider trait-based approach (Phase 25+)
**Never**: Pure macro-based (type safety issues)

---

## 10. Recommendation

### 10.1 Short-Term (Phase 19-20): Keep Native

**Rationale**:
- Current macro system cannot add methods to existing Boxes
- No extension macro feature
- Native implementation works well

**Action**: **No change needed**

### 10.2 Mid-Term (Phase 25): Trait System

**When trait/interface system exists**:
```hakorune
interface Collection {
    get(key) -> any

    default call(key, args) {
        return me.get(key).call(args)
    }
}

box MapBox implements Collection {
    // Inherits call() from interface
}

// Plugin boxes also get it automatically
box PluginMapBox implements Collection {
    // Inherits call() for free
}
```

**Benefits**:
- ✅ Type-safe default implementation
- ✅ Plugin-friendly (automatic inheritance)
- ✅ Idiomatic OOP design
- ✅ No macro complexity

### 10.3 Long-Term (Phase 30+): Extension Methods?

**If extension methods are added**:
```hakorune
// Define extension in separate file
extend MapBox {
    call(key, args) {
        return me.get(key).call(args)
    }
}
```

**Benefits**:
- ✅ True extensibility
- ✅ User-defined extensions
- ✅ Matches C#, Kotlin, Swift patterns

**Challenges**:
- Macro or compiler feature?
- Method resolution complexity
- Type safety implications

---

## 11. Plugin Implications Deep Dive

### 11.1 Current Plugin Burden

**Plugin MapBox Requirements**:
```rust
// src/runtime/plugin_loader_v2.rs
impl PluginMapBox {
    fn register_methods() {
        // Must provide:
        registry.add_method("get", 1, slot_203);
        registry.add_method("set", 2, slot_204);
        registry.add_method("call", 2, slot_210);      // ← Extra work
        registry.add_method("callAsync", 2, slot_211); // ← Extra work
    }
}
```

**What plugins must implement**:
1. Core methods: get, set, has, delete, clear, keys, values (7 methods)
2. Sugar methods: call, callAsync (2 methods)

**Total**: 9 methods instead of 7

### 11.2 If Map.call() Were Macro/Trait

**Plugin would only need**:
```rust
impl PluginMapBox {
    fn register_methods() {
        // Only core methods:
        registry.add_method("get", 1, slot_203);
        registry.add_method("set", 2, slot_204);
        // ... 5 more core methods
        // NO call/callAsync needed!
    }
}
```

**Macro/Trait handles**:
- call() → automatic (uses get())
- callAsync() → automatic (uses get() + CallableBox.callAsync())

**Reduction**: 9 methods → 7 methods (22% reduction)

### 11.3 Is This Worth It?

**Argument FOR**:
- 22% reduction in plugin boilerplate
- Plugins can't forget to implement call()
- More consistent behavior across plugins

**Argument AGAINST**:
- Only saves 2 methods
- Plugins still need CallableBox support anyway
- call() is rarely used in practice

**Verdict**: **Minor benefit**, not worth major language changes.

---

## 12. Summary: Can Map.call() Be a Macro?

### 12.1 Answer Matrix

| Question | Answer | Reason |
|----------|--------|--------|
| **Can current macros do it?** | ❌ No | Cannot modify existing Box declarations |
| **Could future macros do it?** | ⚠️ Maybe | Needs @extend or method macro feature |
| **Should it be a macro?** | ❌ No | Trait system is better solution |
| **Keep native implementation?** | ✅ Yes | Works well, no alternatives ready |

### 12.2 Technical Feasibility

**Phase 19 (Current)**: ❌ Not possible
- Macros can only generate new declarations
- Cannot extend existing MapBox

**Phase 20 (Extension Macros)**: ⚠️ Possible but problematic
- Would need new @extend macro type
- Type safety concerns
- Complexity not justified

**Phase 25 (Trait System)**: ✅ Ideal solution
- Default interface methods
- Type-safe
- Plugin-friendly
- Idiomatic

### 12.3 Design Recommendation

**User's brilliant insight**: "Map.call() is just syntactic sugar"

**Our conclusion**:
- ✅ Correct observation
- ✅ In principle, could be abstracted
- ❌ Current macros can't do it
- ✅ Future trait system should handle it
- ✅ Keep native for now

**Path forward**:
1. **Now**: Keep native implementation
2. **Phase 25**: Migrate to trait default method
3. **Phase 30+**: Consider extension methods

---

## 13. Deliverables

### 13.1 Investigation Complete

- ✅ Current macro system capabilities surveyed
- ✅ @enum macro method injection analyzed
- ✅ Map.call() native implementation understood
- ✅ Hypothetical macro implementation designed
- ✅ Pros/cons comparison completed
- ✅ Plugin implications evaluated
- ✅ Recommendation provided

### 13.2 Key Findings

1. **Current macros CANNOT add methods to existing Boxes**
2. **@enum generates NEW boxes, doesn't extend existing ones**
3. **Map.call() is unique**: only pure-sugar method in codebase
4. **Plugin benefit is minor**: 22% reduction (2 out of 9 methods)
5. **Best future solution**: Trait system with default methods (Phase 25)

### 13.3 Recommendation

**Short-term**: Keep native implementation (no alternatives)
**Mid-term**: Migrate to trait default method (Phase 25)
**Long-term**: Consider extension methods (Phase 30+)

---

## 14. Related Documentation

- **Macro System**: `/home/tomoaki/git/hakorune-selfhost/src/macro/engine.rs`
- **@enum Implementation**: Phase 19 README
- **Map.call() Native**: `/home/tomoaki/git/hakorune-selfhost/src/runtime/method_router_box/mod.rs:410-427`
- **CallableBox Plan**: `/home/tomoaki/git/hakorune-selfhost/docs/development/current/main/callable_box_implementation_plan.md`
- **MapBox Design**: `/home/tomoaki/git/hakorune-selfhost/docs/mapbox-design-analysis.md`

---

## 15. Open Questions for Future Design

1. **Should Hakorune have extension methods?** (C#/Kotlin/Swift pattern)
2. **Should traits have default implementations?** (Rust/Java pattern)
3. **Can macros query type information?** (Rust proc macros can)
4. **Should Map.call() exist at all?** (Alternative: just use get().call())

---

**Status**: ✅ Investigation Complete
**Verdict**: Keep native for now, consider trait system in Phase 25
**User's Insight**: Correct but not currently actionable

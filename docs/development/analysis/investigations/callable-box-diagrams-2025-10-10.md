# CallableBox Architecture Diagrams

## 1. CallableBox Structure

```
┌────────────────────────────────────────────────┐
│           CallableBox (NyashBox)               │
├────────────────────────────────────────────────┤
│  base:     BoxBase { id, parent_type_id }      │
│  receiver: Option<Box<dyn NyashBox>>           │ 👈 The Object
│  method:   String                              │ 👈 Method Name
│  arity:    usize                               │ 👈 Argument Count
├────────────────────────────────────────────────┤
│  Methods:                                      │
│    - new(receiver, method, arity) → Self       │
│    - arity() → usize                           │
│    - call(args) → Result (via router)          │
│    - clone_box() → Box<dyn NyashBox>           │
└────────────────────────────────────────────────┘
```

## 2. Creation Flow (arr.methodRef("size", 0))

```
┌─────────────────────────────────────────────────────────────┐
│  Hakorune Source Code                                       │
│  local cb = arr.methodRef("size", 0)                        │
└─────────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  MIR Compilation                                            │
│  BoxCall {                                                  │
│    receiver: ValueId(arr),                                  │
│    method: "methodRef",                                     │
│    args: [ValueId("size"), ValueId(0)]                      │
│  }                                                          │
└─────────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  VM Execution (method_router_box::route)                   │
│  1. Look up "ArrayBox.methodRef/2" in TypeRegistry          │
│  2. Resolve to slot 113                                     │
│  3. Dispatch to handler                                     │
└─────────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  ArrayBox Slot 113 Handler                                  │
│  let name = args[0].to_string();  // "size"                 │
│  let ar = args[1].as_integer();   // 0                      │
│  let cb = CallableBox::new(                                 │
│      Some(arr.clone_box()),       // receiver               │
│      name,                         // method = "size"       │
│      ar as usize                   // arity = 0             │
│  );                                                         │
│  return VMValue::from_nyash_box(Box::new(cb));              │
└─────────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  Result: CallableBox {                                      │
│    receiver: Some(ArrayBox[10, 20]),                        │
│    method: "size",                                          │
│    arity: 0                                                 │
│  }                                                          │
└─────────────────────────────────────────────────────────────┘
```

## 3. Invocation Flow (cb.call([]))

```
┌─────────────────────────────────────────────────────────────┐
│  Hakorune Source Code                                       │
│  local result = cb.call([])                                 │
└─────────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  MIR Compilation                                            │
│  BoxCall {                                                  │
│    receiver: ValueId(cb),                                   │
│    method: "call",                                          │
│    args: [ValueId(emptyArray)]                              │
│  }                                                          │
└─────────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  VM Execution (method_router_box::route)                   │
│  1. Look up "CallableBox.call/1" in TypeRegistry            │
│  2. Resolve to slot 501                                     │
│  3. Dispatch to handler                                     │
└─────────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  CallableBox Slot 501 Handler                               │
│  let argv: Vec<VMValue> = flatten_argv(args);               │
│  if let Some(recv) = &cb.receiver {                         │
│      let recv_vm = VMValue::BoxRef(recv.clone_box());       │
│      // 👇 RECURSIVE ROUTE CALL                             │
│      return route(                                          │
│          _interp,                                           │
│          &recv_vm,       // ArrayBox[10, 20]                │
│          &cb.method,     // "size"                          │
│          &argv           // []                              │
│      );                                                     │
│  }                                                          │
└─────────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  VM Execution (method_router_box::route - AGAIN)           │
│  1. Look up "ArrayBox.size/0" in TypeRegistry               │
│  2. Resolve to slot 102                                     │
│  3. Dispatch to handler                                     │
└─────────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  ArrayBox Slot 102 Handler                                  │
│  return VMValue::Integer(arr.len() as i64);  // 2           │
└─────────────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  Result: 2                                                  │
└─────────────────────────────────────────────────────────────┘
```

## 4. Storage in Collections

```
┌────────────────────────────────────────────┐
│  ArrayBox Storage                          │
├────────────────────────────────────────────┤
│  [0] → IntegerBox(10)                      │
│  [1] → IntegerBox(20)                      │
│  [2] → CallableBox {                       │ 👈 CallableBox stored here
│          receiver: Some(arr),              │
│          method: "size",                   │
│          arity: 0                          │
│        }                                   │
│  [3] → StringBox("hello")                  │
└────────────────────────────────────────────┘
         ↓ storage.get(2)
┌────────────────────────────────────────────┐
│  Retrieved: CallableBox                    │
│    - Can call: retrieved.call([])          │
│    - Can inspect: retrieved.arity()        │
│    - Can store again: map.set("cb", cb)    │
└────────────────────────────────────────────┘
```

```
┌────────────────────────────────────────────┐
│  MapBox Storage                            │
├────────────────────────────────────────────┤
│  "size"     → CallableBox(arr, "size", 0)  │ 👈 Store by name
│  "get"      → CallableBox(arr, "get", 1)   │
│  "push"     → CallableBox(arr, "push", 1)  │
│  "validate" → CallableBox(obj, "check", 2) │
└────────────────────────────────────────────┘
         ↓ map.call("size", [])
┌────────────────────────────────────────────┐
│  Map.call Shorthand                        │
│  1. map.get("size") → CallableBox          │
│  2. cb.call([]) → result                   │
└────────────────────────────────────────────┘
```

## 5. Method + Arity Relationship

```
┌──────────────────────────────────────────────────────────┐
│  ArrayBox Methods (Example)                              │
├─────────────┬────────┬─────────────────────────────────┤
│  Method     │ Arity  │  methodRef Signature            │
├─────────────┼────────┼─────────────────────────────────┤
│  size       │   0    │  arr.methodRef("size", 0)       │
│  get        │   1    │  arr.methodRef("get", 1)        │
│  set        │   2    │  arr.methodRef("set", 2)        │
│  push       │   1    │  arr.methodRef("push", 1)       │
│  pop        │   0    │  arr.methodRef("pop", 0)        │
│  slice      │   2    │  arr.methodRef("slice", 2)      │
└─────────────┴────────┴─────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│  Call Validation (Future Enhancement)                    │
├──────────────────────────────────────────────────────────┤
│  cb = arr.methodRef("get", 1)  // arity = 1              │
│                                                           │
│  cb.call([0])     ✅ Correct: 1 arg                      │
│  cb.call([0, 1])  ❌ Error: expected 1, got 2            │
│  cb.call([])      ❌ Error: expected 1, got 0            │
└──────────────────────────────────────────────────────────┘
```

## 6. Comparison with Other Languages

```
┌────────────────────────────────────────────────────────────┐
│  JavaScript: .bind()                                       │
├────────────────────────────────────────────────────────────┤
│  const arr = [10, 20];                                     │
│  const bound = arr.length.bind(arr);                       │
│  bound();  // 2                                            │
│                                                            │
│  ❌ No method field                                        │
│  ❌ No arity field                                         │
│  ✅ Captures receiver                                      │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│  Python: functools.partial                                 │
├────────────────────────────────────────────────────────────┤
│  from functools import partial                             │
│  arr = [10, 20]                                            │
│  size_fn = partial(len, arr)                               │
│  size_fn()  # 2                                            │
│                                                            │
│  ❌ No method field                                        │
│  ❌ No arity field                                         │
│  ✅ Captures receiver + function                           │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│  Java: Method References                                   │
├────────────────────────────────────────────────────────────┤
│  List<Integer> arr = Arrays.asList(10, 20);                │
│  Supplier<Integer> size = arr::size;                       │
│  size.get();  // 2                                         │
│                                                            │
│  ✅ Type-checked at compile time                           │
│  ❌ No runtime method field                                │
│  ❌ No runtime arity field                                 │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│  Hakorune: CallableBox                                     │
├────────────────────────────────────────────────────────────┤
│  local arr = new ArrayBox()                                │
│  arr.push(10)                                              │
│  arr.push(20)                                              │
│  local cb = arr.methodRef("size", 0)                       │
│  cb.call([])  // 2                                         │
│                                                            │
│  ✅ Captures receiver                                      │
│  ✅ Stores method name (String)                            │
│  ✅ Stores arity (Integer)                                 │
│  ✅ Runtime inspection: cb.arity()                         │
│  ✅ Dynamic dispatch                                       │
└────────────────────────────────────────────────────────────┘
```

## 7. Registry Pattern (Real-World Use Case)

```
┌────────────────────────────────────────────────────────────┐
│  Setup: Command Registry                                   │
├────────────────────────────────────────────────────────────┤
│  local commands = new MapBox()                             │
│  commands.set("getSize/0", arr.methodRef("size", 0))       │
│  commands.set("getItem/1", arr.methodRef("get", 1))        │
│  commands.set("addItem/1", arr.methodRef("push", 1))       │
└────────────────────────────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│  Execution: Dynamic Dispatch                               │
├────────────────────────────────────────────────────────────┤
│  local cmd = "getSize/0"                                   │
│  local cb = commands.get(cmd)                              │
│  local result = cb.call([])  // Dispatches to arr.size()   │
└────────────────────────────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│  Arity Checking (Future)                                   │
├────────────────────────────────────────────────────────────┤
│  local expected_arity = cb.arity()  // 0                   │
│  if args.size() != expected_arity {                        │
│      print("Error: expected " + expected_arity + " args")  │
│      return 1                                              │
│  }                                                         │
│  local result = cb.call(args)                              │
└────────────────────────────────────────────────────────────┘
```

## 8. Why methodRef Exists (Design Rationale)

```
┌────────────────────────────────────────────────────────────┐
│  WITHOUT methodRef (Hypothetical Manual Construction)      │
├────────────────────────────────────────────────────────────┤
│  // Would require internal APIs:                           │
│  local recv_box = arr.clone()  // Need clone API           │
│  local method_str = "size"     // String construction      │
│  local arity_int = 0           // Manual arity counting    │
│  local cb = CallableBox.new(   // Direct constructor       │
│      recv_box,                                             │
│      method_str,                                           │
│      arity_int                                             │
│  )                                                         │
│                                                            │
│  ❌ Complex                                                │
│  ❌ Error-prone (wrong arity)                              │
│  ❌ No validation                                          │
│  ❌ Not idiomatic                                          │
└────────────────────────────────────────────────────────────┘
                       VS
┌────────────────────────────────────────────────────────────┐
│  WITH methodRef (Actual API)                               │
├────────────────────────────────────────────────────────────┤
│  local cb = arr.methodRef("size", 0)                       │
│                                                            │
│  ✅ Simple                                                 │
│  ✅ Safe (single entry point)                              │
│  ✅ Validated (TypeRegistry checks)                        │
│  ✅ Idiomatic                                              │
└────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│  Design Pattern: Factory Method                            │
├────────────────────────────────────────────────────────────┤
│  methodRef = Factory Method for CallableBox                │
│  - Encapsulates complexity                                 │
│  - Ensures correctness                                     │
│  - Single entry point                                      │
│  - Future-proof (can add validation/optimization)          │
└────────────────────────────────────────────────────────────┘
```

## 9. Known Issue: Receiver Clone

```
┌────────────────────────────────────────────────────────────┐
│  Problem: CallableBox.call() clones receiver               │
├────────────────────────────────────────────────────────────┤
│  local arr = new ArrayBox()                                │
│  arr.push(10)           // arr = [10]                      │
│                                                            │
│  local cb = arr.methodRef("push", 1)                       │
│  cb.call([20])          // Pushes to CLONE!                │
│                                                            │
│  print(arr.size())      // Still 1 (not 2)                 │
└────────────────────────────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│  Root Cause (method_router_box.rs:159)                     │
├────────────────────────────────────────────────────────────┤
│  if let Some(recv) = &cb.receiver {                        │
│      let recv_vm = VMValue::BoxRef(                        │
│          Arc::from(recv.clone_box())  // 👈 CLONE HERE     │
│      );                                                    │
│      route(_interp, &recv_vm, &cb.method, &argv)           │
│  }                                                         │
└────────────────────────────────────────────────────────────┘
                       ↓
┌────────────────────────────────────────────────────────────┐
│  Workaround: Use non-mutating methods                      │
├────────────────────────────────────────────────────────────┤
│  ✅ OK: arr.methodRef("size", 0)   // Read-only            │
│  ✅ OK: arr.methodRef("get", 1)    // Read-only            │
│  ❌ AVOID: arr.methodRef("push", 1)  // Mutating           │
│  ❌ AVOID: arr.methodRef("set", 2)   // Mutating           │
└────────────────────────────────────────────────────────────┘
```

---

**Date**: 2025-10-10
**Purpose**: Visual reference for CallableBox architecture
**Related**: callable-box-fundamental-investigation-2025-10-10.md

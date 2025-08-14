🌐 Nyash WASM Compiler - Processing file: test_local_vars.nyash 🌐
✅ WASM compilation completed successfully!
📄 Generated WAT:
(module
  (import "env" "print" (func $print (param i32) ))
  (memory (export "memory") 1)
  (global $heap_ptr (mut i32) (i32.const 2048))
  (func $malloc (param $size i32) (result i32)
    (local $ptr i32)
    (local $aligned_size i32)
    
    ;; Align size to 4-byte boundary
    local.get $size
    i32.const 3
    i32.add
    i32.const -4
    i32.and
    local.set $aligned_size
    
    ;; Get current heap pointer
    global.get $heap_ptr
    local.set $ptr
    
    ;; Advance heap pointer by aligned size
    global.get $heap_ptr
    local.get $aligned_size
    i32.add
    global.set $heap_ptr
    
    ;; Return allocated pointer
    local.get $ptr
  )
  (func $box_alloc (param $type_id i32) (param $field_count i32) (result i32)
    (local $ptr i32)
    (local $total_size i32)
    
    ;; Calculate total size: header (12) + fields (field_count * 4)
    local.get $field_count
    i32.const 4
    i32.mul
    i32.const 12
    i32.add
    local.set $total_size
    
    ;; Allocate memory
    local.get $total_size
    call $malloc
    local.set $ptr
    
    ;; Initialize type_id
    local.get $ptr
    local.get $type_id
    i32.store
    
    ;; Initialize ref_count to 1
    local.get $ptr
    i32.const 4
    i32.add
    i32.const 1
    i32.store
    
    ;; Initialize field_count
    local.get $ptr
    i32.const 8
    i32.add
    local.get $field_count
    i32.store
    
    ;; Return box pointer
    local.get $ptr
  )
  (func $alloc_stringbox (result i32)
    (local $ptr i32)
    
    ;; Allocate memory for box
    i32.const 20
    call $malloc
    local.set $ptr
    
    ;; Initialize type_id
    local.get $ptr
    i32.const 4097
    i32.store
    
    ;; Initialize ref_count to 1
    local.get $ptr
    i32.const 4
    i32.add
    i32.const 1
    i32.store
    
    ;; Initialize field_count
    local.get $ptr
    i32.const 8
    i32.add
    i32.const 2
    i32.store
    
    ;; Return box pointer
    local.get $ptr
  )
  (func $alloc_integerbox (result i32)
    (local $ptr i32)
    
    ;; Allocate memory for box
    i32.const 16
    call $malloc
    local.set $ptr
    
    ;; Initialize type_id
    local.get $ptr
    i32.const 4098
    i32.store
    
    ;; Initialize ref_count to 1
    local.get $ptr
    i32.const 4
    i32.add
    i32.const 1
    i32.store
    
    ;; Initialize field_count
    local.get $ptr
    i32.const 8
    i32.add
    i32.const 1
    i32.store
    
    ;; Return box pointer
    local.get $ptr
  )
  (func $alloc_boolbox (result i32)
    (local $ptr i32)
    
    ;; Allocate memory for box
    i32.const 16
    call $malloc
    local.set $ptr
    
    ;; Initialize type_id
    local.get $ptr
    i32.const 4099
    i32.store
    
    ;; Initialize ref_count to 1
    local.get $ptr
    i32.const 4
    i32.add
    i32.const 1
    i32.store
    
    ;; Initialize field_count
    local.get $ptr
    i32.const 8
    i32.add
    i32.const 1
    i32.store
    
    ;; Return box pointer
    local.get $ptr
  )
  (func $alloc_databox (result i32)
    (local $ptr i32)
    
    ;; Allocate memory for box
    i32.const 16
    call $malloc
    local.set $ptr
    
    ;; Initialize type_id
    local.get $ptr
    i32.const 4101
    i32.store
    
    ;; Initialize ref_count to 1
    local.get $ptr
    i32.const 4
    i32.add
    i32.const 1
    i32.store
    
    ;; Initialize field_count
    local.get $ptr
    i32.const 8
    i32.add
    i32.const 1
    i32.store
    
    ;; Return box pointer
    local.get $ptr
  )
  (func $main (local $0 i32) (local $1 i32) (local $2 i32)
    nop
    i32.const 10
    local.set $0
    i32.const 20
    local.set $1
    local.get $0
    local.get $1
    i32.add
    local.set $2
    local.get $2
    return
  )
  (export "main" (func $main))
)


(module
  (import "env" "print" (func $print (param i32) ))
  (import "env" "print_str" (func $print_str (param i32 i32) ))
  (import "env" "console_log" (func $console_log (param i32 i32) ))
  (import "env" "canvas_fillRect" (func $canvas_fillRect (param i32 i32 i32 i32 i32 i32 i32 i32) ))
  (import "env" "canvas_fillText" (func $canvas_fillText (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32) ))
  (import "env" "box_to_string" (func $box_to_string (param i32) (result i32)))
  (import "env" "box_print" (func $box_print (param i32) ))
  (import "env" "box_equals" (func $box_equals (param i32 i32) (result i32)))
  (import "env" "box_clone" (func $box_clone (param i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 4096) "\48\65\6c\6c\6f\20\4d\49\52\5c\21")
  (data (i32.const 4107) "\40\70\72\69\6e\74")
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
    call $alloc_stringbox
    local.set $0
    local.get $0
    i32.const 12
    i32.add
    i32.const 4096
    i32.store
    local.get $0
    i32.const 16
    i32.add
    i32.const 11
    i32.store
    local.get $0
    local.set $1
    call $alloc_stringbox
    local.set $2
    local.get $2
    i32.const 12
    i32.add
    i32.const 4107
    i32.store
    local.get $2
    i32.const 16
    i32.add
    i32.const 6
    i32.store
    local.get $1
    call $print
    local.get $1
    return
  )
  (export "main" (func $main))
)

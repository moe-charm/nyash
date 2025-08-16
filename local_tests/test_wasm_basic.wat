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
  (data (i32.const 4096) "\5f\5f\6d\65\5f\5f")
  (data (i32.const 4102) "\43\6f\6e\73\6f\6c\65\42\6f\78")
  (data (i32.const 4121) "\49\6e\74\65\67\65\72\42\6f\78")
  (data (i32.const 4112) "\53\74\72\69\6e\67\42\6f\78")
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
  (func $main (local $0 i32) (local $1 i32) (local $2 i32) (local $3 i32) (local $4 i32) (local $5 i32) (local $6 i32) (local $7 i32) (local $8 i32) (local $9 i32) (local $10 i32) (local $11 i32) (local $12 i32) (local $13 i32) (local $14 i32) (local $15 i32) (local $16 i32) (local $17 i32) (local $18 i32)
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
    i32.const 6
    i32.store
    call $alloc_stringbox
    local.set $2
    local.get $2
    i32.const 12
    i32.add
    i32.const 4102
    i32.store
    local.get $2
    i32.const 16
    i32.add
    i32.const 10
    i32.store
    local.get $2
    local.set $1
    local.get $0
    i32.const 12
    i32.add
    local.get $1
    i32.store
    call $alloc_stringbox
    local.set $3
    local.get $3
    i32.const 12
    i32.add
    i32.const 4096
    i32.store
    local.get $3
    i32.const 16
    i32.add
    i32.const 6
    i32.store
    local.get $3
    i32.const 12
    i32.add
    i32.load
    local.set $4
    call $alloc_stringbox
    local.set $6
    local.get $6
    i32.const 12
    i32.add
    i32.const 4112
    i32.store
    local.get $6
    i32.const 16
    i32.add
    i32.const 9
    i32.store
    local.get $6
    local.set $5
    ;; log() implementation for ValueId(4)
    local.get $4
    local.get $5
    call $console_log
    i32.const 0
    local.set $7
    call $alloc_stringbox
    local.set $9
    local.get $9
    i32.const 12
    i32.add
    i32.const 4121
    i32.store
    local.get $9
    i32.const 16
    i32.add
    i32.const 10
    i32.store
    local.get $9
    local.set $8
    call $alloc_stringbox
    local.set $10
    local.get $10
    i32.const 12
    i32.add
    i32.const 4096
    i32.store
    local.get $10
    i32.const 16
    i32.add
    i32.const 6
    i32.store
    local.get $10
    i32.const 12
    i32.add
    i32.load
    local.set $11
    call $alloc_stringbox
    local.set $13
    local.get $13
    i32.const 12
    i32.add
    i32.const 4112
    i32.store
    local.get $13
    i32.const 16
    i32.add
    i32.const 9
    i32.store
    local.get $13
    local.set $12
    ;; toString() implementation for ValueId(8)
    local.get $8
    call $box_to_string
    local.set $14
    local.get $12
    local.get $14
    i32.add
    local.set $15
    ;; log() implementation for ValueId(11)
    local.get $11
    local.get $15
    call $console_log
    i32.const 0
    local.set $16
    call $alloc_stringbox
    local.set $18
    local.get $18
    i32.const 12
    i32.add
    i32.const 4112
    i32.store
    local.get $18
    i32.const 16
    i32.add
    i32.const 9
    i32.store
    local.get $18
    local.set $17
    local.get $17
    return
  )
  (export "main" (func $main))
)

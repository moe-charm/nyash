(module
  (import "env" "print" (func $print (param i32) ))
  (memory (export "memory") 1)
  (global $heap_ptr (mut i32) (i32.const 2048))
  (func $main (local $0 i32)
    nop
    i32.const 42
    local.set $0
    local.get $0
    return
  )
  (export "main" (func $main))
)

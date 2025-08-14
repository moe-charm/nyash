(module
  (func $main (result i32)
    (local $x i32)
    (local $y i32)
    (local $result i32)
    
    i32.const 10
    local.set $x
    
    i32.const 20
    local.set $y
    
    local.get $x
    local.get $y
    i32.add
    local.set $result
    
    local.get $result
  )
  (export "main" (func $main))
)
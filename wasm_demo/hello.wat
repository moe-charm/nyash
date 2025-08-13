;; Simple WASM demo - Hello from Nyash!
(module
  ;; Import console.log from JavaScript
  (import "console" "log" (func $log (param i32)))
  
  ;; Memory for string storage
  (memory (export "memory") 1)
  
  ;; Store "Hello Nyash!" at memory offset 0
  (data (i32.const 0) "Hello Nyash! 42\00")
  
  ;; Function to get string pointer
  (func (export "getHelloString") (result i32)
    i32.const 0  ;; Return pointer to string
  )
  
  ;; Function to add two numbers
  (func (export "add") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
  
  ;; Main function that logs 42
  (func (export "main") 
    i32.const 42
    call $log
  )
)
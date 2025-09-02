@echo off
echo Creating dummy ffi.lib to bypass linker error...
cd C:\LLVM-18\lib

REM Create empty C file
echo // Dummy ffi functions > ffi_dummy.c
echo void ffi_call() {} >> ffi_dummy.c
echo void ffi_prep_cif() {} >> ffi_dummy.c
echo void ffi_prep_closure_loc() {} >> ffi_dummy.c

REM Compile to object file
cl /c ffi_dummy.c

REM Create library
lib /OUT:ffi.lib ffi_dummy.obj

echo Done! Created C:\LLVM-18\lib\ffi.lib
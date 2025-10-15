# Create dummy ffi.lib
$dummyContent = @"
// Dummy FFI functions for AOT-only build
void ffi_call() {}
void ffi_prep_cif() {}
void ffi_prep_closure_loc() {}
void ffi_type_void() {}
"@

# Write C file
$dummyContent | Out-File -FilePath "C:\LLVM-18\lib\ffi_dummy.c" -Encoding UTF8

Write-Host "Created dummy C file at C:\LLVM-18\lib\ffi_dummy.c"
Write-Host "Now you need to compile it with Visual Studio Developer Command Prompt:"
Write-Host "  cd C:\LLVM-18\lib"
Write-Host "  cl /c ffi_dummy.c"
Write-Host "  lib /OUT:ffi.lib ffi_dummy.obj"
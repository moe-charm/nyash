# Windows Linking Helpers (Frozen v1)

This folder contains minimal helpers for linking a Hakorune program on Windows. Two toolchains are considered:

- MinGW/Clang (lld): produces `libhako_kernel.a`. Recommended when using Clang on Windows.
- MSVC/clang-cl + link.exe: produces `hako_kernel.lib`. Use from a Developer Command Prompt.

MinGW/Clang (example)
```
REM Generate objects on WSL or Windows, then link on Windows
clang build\obj\main.o -o bin\hako-frozen-v1.exe ^
  -Wl,--whole-archive crates\hako_kernel\target\release\libhako_kernel.a -Wl,--no-whole-archive ^
  -lws2_32 -lbcrypt
```

MSVC/clang-cl (example)
```
REM From a Developer Command Prompt (LIB/INCLUDE configured)
clang-cl /Fe:bin\hako-frozen-v1.exe build\obj\main.obj ^
  /link /LIBPATH:crates\hako_kernel\target\release hako_kernel.lib
```

Notes
- If you don’t have a static runtime yet, you can link a tiny C stub (provided at `link_stub_main.c`) with your `.o` to run a minimal program that returns a code via `ny_main()`.
- When the linker complains about unresolved `nyash.box.from_i8_string` or `nyash.string.concat_hh` (runtime helpers), and you don’t have `hako_kernel.lib` yet, you can provide dev stubs:
  - MinGW/Clang:
    ```
    clang -c tools/aot/windows/nyrt_min_stubs_win.S -o build/obj/nyrt_min_stubs_win.obj
    ```
    Then add `build/obj/nyrt_min_stubs_win.obj` to your link line.
  - MSVC/clang-cl:
    Prefer using the `clang` driver to assemble the `.S` file (gas syntax). If you must stay on clang-cl, provide a MASM equivalent or link the static runtime instead.
- For multi-obj linking where some generated helper symbols collide, add `/FORCE:MULTIPLE` (MSVC) or `-Wl,--allow-multiple-definition` (MinGW) as a dev-only workaround.
- Prefer generating `.o` on WSL and linking on Windows during bootstrap to reduce environment variance.

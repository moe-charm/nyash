Minimal JSON Builder (selfhost-only)

Scope
- This directory provides a tiny, selfhost-only JSON string builder focused on MIR(JSON v0) scaffolds.
- Goal: eliminate fragile string concatenation and escaping issues in tests and selfhost drivers.

Policy
- Builder is minimal and local to selfhost; it does not change core runtime or public specs.
- Fail-Fast: avoid silent fallbacks; prefer well-formed output or explicit early errors in the builder.

Usage (pipeline-friendly)
- Enable syntax sugar if needed: NYASH_SYNTAX_SUGAR_LEVEL=basic|full
- Example:
  using selfhost.common.json.mir_builder_min as Mb
  using selfhost.vm.mir_min as MirVmMin

  static box Main {
    main() {
      local j = Mb.new()
        |> Mb.start_module()
        |> Mb.start_function("main")
        |> Mb.start_block(0)
        |> Mb.add_const(1, 5)
        |> Mb.add_const(2, 4)
        |> Mb.add_compare("Gt", 1, 2, 3)
        |> Mb.add_ret(3)
        |> Mb.end_all()
        |> Mb.to_string()
      return MirVmMin.run(j)
    }
  }


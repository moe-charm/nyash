"""
call_resolver/tracer.py - Debug tracing for call argument resolution

Box Theory Principle: "見える化" - Make argument resolution observable
"""
import sys
import os
from typing import Optional, Any, Dict
from .types import ArgResolveResult, ArgResolveFailureReason


class CallArgTracerBox:
    """
    Debug tracer for call argument resolution (Box Theory: observability)

    Enable with: NYASH_LLVM_CALL_ARG_TRACE=1

    Unlike vmap tracer, this focuses on:
    - Which resolution path was used (vmap direct, legacy, phi dispatch)
    - Type coercion steps (i8* → i64, ptr → handle)
    - Performance metrics (registry rebuild count)
    """

    def __init__(self):
        self.enabled = os.environ.get('NYASH_LLVM_CALL_ARG_TRACE', '0') == '1'
        self.resolution_count = 0
        self.registry_rebuild_count = 0

    def start_resolve(self, vid: int, context_info: str = ""):
        """Start tracking argument resolution"""
        if not self.enabled:
            return

        self.resolution_count += 1
        print(
            f"[call-arg-trace] 🔍 RESOLVE vid={vid} {context_info}",
            file=sys.stderr
        )

    def attempt(self, path: str, vid: int, details: Optional[str] = None):
        """Record a resolution attempt"""
        if not self.enabled:
            return

        msg = f"[call-arg-trace]   → {path}: vid={vid}"
        if details:
            msg += f" ({details})"
        print(msg, file=sys.stderr)

    def success(self, path: str, vid: int, value: Any):
        """Record successful resolution"""
        if not self.enabled:
            return

        value_str = str(value)[:50] if value else "None"
        print(
            f"[call-arg-trace]   ✅ SUCCESS via {path}: vid={vid} → {value_str}",
            file=sys.stderr
        )

    def failure(self, path: str, vid: int, reason: str):
        """Record failed resolution attempt"""
        if not self.enabled:
            return

        print(
            f"[call-arg-trace]   ❌ FAILED at {path}: vid={vid} - {reason}",
            file=sys.stderr
        )

    def type_coercion(self, from_type: str, to_type: str, vid: int):
        """Record type coercion step"""
        if not self.enabled:
            return

        print(
            f"[call-arg-trace]   🔄 TYPE COERCE: vid={vid} {from_type} → {to_type}",
            file=sys.stderr
        )

    def registry_rebuild(self, vmap_size: int):
        """Record vmap registry rebuild (performance metric)"""
        if not self.enabled:
            return

        self.registry_rebuild_count += 1
        print(
            f"[call-arg-trace]   ⚠️  REGISTRY REBUILD #{self.registry_rebuild_count} (vmap size={vmap_size})",
            file=sys.stderr
        )

    def final_result(self, vid: int, result: ArgResolveResult):
        """Record final resolution result"""
        if not self.enabled:
            return

        if result.success:
            print(
                f"[call-arg-trace] ✅ FINAL SUCCESS: vid={vid}",
                file=sys.stderr
            )
        else:
            print(
                f"[call-arg-trace] 🚫 FINAL FAILURE: vid={vid} - {result.reason.value}",
                file=sys.stderr
            )
            if result.diagnostics:
                print(
                    f"[call-arg-trace]   Diagnostics: {result.diagnostics}",
                    file=sys.stderr
                )

    def print_summary(self):
        """Print summary statistics"""
        if not self.enabled:
            return

        print("\n[call-arg-trace] === SUMMARY ===", file=sys.stderr)
        print(f"[call-arg-trace] Total resolutions: {self.resolution_count}", file=sys.stderr)
        print(f"[call-arg-trace] Registry rebuilds: {self.registry_rebuild_count}", file=sys.stderr)
        if self.registry_rebuild_count > 0:
            print(
                f"[call-arg-trace] ⚠️  Performance issue: Registry rebuilt {self.registry_rebuild_count} times",
                file=sys.stderr
            )

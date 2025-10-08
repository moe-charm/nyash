"""
vmap/tracer.py - Debug tracing for vmap resolution

Box Theory Principle: "見える化" - Make vmap resolution observable
"""
import sys
import os
import time
from typing import List, Dict, Optional, Any
from .types import ResolveContext, ResolveResult, ResolveFailureReason


class VmapTracerBox:
    """
    Debug tracer for vmap resolution (Box Theory: observability)

    Enable with: NYASH_LLVM_VMAP_TRACE=1
    """

    def __init__(self):
        self.enabled = os.environ.get('NYASH_LLVM_VMAP_TRACE', '0') == '1'
        self.history: List[Dict[str, Any]] = []
        self._current_resolve: Optional[Dict[str, Any]] = None

    def start_resolve(self, vid: int, context: ResolveContext):
        """Start tracking a resolution attempt"""
        if not self.enabled:
            return

        self._current_resolve = {
            "vid": vid,
            "block": context.block.name if context.block else None,
            "timestamp": time.time(),
            "attempts": []
        }

        print(
            f"[vmap-trace] 🔍 START resolve vid={vid} in block={self._current_resolve['block']}",
            file=sys.stderr
        )

    def attempt(self, tier: str, vid: int, details: Optional[str] = None):
        """Record a resolution tier attempt"""
        if not self.enabled or not self._current_resolve:
            return

        msg = f"[vmap-trace]   Tier {tier}: vid={vid}"
        if details:
            msg += f" ({details})"
        print(msg, file=sys.stderr)

    def success(self, tier: str, vid: int, value: Any):
        """Record successful resolution"""
        if not self.enabled or not self._current_resolve:
            return

        self._current_resolve["attempts"].append({
            "tier": tier,
            "success": True,
            "value": str(value)[:50]  # Truncate for readability
        })

        print(
            f"[vmap-trace]   ✅ SUCCESS via {tier}: vid={vid} → {str(value)[:50]}",
            file=sys.stderr
        )

        self._finish_resolve()

    def failure(self, tier: str, vid: int, reason: str):
        """Record failed resolution attempt"""
        if not self.enabled or not self._current_resolve:
            return

        self._current_resolve["attempts"].append({
            "tier": tier,
            "success": False,
            "reason": reason
        })

        print(
            f"[vmap-trace]   ❌ FAILED at {tier}: vid={vid} - {reason}",
            file=sys.stderr
        )

    def end_failure(self, vid: int, reason: ResolveFailureReason, diagnostics: Dict):
        """Record final failure (all tiers exhausted)"""
        if not self.enabled or not self._current_resolve:
            return

        print(
            f"[vmap-trace] 🚫 FINAL FAILURE: vid={vid} - {reason.value}",
            file=sys.stderr
        )
        print(
            f"[vmap-trace]   Diagnostics: {diagnostics}",
            file=sys.stderr
        )

        self._finish_resolve()

    def _finish_resolve(self):
        """Finish current resolution tracking"""
        if self._current_resolve:
            self.history.append(self._current_resolve)
            self._current_resolve = None

    def get_diagnostics(self, vid: int) -> Dict[str, Any]:
        """Get full resolution history for a vid"""
        return {
            "history": [h for h in self.history if h["vid"] == vid],
            "total_attempts": len(self.history)
        }

    def print_summary(self):
        """Print summary statistics"""
        if not self.enabled or not self.history:
            return

        total = len(self.history)
        successes = sum(1 for h in self.history if any(a["success"] for a in h["attempts"]))
        failures = total - successes

        print("\n[vmap-trace] === SUMMARY ===", file=sys.stderr)
        print(f"[vmap-trace] Total resolutions: {total}", file=sys.stderr)
        print(f"[vmap-trace] Successes: {successes} ({100*successes//total if total else 0}%)", file=sys.stderr)
        print(f"[vmap-trace] Failures: {failures} ({100*failures//total if total else 0}%)", file=sys.stderr)

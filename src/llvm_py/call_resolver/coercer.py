"""
call_resolver/coercer.py - Unified type coercion for call arguments

Box Theory Principle: "箱にする" - Centralize all type conversion logic
"""
from typing import Optional, Any
try:
    from llvmlite import ir
except ImportError:
    ir = None

from .types import ArgResolveResult, ArgResolveFailureReason
from .tracer import CallArgTracerBox


class TypeCoercerBox:
    """
    Unified type conversion for call arguments (Box Theory: SSOT)

    OLD CODE PROBLEM:
    - Lines 356-363 (lower_method_call): One coercion strategy
    - Lines 542-549 (lower_constructor_call): Different coercion
    - Lines 829-838 (lower_extern_call): Yet another variant
    → Result: Same i8* pointer coerced 3 different ways!

    NEW CODE:
    - Single TypeCoercerBox
    - Consistent coercion across all 8 call types
    - Explicit tracing of conversions
    """

    def __init__(self, builder, module, tracer: Optional[CallArgTracerBox] = None):
        self.builder = builder
        self.module = module
        self.tracer = tracer or CallArgTracerBox()
        self.i64 = ir.IntType(64) if ir else None
        self.i8 = ir.IntType(8) if ir else None
        self.i8p = self.i8.as_pointer() if self.i8 else None

    def to_i64_handle(self, value: Any, vid: Optional[int] = None) -> Any:
        """
        Convert any value to i64 handle (unified coercion)

        Handles:
        - i64 → i64 (pass through)
        - i1/i8/i16/i32 → i64 (zext)
        - i65+ → i64 (trunc)
        - ptr → i64 (via boxing if needed)

        Args:
            value: LLVM IR value
            vid: Original vid (for tracing, optional)

        Returns:
            Coerced i64 value
        """
        if value is None or ir is None:
            return value

        vid_str = f"vid={vid}" if vid is not None else ""

        # Already i64
        if isinstance(value.type, ir.IntType) and value.type.width == 64:
            return value

        # Integer coercion
        if isinstance(value.type, ir.IntType):
            from_type = f"i{value.type.width}"
            to_type = "i64"

            if value.type.width < 64:
                # Narrow int → i64 (zext)
                self.tracer.type_coercion(from_type, to_type, vid if vid else -1)
                return self.builder.zext(value, self.i64, name=f"zext_to_i64")
            elif value.type.width > 64:
                # Wide int → i64 (trunc)
                self.tracer.type_coercion(from_type, to_type, vid if vid else -1)
                return self.builder.trunc(value, self.i64, name=f"trunc_to_i64")

        # Pointer coercion
        if hasattr(value.type, 'is_pointer') and value.type.is_pointer:
            from_type = "ptr"
            to_type = "i64"

            # Check if it's an i8* (string pointer that needs boxing)
            if isinstance(value.type.pointee, ir.IntType) and value.type.pointee.width == 8:
                # i8* → handle (requires boxing)
                self.tracer.type_coercion("i8*", "i64_handle", vid if vid else -1)
                return self._box_string_ptr(value)
            else:
                # Generic pointer → i64 (ptrtoint)
                self.tracer.type_coercion(from_type, to_type, vid if vid else -1)
                return self.builder.ptrtoint(value, self.i64, name=f"ptr_to_i64")

        # Unknown type - pass through and hope for the best
        # (LLVM will error if incompatible)
        return value

    def _box_string_ptr(self, ptr_value: Any) -> Any:
        """
        Convert i8* to string handle via nyash.box.from_i8_string

        This is the ONLY correct way to convert i8* → i64 handle in Hakorune.

        OLD CODE PROBLEMS:
        - Some places used ptrtoint (WRONG - not a handle!)
        - Some places called from_i8_string (CORRECT)
        - Inconsistent behavior across call types

        NEW CODE:
        - Always call from_i8_string for i8*
        - Consistent handle generation
        """
        if ir is None:
            return ptr_value

        # Find or declare nyash.box.from_i8_string
        func = self._get_or_declare_function(
            "nyash.box.from_i8_string",
            self.i64,
            [self.i8p]
        )

        # Call to convert i8* → handle
        return self.builder.call(func, [ptr_value], name="ptr2handle")

    def _get_or_declare_function(self, name: str, ret_type, arg_types: list):
        """
        Get existing function or declare it

        This is a common pattern in LLVM codegen - extract to helper.
        """
        # Try to find existing function
        for f in self.module.functions:
            if f.name == name:
                return f

        # Declare new function
        func_type = ir.FunctionType(ret_type, arg_types)
        return ir.Function(self.module, func_type, name=name)

    def coerce_arg_list(self, vids: list, resolver) -> list:
        """
        Coerce a list of argument vids to i64 handles

        This is a convenience method for the common pattern:
            args_resolved = []
            for vid in args:
                value = resolver.resolve_i64_or_zero(vid)
                coerced = coercer.to_i64_handle(value, vid)
                args_resolved.append(coerced)

        Returns:
            List of coerced i64 values
        """
        result = []
        for vid in vids:
            value = resolver.resolve_i64_or_zero(vid)
            coerced = self.to_i64_handle(value, vid)
            result.append(coerced)
        return result

use super::super::builder::IRBuilder;
use super::LowerCore;

impl LowerCore {
    /// Emit robust length retrieval with fallback for String/Any:
    /// 1) Prefer `nyash.string.len_h(recv)`
    /// 2) If that yields 0 at runtime, select `nyash.any.length_h(recv)`
    /// Returns: pushes selected length (i64) onto builder stack.
    pub(super) fn emit_len_with_fallback_param(&mut self, b: &mut dyn IRBuilder, pidx: usize) {
        use super::super::builder::CmpKind;
        // Temp locals
        let hslot = self.next_local;
        self.next_local += 1; // receiver handle slot
        let t_string = self.next_local;
        self.next_local += 1;
        let t_any = self.next_local;
        self.next_local += 1;
        let t_cond = self.next_local;
        self.next_local += 1;
        // Materialize receiver handle from param index
        b.emit_param_i64(pidx);
        b.emit_host_call(crate::jit::r#extern::handles::SYM_HANDLE_OF, 1, true);
        b.store_local_i64(hslot);
        // String.len_h
        crate::jit::observe::lower_hostcall(
            crate::jit::r#extern::collections::SYM_STRING_LEN_H,
            1,
            &["Handle"],
            "allow",
            "core_len_param",
        );
        b.load_local_i64(hslot);
        b.emit_host_call(crate::jit::r#extern::collections::SYM_STRING_LEN_H, 1, true);
        b.store_local_i64(t_string);
        // debug: observe string len
        b.emit_debug_i64_local(1100, t_string);
        // Any.length_h
        crate::jit::observe::lower_hostcall(
            crate::jit::r#extern::collections::SYM_ANY_LEN_H,
            1,
            &["Handle"],
            "allow",
            "core_len_param",
        );
        b.load_local_i64(hslot);
        b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_LEN_H, 1, true);
        b.store_local_i64(t_any);
        // debug: observe any len
        b.emit_debug_i64_local(1101, t_any);
        // cond = (string_len == 0)
        b.load_local_i64(t_string);
        b.emit_const_i64(0);
        b.emit_compare(CmpKind::Eq);
        b.store_local_i64(t_cond);
        // debug: observe condition
        b.emit_debug_i64_local(1102, t_cond);
        // select(cond ? any_len : string_len)
        b.load_local_i64(t_cond); // cond (bottom)
        b.load_local_i64(t_any); // then
        b.load_local_i64(t_string); // else
        b.emit_select_i64();
    }

    pub(super) fn emit_len_with_fallback_local_handle(
        &mut self,
        b: &mut dyn IRBuilder,
        slot: usize,
    ) {
        use super::super::builder::CmpKind;
        let t_string = self.next_local;
        self.next_local += 1;
        let t_any = self.next_local;
        self.next_local += 1;
        let t_cond = self.next_local;
        self.next_local += 1;
        // String.len_h
        crate::jit::observe::lower_hostcall(
            crate::jit::r#extern::collections::SYM_STRING_LEN_H,
            1,
            &["Handle"],
            "allow",
            "core_len_local",
        );
        b.load_local_i64(slot);
        b.emit_host_call(crate::jit::r#extern::collections::SYM_STRING_LEN_H, 1, true);
        b.store_local_i64(t_string);
        b.emit_debug_i64_local(1200, t_string);
        // Any.length_h
        crate::jit::observe::lower_hostcall(
            crate::jit::r#extern::collections::SYM_ANY_LEN_H,
            1,
            &["Handle"],
            "allow",
            "core_len_local",
        );
        b.load_local_i64(slot);
        b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_LEN_H, 1, true);
        b.store_local_i64(t_any);
        b.emit_debug_i64_local(1201, t_any);
        // cond = (string_len == 0)
        b.load_local_i64(t_string);
        b.emit_const_i64(0);
        b.emit_compare(CmpKind::Eq);
        b.store_local_i64(t_cond);
        b.emit_debug_i64_local(1202, t_cond);
        // select(cond ? any_len : string_len)
        b.load_local_i64(t_cond);
        b.load_local_i64(t_any);
        b.load_local_i64(t_string);
        b.emit_select_i64();
    }

    pub(super) fn emit_len_with_fallback_literal(&mut self, b: &mut dyn IRBuilder, s: &str) {
        use super::super::builder::CmpKind;
        let t_string = self.next_local;
        self.next_local += 1;
        let t_any = self.next_local;
        self.next_local += 1;
        let t_cond = self.next_local;
        self.next_local += 1;
        // String.len_h on literal handle
        crate::jit::observe::lower_hostcall(
            crate::jit::r#extern::collections::SYM_STRING_LEN_H,
            1,
            &["Handle"],
            "allow",
            "core_len_lit",
        );
        b.emit_string_handle_from_literal(s);
        b.emit_host_call(crate::jit::r#extern::collections::SYM_STRING_LEN_H, 1, true);
        b.store_local_i64(t_string);
        b.emit_debug_i64_local(1300, t_string);
        // Any.length_h on literal handle (recreate handle; safe in v0)
        crate::jit::observe::lower_hostcall(
            crate::jit::r#extern::collections::SYM_ANY_LEN_H,
            1,
            &["Handle"],
            "allow",
            "core_len_lit",
        );
        b.emit_string_handle_from_literal(s);
        b.emit_host_call(crate::jit::r#extern::collections::SYM_ANY_LEN_H, 1, true);
        b.store_local_i64(t_any);
        b.emit_debug_i64_local(1301, t_any);
        // cond = (string_len == 0)
        b.load_local_i64(t_string);
        b.emit_const_i64(0);
        b.emit_compare(CmpKind::Eq);
        b.store_local_i64(t_cond);
        b.emit_debug_i64_local(1302, t_cond);
        // select(cond ? any_len : string_len)
        b.load_local_i64(t_cond);
        b.load_local_i64(t_any);
        b.load_local_i64(t_string);
        b.emit_select_i64();
    }
}

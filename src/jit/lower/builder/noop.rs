use super::{BinOpKind, CmpKind, IRBuilder, ParamKind};

pub struct NoopBuilder {
    pub consts: usize,
    pub binops: usize,
    pub cmps: usize,
    pub branches: usize,
    pub rets: usize,
}

impl NoopBuilder {
    pub fn new() -> Self {
        Self {
            consts: 0,
            binops: 0,
            cmps: 0,
            branches: 0,
            rets: 0,
        }
    }
}

impl IRBuilder for NoopBuilder {
    fn begin_function(&mut self, _name: &str) {}
    fn end_function(&mut self) {}
    fn emit_param_i64(&mut self, _index: usize) {
        self.consts += 1;
    }
    fn emit_const_i64(&mut self, _val: i64) {
        self.consts += 1;
    }
    fn emit_const_f64(&mut self, _val: f64) {
        self.consts += 1;
    }
    fn emit_binop(&mut self, _op: BinOpKind) {
        self.binops += 1;
    }
    fn emit_compare(&mut self, _op: CmpKind) {
        self.cmps += 1;
    }
    fn emit_jump(&mut self) {
        self.branches += 1;
    }
    fn emit_branch(&mut self) {
        self.branches += 1;
    }
    fn emit_return(&mut self) {
        self.rets += 1;
    }
    fn emit_select_i64(&mut self) {
        self.binops += 1;
    }
    fn emit_host_call_typed(
        &mut self,
        _symbol: &str,
        _params: &[ParamKind],
        has_ret: bool,
        _ret_is_f64: bool,
    ) {
        if has_ret {
            self.consts += 1;
        }
    }
    fn emit_host_call_fixed3(&mut self, _symbol: &str, has_ret: bool) {
        if has_ret {
            self.consts += 1;
        }
    }
    fn emit_plugin_invoke(&mut self, _type_id: u32, _method_id: u32, _argc: usize, has_ret: bool) {
        if has_ret {
            self.consts += 1;
        }
    }
    fn emit_plugin_invoke_by_name(&mut self, _method: &str, _argc: usize, has_ret: bool) {
        if has_ret {
            self.consts += 1;
        }
    }
    fn emit_string_handle_from_literal(&mut self, _s: &str) {
        self.consts += 1;
    }
    fn ensure_local_i64(&mut self, _index: usize) {}
    fn store_local_i64(&mut self, _index: usize) {
        self.consts += 1;
    }
    fn load_local_i64(&mut self, _index: usize) {
        self.consts += 1;
    }
}

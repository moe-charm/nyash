pub mod invoke;
pub mod config_resolver;

use std::sync::{OnceLock, Mutex};

#[derive(Clone, Debug, Default)]
pub struct JitPolicyState {
    pub read_only: bool,
    pub hostcall_whitelist: Vec<String>,
}

static STATE: OnceLock<Mutex<JitPolicyState>> = OnceLock::new();

fn state() -> &'static Mutex<JitPolicyState> {
    STATE.get_or_init(|| Mutex::new(JitPolicyState::default()))
}

pub fn current() -> JitPolicyState {
    state().lock().unwrap().clone()
}

pub fn set_current(s: JitPolicyState) {
    *state().lock().unwrap() = s;
}


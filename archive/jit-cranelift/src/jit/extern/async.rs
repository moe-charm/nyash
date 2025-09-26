//! Async/Future-related JIT extern symbols

#[allow(unused_imports)]
use crate::{
    backend::vm::VMValue,
    box_trait::{BoolBox, IntegerBox, NyashBox, StringBox},
};

/// Symbol name for awaiting a FutureBox and returning a value/handle (i64)
pub const SYM_FUTURE_AWAIT_H: &str = "nyash.future.await_h";
pub const SYM_FUTURE_SPAWN_INSTANCE3_I64: &str = "nyash.future.spawn_instance3_i64";

#[cfg(feature = "cranelift-jit")]
pub extern "C" fn nyash_future_await_h(arg0: i64) -> i64 {
    use crate::jit::rt::handles;

    // Resolve FutureBox from handle or legacy VM args
    let mut fut_opt: Option<crate::boxes::future::FutureBox> = None;
    if arg0 > 0 {
        if let Some(obj) = handles::get(arg0 as u64) {
            if let Some(fb) = obj
                .as_any()
                .downcast_ref::<crate::boxes::future::FutureBox>()
            {
                fut_opt = Some(fb.clone());
            }
        }
    }
    #[cfg(not(feature = "jit-direct-only"))]
    if fut_opt.is_none() {
        crate::jit::rt::with_legacy_vm_args(|args| {
            let pick = if arg0 >= 0 {
                (arg0 as usize)..(arg0 as usize + 1)
            } else {
                0..args.len()
            };
            for i in pick {
                if let Some(VMValue::BoxRef(b)) = args.get(i) {
                    if let Some(fb) = b.as_any().downcast_ref::<crate::boxes::future::FutureBox>() {
                        fut_opt = Some(fb.clone());
                        break;
                    }
                }
            }
        });
    }
    let Some(fut) = fut_opt else {
        return 0;
    };
    // Cooperative wait with scheduler polling and timeout
    let max_ms: u64 = crate::config::env::await_max_ms();
    let start = std::time::Instant::now();
    while !fut.ready() {
        crate::runtime::global_hooks::safepoint_and_poll();
        std::thread::yield_now();
        if start.elapsed() >= std::time::Duration::from_millis(max_ms) {
            // Timeout: return 0 (caller may handle as failure)
            return 0;
        }
    }
    // Get NyashBox result and always return a handle
    let out_box: Box<dyn NyashBox> = fut.get();
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(out_box);
    let h = handles::to_handle(arc);
    h as i64
}

//! Async/Future-related JIT extern symbols

#[allow(unused_imports)]
use crate::{backend::vm::VMValue, box_trait::{NyashBox, IntegerBox, BoolBox, StringBox}};

/// Symbol name for awaiting a FutureBox and returning a value/handle (i64)
pub const SYM_FUTURE_AWAIT_H: &str = "nyash.future.await_h";

#[cfg(feature = "cranelift-jit")]
pub extern "C" fn nyash_future_await_h(arg0: i64) -> i64 {
    use crate::jit::rt::{handles, with_legacy_vm_args};

    // Resolve FutureBox from handle or legacy VM args
    let mut fut_opt: Option<crate::boxes::future::FutureBox> = None;
    if arg0 > 0 {
        if let Some(obj) = handles::get(arg0 as u64) {
            if let Some(fb) = obj.as_any().downcast_ref::<crate::boxes::future::FutureBox>() {
                fut_opt = Some(fb.clone());
            }
        }
    }
    if fut_opt.is_none() {
        with_legacy_vm_args(|args| {
            // If arg0>=0 treat as positional index, else scan for first FutureBox
            let pick = if arg0 >= 0 { (arg0 as usize)..(arg0 as usize + 1) } else { 0..args.len() };
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
    let Some(fut) = fut_opt else { return 0; };
    // Block until completion, get NyashBox result
    let out_box: Box<dyn NyashBox> = fut.get();
    // Fast-path: primitive returns
    if let Some(ib) = out_box.as_any().downcast_ref::<IntegerBox>() { return ib.value; }
    if let Some(bb) = out_box.as_any().downcast_ref::<BoolBox>() { return if bb.value { 1 } else { 0 }; }
    // Otherwise, register handle and return id (works for String/Map/Array/Instance/etc.)
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(out_box);
    let h = handles::to_handle(arc);
    h as i64
}

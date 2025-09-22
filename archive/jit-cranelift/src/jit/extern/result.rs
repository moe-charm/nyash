//! Result-related JIT extern symbols

#[cfg(feature = "cranelift-jit")]
use crate::box_trait::NyashBox;

/// Symbol name for wrapping a handle into Result.Ok(handle)
pub const SYM_RESULT_OK_H: &str = "nyash.result.ok_h";
/// Symbol name for wrapping a handle into Result.Err(handle)
pub const SYM_RESULT_ERR_H: &str = "nyash.result.err_h";

#[cfg(feature = "cranelift-jit")]
pub extern "C" fn nyash_result_ok_h(handle: i64) -> i64 {
    use crate::boxes::result::NyashResultBox;
    use crate::jit::rt::handles;
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        let boxed = obj.clone_box();
        let res = NyashResultBox::new_ok(boxed);
        let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(res);
        let h = handles::to_handle(arc);
        return h as i64;
    }
    0
}

#[cfg(feature = "cranelift-jit")]
pub extern "C" fn nyash_result_err_h(handle: i64) -> i64 {
    use crate::boxes::result::NyashResultBox;
    use crate::jit::rt::handles;
    // If handle <= 0, synthesize a Timeout StringBox error for await paths.
    let err_box: Box<dyn NyashBox> = if handle <= 0 {
        Box::new(crate::box_trait::StringBox::new("Timeout".to_string()))
    } else if let Some(obj) = handles::get(handle as u64) {
        obj.clone_box()
    } else {
        Box::new(crate::box_trait::StringBox::new("UnknownError".to_string()))
    };
    let res = NyashResultBox::new_err(err_box);
    let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::new(res);
    let h = handles::to_handle(arc);
    h as i64
}

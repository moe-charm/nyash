use std::cell::RefCell;

use crate::backend::vm::VMValue;

thread_local! {
    static CURRENT_ARGS: RefCell<Vec<VMValue>> = RefCell::new(Vec::new());
}

pub fn set_current_args(args: &[VMValue]) {
    CURRENT_ARGS.with(|cell| {
        let mut v = cell.borrow_mut();
        v.clear();
        v.extend_from_slice(args);
    });
}

pub fn with_args<F, R>(f: F) -> R
where
    F: FnOnce(&[VMValue]) -> R,
{
    CURRENT_ARGS.with(|cell| {
        let v = cell.borrow();
        f(&v)
    })
}

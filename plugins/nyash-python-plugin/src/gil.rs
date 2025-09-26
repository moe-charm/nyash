use crate::ffi::{CPython, PyGILState_STATE};

pub struct GILGuard<'a> {
    cpy: &'a CPython,
    state: PyGILState_STATE,
}

impl<'a> GILGuard<'a> {
    pub fn acquire(cpy: &'a CPython) -> Self {
        let state = unsafe { (cpy.PyGILState_Ensure)() };
        GILGuard { cpy, state }
    }
}

impl<'a> Drop for GILGuard<'a> {
    fn drop(&mut self) {
        unsafe { (self.cpy.PyGILState_Release)(self.state) };
    }
}

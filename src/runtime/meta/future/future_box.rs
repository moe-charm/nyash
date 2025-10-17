//! Runtime-owned FutureBox abstraction (host-owned).
//!
//! With legacy boxes enabled we simply reuse the historical implementation.
//! In plugin-only builds we ship a lightweight clone that exposes the same
//! API surface to the VM and scheduler.

#[cfg(feature = "legacy-boxes")]
pub use crate::boxes::future::{FutureBox, FutureWeak};

#[cfg(not(feature = "legacy-boxes"))]
mod shim {
    use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
    use std::any::Any;
    use std::sync::{Arc, Condvar, Mutex, Weak};

    #[derive(Debug)]
    struct FutureState {
        result: Option<Box<dyn NyashBox>>,
        ready: bool,
    }

    #[derive(Debug)]
    struct Inner {
        state: Mutex<FutureState>,
        cv: Condvar,
    }

    #[derive(Debug)]
    pub struct FutureBox {
        inner: Arc<Inner>,
        base: BoxBase,
    }

    #[derive(Clone, Debug)]
    pub struct FutureWeak {
        inner: Weak<Inner>,
    }

    impl Clone for FutureBox {
        fn clone(&self) -> Self {
            Self { inner: self.inner.clone(), base: BoxBase::new() }
        }
    }

    impl FutureBox {
        pub fn new() -> Self {
            Self {
                inner: Arc::new(Inner { state: Mutex::new(FutureState { result: None, ready: false }), cv: Condvar::new() }),
                base: BoxBase::new(),
            }
        }
        pub fn set_result(&self, value: Box<dyn NyashBox>) {
            let mut st = self.inner.state.lock().unwrap();
            st.result = Some(value);
            st.ready = true;
            self.inner.cv.notify_all();
        }
        pub fn get(&self) -> Box<dyn NyashBox> {
            let mut st = self.inner.state.lock().unwrap();
            while !st.ready { st = self.inner.cv.wait(st).unwrap(); }
            st.result.as_ref().unwrap().clone_box()
        }
        pub fn ready(&self) -> bool { self.inner.state.lock().unwrap().ready }
        pub fn downgrade(&self) -> FutureWeak { FutureWeak { inner: Arc::downgrade(&self.inner) } }
        pub fn wait_and_get(&self) -> Result<Box<dyn NyashBox>, String> { Ok(self.get()) }
    }

    impl BoxCore for FutureBox {
        fn box_id(&self) -> u64 { self.base.id }
        fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
        fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.ready() {
                let st = self.inner.state.lock().unwrap();
                if let Some(value) = st.result.as_ref() { write!(f, "Future(ready: {})", value.to_string_box().value) }
                else { write!(f, "Future(ready: void)") }
            } else {
                write!(f, "<future>")
            }
        }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    impl NyashBox for FutureBox {
        fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
        fn share_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
        fn to_string_box(&self) -> StringBox {
            if self.ready() {
                let st = self.inner.state.lock().unwrap();
                if let Some(value) = st.result.as_ref() { StringBox::new(format!("Future(ready: {})", value.to_string_box().value)) }
                else { StringBox::new("Future(ready: void)".to_string()) }
            } else {
                StringBox::new("<future>".to_string())
            }
        }
        fn type_name(&self) -> &'static str { "FutureBox" }
        fn equals(&self, other: &dyn NyashBox) -> BoolBox {
            if let Some(o) = other.as_any().downcast_ref::<FutureBox>() { BoolBox::new(self.box_id() == o.box_id()) } else { BoolBox::new(false) }
        }
    }

    impl FutureWeak {
        pub fn is_ready(&self) -> Option<bool> {
            self.inner.upgrade().map(|arc| arc.state.lock().unwrap().ready)
        }
    }
}

#[cfg(not(feature = "legacy-boxes"))]
pub use shim::{FutureBox, FutureWeak};


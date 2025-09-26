//! FutureBox 🔄 - 非同期処理基盤
// Nyashの箱システムによる非同期処理の基盤を提供します。
// 参考: 既存Boxの設計思想

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;
use std::sync::{Arc, Condvar, Mutex, Weak};

#[derive(Debug)]
pub struct NyashFutureBox {
    inner: Arc<Inner>,
    base: BoxBase,
}

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

/// A weak handle to a Future's inner state.
/// Used for non-owning registries (TaskGroup/implicit group) to avoid leaks.
#[derive(Clone, Debug)]
pub struct FutureWeak {
    inner: Weak<Inner>,
}

impl Clone for NyashFutureBox {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            base: BoxBase::new(),
        }
    }
}

impl NyashFutureBox {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                state: Mutex::new(FutureState {
                    result: None,
                    ready: false,
                }),
                cv: Condvar::new(),
            }),
            base: BoxBase::new(),
        }
    }

    /// Set the result of the future
    pub fn set_result(&self, value: Box<dyn NyashBox>) {
        let mut st = self.inner.state.lock().unwrap();
        st.result = Some(value);
        st.ready = true;
        self.inner.cv.notify_all();
    }

    /// Get the result (blocks until ready)
    pub fn get(&self) -> Box<dyn NyashBox> {
        let mut st = self.inner.state.lock().unwrap();
        while !st.ready {
            st = self.inner.cv.wait(st).unwrap();
        }
        st.result.as_ref().unwrap().clone_box()
    }

    /// Check if the future is ready
    pub fn ready(&self) -> bool {
        self.inner.state.lock().unwrap().ready
    }

    /// Create a non-owning weak handle to this Future's state
    pub fn downgrade(&self) -> FutureWeak {
        FutureWeak {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

impl NyashBox for NyashFutureBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }

    /// 仮実装: clone_boxと同じ（後で修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }

    fn to_string_box(&self) -> StringBox {
        let ready = self.inner.state.lock().unwrap().ready;
        if ready {
            let st = self.inner.state.lock().unwrap();
            if let Some(value) = st.result.as_ref() {
                StringBox::new(format!("Future(ready: {})", value.to_string_box().value))
            } else {
                StringBox::new("Future(ready: void)".to_string())
            }
        } else {
            StringBox::new("Future(pending)".to_string())
        }
    }

    fn type_name(&self) -> &'static str {
        "NyashFutureBox"
    }

    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_future) = other.as_any().downcast_ref::<NyashFutureBox>() {
            BoolBox::new(self.base.id == other_future.base.id)
        } else {
            BoolBox::new(false)
        }
    }
}

impl BoxCore for NyashFutureBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }

    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }

    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ready = self.inner.state.lock().unwrap().ready;
        if ready {
            let st = self.inner.state.lock().unwrap();
            if let Some(value) = st.result.as_ref() {
                write!(f, "Future(ready: {})", value.to_string_box().value)
            } else {
                write!(f, "Future(ready: void)")
            }
        } else {
            write!(f, "Future(pending)")
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl std::fmt::Display for NyashFutureBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_box(f)
    }
}

// Export NyashFutureBox as FutureBox for consistency
pub type FutureBox = NyashFutureBox;

impl FutureBox {
    /// wait_and_get()の実装 - await演算子で使用
    pub fn wait_and_get(&self) -> Result<Box<dyn NyashBox>, String> {
        Ok(self.get())
    }
}

impl FutureWeak {
    /// Try to upgrade and check readiness
    pub(crate) fn is_ready(&self) -> Option<bool> {
        self.inner
            .upgrade()
            .map(|arc| arc.state.lock().unwrap().ready)
    }
}

use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox, VoidBox};
use std::any::Any;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(crate) struct TaskGroupInner {
    pub strong: Mutex<Vec<crate::boxes::future::FutureBox>>,
}

#[derive(Debug, Clone)]
pub struct TaskGroupBox {
    base: BoxBase,
    // Skeleton: cancellation token owned by this group (future wiring)
    cancelled: bool,
    pub(crate) inner: Arc<TaskGroupInner>,
}

impl TaskGroupBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            cancelled: false,
            inner: Arc::new(TaskGroupInner {
                strong: Mutex::new(Vec::new()),
            }),
        }
    }
    pub fn cancel_all(&mut self) {
        self.cancelled = true;
    }
    /// Cancel all child tasks (scaffold) and return void
    pub fn cancelAll(&mut self) -> Box<dyn NyashBox> {
        self.cancel_all();
        Box::new(VoidBox::new())
    }
    /// Join all child tasks with optional timeout (ms); returns void
    pub fn joinAll(&self, timeout_ms: Option<i64>) -> Box<dyn NyashBox> {
        let ms = timeout_ms.unwrap_or(2000).max(0) as u64;
        self.join_all_inner(ms);
        Box::new(VoidBox::new())
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    /// Register a Future into this group's ownership
    pub fn add_future(&self, fut: &crate::boxes::future::FutureBox) {
        if let Ok(mut v) = self.inner.strong.lock() {
            v.push(fut.clone());
        }
    }

    fn join_all_inner(&self, timeout_ms: u64) {
        use std::time::{Duration, Instant};
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            let mut all_ready = true;
            if let Ok(mut list) = self.inner.strong.lock() {
                list.retain(|f| !f.ready());
                if !list.is_empty() {
                    all_ready = false;
                }
            }
            if all_ready {
                break;
            }
            if Instant::now() >= deadline {
                break;
            }
            crate::runtime::global_hooks::safepoint_and_poll();
            std::thread::yield_now();
        }
    }
}

impl BoxCore for TaskGroupBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        None
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TaskGroup(cancelled={})", self.cancelled)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for TaskGroupBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("TaskGroup(cancelled={})", self.cancelled))
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(g) = other.as_any().downcast_ref::<TaskGroupBox>() {
            BoolBox::new(self.base.id == g.base.id)
        } else {
            BoolBox::new(false)
        }
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

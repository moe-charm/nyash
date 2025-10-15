//! Static singleton registry for Phase-31 box normalization.
//!
//! Provides a lazily-initialized `Arc<dyn NyashBox>` per box name so that
//! the interpreter can materialize `me` for static methods without re-creating
//! instances on every call.

use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::box_trait::NyashBox;

type BoxMap = HashMap<String, Arc<dyn NyashBox>>;

static SINGLETONS: OnceCell<Mutex<BoxMap>> = OnceCell::new();

fn store() -> &'static Mutex<BoxMap> {
    SINGLETONS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Obtain the singleton instance for `box_name`, creating it on demand.
pub fn get(box_name: &str) -> Result<Arc<dyn NyashBox>, crate::backend::vm_types::VMError> {
    {
        let guard = store().lock().unwrap();
        if let Some(existing) = guard.get(box_name) {
            return Ok(existing.clone());
        }
    }

    let registry = crate::runtime::get_global_registry();
    let created = registry
        .create_box(box_name, &[])
        .map_err(|err| crate::backend::vm_types::VMError::InvalidInstruction(format!(
            "Failed to create singleton {}: {:?}",
            box_name, err
        )))?;
    let arc: Arc<dyn NyashBox> = Arc::from(created);

    let mut guard = store().lock().unwrap();
    let entry = guard.entry(box_name.to_string()).or_insert_with(|| arc.clone());
    Ok(entry.clone())
}

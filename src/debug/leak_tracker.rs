use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static ENABLED: Lazy<bool> = Lazy::new(|| std::env::var("NYASH_LEAK_LOG").unwrap_or_default() == "1");
static LEAKS: Lazy<Mutex<HashMap<(String, u32), &'static str>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn init() {
    let _ = &*REPORTER; // ensure reporter is constructed
}

pub fn register_plugin(box_type: &str, instance_id: u32) {
    if !*ENABLED { return; }
    let mut m = LEAKS.lock().unwrap();
    m.insert((box_type.to_string(), instance_id), "plugin");
}

pub fn finalize_plugin(box_type: &str, instance_id: u32) {
    if !*ENABLED { return; }
    let mut m = LEAKS.lock().unwrap();
    m.remove(&(box_type.to_string(), instance_id));
}

struct Reporter;
impl Drop for Reporter {
    fn drop(&mut self) {
        if !*ENABLED { return; }
        let m = LEAKS.lock().unwrap();
        if m.is_empty() { return; }
        eprintln!("[leak] Detected {} non-finalized plugin boxes:", m.len());
        for ((ty, id), _) in m.iter() {
            eprintln!("  - {}(id={}) not finalized (missing fini or scope)", ty, id);
        }
    }
}

static REPORTER: Lazy<Reporter> = Lazy::new(|| Reporter);


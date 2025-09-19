use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Default)]
pub struct MacroCaps {
    pub io: bool,
    pub net: bool,
    pub env: bool,
}

#[derive(Debug, Clone)]
pub struct MacroCtx {
    pub caps: MacroCaps,
}

impl MacroCtx {
    pub fn from_env() -> Self {
        fn on(name: &str) -> bool {
            std::env::var(name)
                .ok()
                .map(|v| {
                    let v = v.to_ascii_lowercase();
                    v == "1" || v == "true" || v == "on"
                })
                .unwrap_or(false)
        }
        MacroCtx {
            caps: MacroCaps {
                io: on("NYASH_MACRO_CAP_IO"),
                net: on("NYASH_MACRO_CAP_NET"),
                env: on("NYASH_MACRO_CAP_ENV"),
            },
        }
    }

    pub fn gensym(&self, prefix: &str) -> String { gensym(prefix) }

    pub fn report(&self, level: &str, message: &str) {
        eprintln!("[macro][{}] {}", level, message);
    }

    pub fn get_env(&self, key: &str) -> Option<String> {
        if !self.caps.env { return None; }
        std::env::var(key).ok()
    }
}

static GENSYM_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn gensym(prefix: &str) -> String {
    let n = GENSYM_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}_{}", prefix, n)
}

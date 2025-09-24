use once_cell::sync::Lazy;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use std::sync::Mutex;

static LOG_ON: Lazy<bool> = Lazy::new(|| std::env::var("NYASH_NET_LOG").unwrap_or_default() == "1");
static LOG_PATH: Lazy<String> = Lazy::new(|| {
    std::env::var("NYASH_NET_LOG_FILE").unwrap_or_else(|_| "net_plugin.log".to_string())
});
static LOG_MTX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub(crate) fn net_log(msg: &str) {
    if !*LOG_ON {
        return;
    }
    eprintln!("[net] {}", msg);
    let _g = LOG_MTX.lock().unwrap();
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&*LOG_PATH)
    {
        let _ = writeln!(f, "[{:?}] {}", std::time::SystemTime::now(), msg);
    }
}

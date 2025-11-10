use std::fs::OpenOptions;
use std::sync::Mutex;

pub static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub fn init_debug_log() {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("/tmp/term39_debug.log")
        .ok();

    *LOG_FILE.lock().unwrap() = file;
}

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {{
        use std::io::Write;
        if let Some(file) = $crate::debug_log::LOG_FILE.lock().unwrap().as_mut() {
            let _ = writeln!(file, "[{}] {}", chrono::Local::now().format("%H:%M:%S%.3f"), format!($($arg)*));
            let _ = file.flush();
        }
    }};
}

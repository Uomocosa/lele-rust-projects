static GLOBAL: std::sync::Mutex<u32> = std::sync::Mutex::new(0);

pub fn hidden_write_global(v: u32) {
    *GLOBAL.lock().unwrap() = v;
}

use std::sync::OnceLock;

pub static CONFIG: OnceLock<u32> = OnceLock::new();

pub fn config_value() -> u32 {
    *CONFIG.get_or_init(|| 7)
}

#![allow(clippy::missing_const_for_fn)]

pub const fn alloc_string() -> &'static str {
    "hello"
}

pub const fn alloc_array() -> [u8; 3] {
    [1, 2, 3]
}

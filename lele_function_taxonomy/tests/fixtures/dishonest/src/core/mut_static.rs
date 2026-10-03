static mut MUTABLE: u32 = 0;

pub fn write_mutable() {
    unsafe {
        MUTABLE = 1;
    }
}

pub fn read_mutable() -> u32 {
    unsafe { MUTABLE }
}

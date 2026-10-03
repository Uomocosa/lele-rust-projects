use std::cell::RefCell;

thread_local! {
    static SCRATCH: RefCell<u32> = const { RefCell::new(0) };
}

pub fn scratch_value() -> u32 {
    SCRATCH.with(|cell| *cell.borrow())
}

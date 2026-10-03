use std::cell::Cell;

pub struct Counter {
    value: Cell<u64>,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            value: Cell::new(0),
        }
    }

    pub fn bump(&self) -> u64 {
        let next = self.value.get() + 1;
        self.value.set(next);
        next
    }
}

pub fn use_counter() -> u64 {
    let counter = Counter::new();
    counter.bump()
}

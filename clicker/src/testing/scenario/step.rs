#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Click { player: u64, times: u32 },
    Partition { first: u64, second: u64 },
    Heal { first: u64, second: u64 },
    ExpectGlobal { total: i32 },
}

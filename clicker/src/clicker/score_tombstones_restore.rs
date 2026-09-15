use crate::clicker;

pub fn restore(stones: &mut clicker::ScoreTombstones, logical: u64) -> Option<i32> {
    stones.remove(&logical)
}
// no test_usage necessary

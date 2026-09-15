use crate::clicker;

pub fn keep(stones: &mut clicker::ScoreTombstones, logical: u64, count: i32) {
    let entry = stones.entry(logical).or_default();
    if count > *entry {
        *entry = count;
    }
}
// no test_usage necessary

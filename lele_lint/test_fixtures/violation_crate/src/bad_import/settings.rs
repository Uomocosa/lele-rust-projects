use crate::bad_getter::Score; // VIOLATION: direct type import, should be crate::bad_getter then bad_getter::Score

pub struct Settings {
    pub high_score: Score,
}

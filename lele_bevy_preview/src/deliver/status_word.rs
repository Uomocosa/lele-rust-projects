use crate::deliver::basic::enums::Status;

#[must_use]
pub const fn status_word(status: Status) -> &'static str {
    match status {
        Status::FirstRun => "first run",
        Status::New => "new",
        Status::Changed => "changed",
        Status::Same => "same",
    }
}
// no test_usage necessary

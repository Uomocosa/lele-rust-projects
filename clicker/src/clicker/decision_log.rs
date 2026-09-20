use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::SystemTime;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

static BUFFER: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

#[derive(Resource, Deref, DerefMut, Default)]
pub struct DecisionLog(pub Option<std::fs::File>);

impl DecisionLog {
    pub const MAX_BUFFERED: usize = 8192;

    pub fn record(text: &str) {
        let line = stamp(text);
        lock().push_back(line);
        trim();
    }

    pub(crate) fn take() -> Vec<String> {
        let taken: VecDeque<String> = std::mem::take(&mut *lock());
        taken.into_iter().collect()
    }
}

// needed helper: single-statement guard borrow keeps lock scopes tight
fn lock() -> std::sync::MutexGuard<'static, VecDeque<String>> {
    BUFFER
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

// needed helper: drops oldest buffered lines past the cap
fn trim() {
    while lock().len() > DecisionLog::MAX_BUFFERED {
        lock().pop_front();
    }
}

// needed helper: prefixes a decision line with wall-clock millis
fn stamp(text: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default();
    format!("{millis} {text}")
}

#[cfg(test)]
mod tests {
    use super::DecisionLog;

    #[test]
    fn test_usage() {
        assert!(DecisionLog::default().is_none());
        let log = DecisionLog(tempfile::tempfile().ok());
        assert!(log.is_some());
    }
}

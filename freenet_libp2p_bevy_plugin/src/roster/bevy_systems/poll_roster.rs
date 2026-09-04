use bevy::prelude::*;

use crate::roster::Roster;

pub fn poll_roster(_roster: ResMut<Roster>) {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert!(true);
    }
}
// no test_usage necessary

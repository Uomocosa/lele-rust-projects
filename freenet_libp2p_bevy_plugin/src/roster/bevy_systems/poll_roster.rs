use bevy::prelude::*;

use crate::roster::Roster;

pub const fn poll_roster(_roster: ResMut<Roster>) {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(poll_roster);
    }
}
// no test_usage necessary

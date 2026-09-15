use crate::discovery;

pub fn refresh_addrs(roster: &mut discovery::Roster, addrs: Vec<String>) {
    roster.addrs = addrs;
}
// no test_usage necessary

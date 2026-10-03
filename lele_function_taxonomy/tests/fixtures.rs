mod common;

use common::{assert_or_bless, fixture_dirs, run_fixture};

#[test]
fn test_usage() {
    let dirs = fixture_dirs();
    assert!(!dirs.is_empty(), "no fixtures found");
    for dir in dirs {
        let run = run_fixture(&dir);
        assert_or_bless(&run);
    }
}

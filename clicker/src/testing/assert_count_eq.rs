use bevy::prelude::*;

use crate::testing;

/// # Errors
/// Returns an error describing the mismatch when the count differs from `want`.
pub fn assert_count_eq(app: &mut App, owner: u64, want: i32) -> Result<(), String> {
    let got = testing::get_count(app, owner);
    if got == want {
        Ok(())
    } else {
        Err(format!("owner={owner} count={got} want={want}"))
    }
}

#[cfg(test)]
mod tests {
    use super::assert_count_eq;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut app = testing::fixture(1, "alpha");
        app.update();
        assert!(assert_count_eq(&mut app, 1, 0).is_ok());
        assert!(assert_count_eq(&mut app, 1, 5).is_err());
    }
}

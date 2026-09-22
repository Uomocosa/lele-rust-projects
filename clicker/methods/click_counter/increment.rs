use crate::clicker;

pub fn increment(counter: &mut clicker::ClickCounter) {
    **counter = (**counter).wrapping_add(1);
}

#[cfg(test)]
mod tests {
    use super::increment;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut counter = clicker::ClickCounter::default();
        increment(&mut counter);
        assert_eq!(*counter, 1);
    }
}

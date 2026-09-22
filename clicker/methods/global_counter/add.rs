use crate::clicker;

pub fn add(counter: &mut clicker::GlobalCounter, delta: i32) {
    **counter = (**counter).saturating_add(delta);
}

#[cfg(test)]
mod tests {
    use super::add;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut counter = clicker::GlobalCounter::default();
        add(&mut counter, 5);
        assert_eq!(*counter, 5);
        add(&mut counter, i32::MAX);
        assert_eq!(*counter, i32::MAX);
    }
}

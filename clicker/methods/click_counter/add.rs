use crate::clicker;

pub fn add(counter: &mut clicker::ClickCounter, delta: i32) {
    **counter = (**counter).saturating_add(delta);
}

#[cfg(test)]
mod tests {
    use super::add;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut counter = clicker::ClickCounter::default();
        add(&mut counter, 5);
        assert_eq!(*counter, 5);
    }
}

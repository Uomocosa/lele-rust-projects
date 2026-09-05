use super::click_counter::ClickCounter;

pub fn decrement(counter: &mut ClickCounter) {
    **counter = (**counter).wrapping_sub(1);
}

#[cfg(test)]
mod tests {
    use super::decrement;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut counter = clicker::ClickCounter(4);
        decrement(&mut counter);
        assert_eq!(*counter, 3);
    }
}

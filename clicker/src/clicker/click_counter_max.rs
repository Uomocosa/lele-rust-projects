use super::click_counter::ClickCounter;

pub fn max(counter: &mut ClickCounter, value: i32) {
    if value > **counter {
        **counter = value;
    }
}

#[cfg(test)]
mod tests {
    use super::max;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut counter = clicker::ClickCounter(4);
        max(&mut counter, 7);
        assert_eq!(*counter, 7);
        max(&mut counter, 2);
        assert_eq!(*counter, 7);
    }
}

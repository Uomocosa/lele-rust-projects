use crate::clicker::component::ClickCounter;

pub fn increment(counter: &mut ClickCounter) {
    counter.count += 1;
}

#[cfg(test)]
mod tests {
    use crate::clicker::component::ClickCounter;

    #[test]
    fn test_usage() {
        let mut counter = ClickCounter { count: 0 };
        super::increment(&mut counter);
        assert_eq!(counter.count, 1);
    }
}

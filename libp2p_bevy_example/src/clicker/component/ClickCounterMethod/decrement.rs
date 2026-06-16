use crate::clicker::component::ClickCounter;

pub fn decrement(counter: &mut ClickCounter) {
    counter.count = counter.count.saturating_sub(1);
}

#[cfg(test)]
mod tests {
    use crate::clicker::component::ClickCounter;

    #[test]
    fn test_usage() {
        let mut counter = ClickCounter { count: 5 };
        super::decrement(&mut counter);
        assert_eq!(counter.count, 4);
    }
}

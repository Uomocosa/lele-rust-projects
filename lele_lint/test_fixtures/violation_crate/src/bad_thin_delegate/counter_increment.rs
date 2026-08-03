use super::counter::Counter;

pub fn increment(counter: &mut Counter) {
    counter.count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;          // VIOLATION: super::increment, not super::*

    #[test]
    fn test_usage() {
        let mut c = super::counter::Counter { count: 0 };
        increment(&mut c);
        assert_eq!(c.count, 1);
    }
}

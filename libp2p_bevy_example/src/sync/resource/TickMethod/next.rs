use crate::sync::resource::Tick;

#[allow(clippy::should_implement_trait)]
pub fn next(tick: &mut Tick) -> u64 {
    let current = tick.current;
    tick.current = tick.current.wrapping_add(1);
    current
}

#[cfg(test)]
mod tests {
    use crate::sync::resource::Tick;

    #[test]
    fn test_usage() {
        let mut tick = Tick::default();
        assert_eq!(tick.current, 0);
        let prev = tick.next();
        assert_eq!(prev, 0);
        assert_eq!(tick.current, 1);
    }
}

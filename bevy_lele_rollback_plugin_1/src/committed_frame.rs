pub struct CommittedFrame<State> {
    pub tick: u64,
    pub state: State,
    pub hash: u64,
}

#[cfg(test)]
mod tests {
    use super::CommittedFrame;

    #[test]
    fn test_usage() {
        let frame = CommittedFrame::<u64> {
            tick: 3,
            state: 5,
            hash: 9,
        };
        assert_eq!(frame.tick, 3);
        assert_eq!(frame.state, 5);
        assert_eq!(frame.hash, 9);
    }
}

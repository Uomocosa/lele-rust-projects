use derive_more::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deref)]
pub struct PlayerId(pub u64);

#[rustfmt::skip]
impl PlayerId {
    pub fn as_u64(self) -> u64 { *self }
}

#[cfg(test)]
mod tests {
    use crate::player_id::PlayerId;

    #[test]
    fn test_usage() {
        let id = PlayerId(7);
        assert_eq!(*id, 7);
        assert_eq!(id.as_u64(), 7);
    }
}

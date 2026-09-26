use crate::PlayerId;

pub fn as_u64(id: PlayerId) -> u64 {
    *id
}

#[cfg(test)]
mod tests {
    use super::as_u64;
    use crate::PlayerId;

    #[test]
    fn test_usage() {
        assert_eq!(as_u64(PlayerId(7)), 7);
    }
}

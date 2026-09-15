#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomEntry {
    pub name: String,
    pub updated_at: u64,
}

#[cfg(test)]
mod tests {
    use super::RoomEntry;

    #[test]
    fn test_usage() {
        let entry = RoomEntry {
            name: "room-20250101-120000".to_string(),
            updated_at: 7,
        };
        assert_eq!(entry.name, "room-20250101-120000");
        assert_eq!(entry.updated_at, 7);
        assert_ne!(
            entry,
            RoomEntry {
                name: "other".to_string(),
                updated_at: 7,
            }
        );
    }
}

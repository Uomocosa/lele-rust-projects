use bevy::prelude::Resource;

use super::room_entry::RoomEntry;

#[derive(Resource, Debug, Default, Clone)]
pub struct RoomList {
    pub entries: Vec<RoomEntry>,
    pub revision: u64,
}

#[cfg(test)]
mod tests {
    use super::RoomList;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut list = RoomList::default();
        assert_eq!(list.entries.len(), 0);
        assert_eq!(list.revision, 0);
        list.entries.push(discovery::RoomEntry {
            name: "room-a".to_string(),
            updated_at: 3,
        });
        list.revision = list.revision.wrapping_add(1);
        assert_eq!(list.entries.len(), 1);
        assert_eq!(list.revision, 1);
    }
}

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Photo,
    Video,
}

#[derive(Debug, Clone, Copy)]
pub struct MediaItem<'a> {
    pub kind: MediaKind,
    pub path: &'a Path,
}

#[cfg(test)]
mod tests {
    use super::{MediaItem, MediaKind};
    use std::path::Path;

    #[test]
    fn test_usage() {
        let item = MediaItem {
            kind: MediaKind::Photo,
            path: Path::new("a.png"),
        };
        assert_eq!(item.kind, MediaKind::Photo);
        assert_ne!(item.kind, MediaKind::Video);
        assert_eq!(item.path, Path::new("a.png"));
    }
}

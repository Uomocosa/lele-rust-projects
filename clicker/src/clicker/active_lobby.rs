use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref)]
pub struct ActiveLobby(pub String);

#[cfg(test)]
mod tests {
    use super::ActiveLobby;

    #[test]
    fn test_usage() {
        let lobby = ActiveLobby("alpha".to_string());
        assert_eq!(&**lobby, "alpha");
        assert_eq!(ActiveLobby::default(), ActiveLobby(String::new()));
    }
}

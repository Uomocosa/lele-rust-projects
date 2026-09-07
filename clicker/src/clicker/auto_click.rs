use bevy::prelude::Resource;
use derive_more::Deref;

#[derive(Resource, Debug, Default, Clone, Copy, Deref)]
pub struct AutoClick(pub bool);

#[cfg(test)]
mod tests {
    use super::AutoClick;

    #[test]
    fn test_usage() {
        assert!(!*AutoClick::default());
        assert!(*AutoClick(true));
    }
}

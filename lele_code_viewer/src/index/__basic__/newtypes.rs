use derive_more::Deref;

#[derive(Debug, Clone, Deref)]
pub struct ItemGroup(pub Vec<usize>);

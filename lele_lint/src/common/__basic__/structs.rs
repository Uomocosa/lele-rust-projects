use std::collections::BTreeSet;

#[derive(Debug, Clone, Default)]
pub struct DeclaredType {
    pub methods: BTreeSet<String>,
    pub cfgs: Vec<String>,
}

pub(crate) struct CommentHit {
    pub line: usize,
    pub text: String,
    pub block: bool,
}

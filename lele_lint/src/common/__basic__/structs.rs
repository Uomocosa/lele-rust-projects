use std::collections::BTreeSet;

use crate::common;

#[derive(Debug, Clone, Default)]
pub struct DeclaredType {
    pub methods: BTreeSet<String>,
    pub cfgs: Vec<String>,
}

pub(crate) struct CommentHit {
    pub line: usize,
    pub kind: common::CommentKind,
}

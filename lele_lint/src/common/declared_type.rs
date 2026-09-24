use std::collections::BTreeSet;

#[derive(Debug, Clone, Default)]
pub struct DeclaredType {
    pub methods: BTreeSet<String>,
    pub cfgs: Vec<String>,
}

// no test_usage necessary

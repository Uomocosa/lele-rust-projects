use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateRecord {
    pub id: String,
    pub group: String,
    pub screen: String,
    pub path: Vec<String>,
    pub location: String,
    pub png: String,
    pub fingerprint: String,
    pub pixel_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeRecord {
    pub from: String,
    pub to: String,
    pub action: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capture {
    pub states: Vec<StateRecord>,
    pub edges: Vec<EdgeRecord>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub crate_name: String,
    pub driver: String,
    pub states: Vec<StateRecord>,
    pub edges: Vec<EdgeRecord>,
    pub notes: Vec<String>,
}

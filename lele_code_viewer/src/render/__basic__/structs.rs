use crate::render::basic::enums;

pub struct LinkConfig {
    pub prefix: String,
    pub assets: String,
    pub html: bool,
    pub nav: Option<Nav>,
}

pub struct Nav {
    pub current: String,
    pub projects: Vec<NavProject>,
    pub view: enums::ViewKind,
}

pub struct NavProject {
    pub id: String,
    pub name: String,
}

// no test_usage necessary

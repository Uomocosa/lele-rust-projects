// no test_usage necessary
mod constructor_no_skip;
mod domain_import;
mod method_visibility;
mod no_cross_domain_reexport;
mod no_positional;
mod no_trivial_accessors;
mod snake_case_files;
mod test_inline;
mod test_usage;
mod thin_delegates;

use crate::checker::Checker;
use crate::config::Config;

pub fn build_checkers(config: &Config) -> Vec<Box<dyn Checker>> {
    let mut checkers: Vec<Box<dyn Checker>> = Vec::new();
    snake_case_files::register(&mut checkers, config);
    method_visibility::register(&mut checkers, config);
    no_cross_domain_reexport::register(&mut checkers, config);
    test_inline::register(&mut checkers, config);
    test_usage::register(&mut checkers, config);
    no_positional::register(&mut checkers, config);
    no_trivial_accessors::register(&mut checkers, config);
    domain_import::register(&mut checkers, config);
    thin_delegates::register(&mut checkers, config);
    constructor_no_skip::register(&mut checkers, config);
    checkers
}

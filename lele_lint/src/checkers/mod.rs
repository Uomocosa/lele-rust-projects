// no test_usage necessary
mod atomic_file;
mod atomic_file_register;
mod constructor_no_skip;
mod constructor_no_skip_register;
mod domain_import;
mod domain_import_register;
mod helper_count;
mod helper_count_register;
mod method_visibility;
mod method_visibility_register;
mod no_cross_domain_reexport;
mod no_cross_domain_reexport_register;
mod no_positional;
mod no_positional_register;
mod no_trivial_accessors;
mod no_trivial_accessors_register;
mod snake_case_files;
mod snake_case_files_register;
mod test_inline;
mod test_inline_register;
mod test_usage;
mod test_usage_register;
mod thin_delegates;
mod thin_delegates_register;

use crate::checker::Checker;
use crate::config::Config;

pub fn build_checkers(config: &Config) -> Vec<Box<dyn Checker>> {
    let mut checkers: Vec<Box<dyn Checker>> = Vec::new();
    atomic_file::AtomicFile::register(&mut checkers, config);
    snake_case_files::SnakeCaseFiles::register(&mut checkers, config);
    method_visibility::MethodVisibility::register(&mut checkers, config);
    no_cross_domain_reexport::NoCrossDomainReexport::register(&mut checkers, config);
    test_inline::TestInline::register(&mut checkers, config);
    test_usage::TestUsage::register(&mut checkers, config);
    no_positional::NoPositional::register(&mut checkers, config);
    no_trivial_accessors::NoTrivialAccessors::register(&mut checkers, config);
    domain_import::DomainImport::register(&mut checkers, config);
    thin_delegates::ThinDelegates::register(&mut checkers, config);
    constructor_no_skip::ConstructorNoSkip::register(&mut checkers, config);
    helper_count::HelperCount::register(&mut checkers, config);
    checkers
}

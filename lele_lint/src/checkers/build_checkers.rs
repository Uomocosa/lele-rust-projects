use super::atomic_file;
use super::constructor_no_skip;
use super::domain_import;
use super::helper_count;
use super::method_file_co_location;
use super::method_visibility;
use super::mod_rs_purity;
use super::no_crate_paths;
use super::no_cross_domain_reexport;
use super::no_positional;
use super::no_trivial_accessors;
use super::single_caller_type;
use super::single_field_newtype;
use super::snake_case_files;
use super::test_inline;
use super::test_usage;
use super::thin_delegates;
use crate::checker;
use crate::config;

pub fn build_checkers(config: &config::Config) -> Vec<Box<dyn checker::Checker>> {
    let mut checkers: Vec<Box<dyn checker::Checker>> = Vec::new();
    atomic_file::AtomicFile::register(&mut checkers, config);
    snake_case_files::SnakeCaseFiles::register(&mut checkers, config);
    method_file_co_location::MethodFileCoLocation::register(&mut checkers, config);
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
    single_field_newtype::SingleFieldNewtype::register(&mut checkers, config);
    mod_rs_purity::ModRsPurity::register(&mut checkers, config);
    no_crate_paths::NoCratePaths::register(&mut checkers, config);
    single_caller_type::SingleCallerType::register(&mut checkers, config);
    checkers
}

// no test_usage necessary

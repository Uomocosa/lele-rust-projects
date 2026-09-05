use super::atomic_delegates;
use super::atomic_file;
use super::clippy_config_cargo;
use super::clippy_config_clippy;
use super::constants_placement;
use super::constructor_no_skip;
use super::domain_import;
use super::helper_count;
use super::method_file_co_location;
use super::method_visibility;
use super::mod_rs_purity;
use super::no_allow_attributes;
use super::no_collection_newtype;
use super::no_crate_paths;
use super::no_cross_domain_reexport;
use super::no_positional;
use super::no_stuttered_path;
use super::no_stuttered_type;
use super::no_trivial_accessors;
use super::root_reexport;
use super::single_caller_type;
use super::single_field_newtype;
use super::snake_case_files;
use super::test_inline;
use super::test_usage;
use crate::Checker;
use crate::Config;

pub fn build_checkers(config: &Config) -> Vec<Box<dyn Checker>> {
    let mut checkers: Vec<Box<dyn Checker>> = Vec::new();
    atomic_file::AtomicFile::register(&mut checkers, config);
    snake_case_files::SnakeCaseFiles::register(&mut checkers, config);
    method_file_co_location::MethodFileCoLocation::register(&mut checkers, config);
    method_visibility::MethodVisibility::register(&mut checkers, config);
    no_cross_domain_reexport::NoCrossDomainReexport::register(&mut checkers, config);
    test_inline::TestInline::register(&mut checkers, config);
    test_usage::TestUsage::register(&mut checkers, config);
    no_positional::NoPositional::register(&mut checkers, config);
    no_stuttered_path::NoStutteredPath::register(&mut checkers, config);
    no_stuttered_type::NoStutteredType::register(&mut checkers, config);
    no_trivial_accessors::NoTrivialAccessors::register(&mut checkers, config);
    root_reexport::RootReexport::register(&mut checkers, config);
    domain_import::DomainImport::register(&mut checkers, config);
    atomic_delegates::AtomicDelegates::register(&mut checkers, config);
    constructor_no_skip::ConstructorNoSkip::register(&mut checkers, config);
    constants_placement::ConstantsPlacement::register(&mut checkers, config);
    helper_count::HelperCount::register(&mut checkers, config);
    single_field_newtype::SingleFieldNewtype::register(&mut checkers, config);
    mod_rs_purity::ModRsPurity::register(&mut checkers, config);
    no_allow_attributes::NoAllowAttributes::register(&mut checkers, config);
    no_crate_paths::NoCratePaths::register(&mut checkers, config);
    no_collection_newtype::NoCollectionNewtype::register(&mut checkers, config);
    single_caller_type::SingleCallerType::register(&mut checkers, config);
    clippy_config_cargo::ClippyConfigCargo::register(&mut checkers, config);
    clippy_config_clippy::ClippyConfigClippy::register(&mut checkers, config);
    checkers
}

// no test_usage necessary

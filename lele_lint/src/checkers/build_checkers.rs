use crate::checkers;
use crate::Checker;

pub fn build_checkers() -> Vec<Box<dyn Checker>> {
    let mut checkers: Vec<Box<dyn Checker>> = Vec::new();
    checkers::atomic_file::AtomicFile::register(&mut checkers);
    checkers::snake_case_files::SnakeCaseFiles::register(&mut checkers);
    checkers::method_file_co_location::MethodFileCoLocation::register(&mut checkers);
    checkers::method_visibility::MethodVisibility::register(&mut checkers);
    checkers::no_cross_domain_reexport::NoCrossDomainReexport::register(&mut checkers);
    checkers::no_dunder_tests::NoDunderTests::register(&mut checkers);
    checkers::test_inline::TestInline::register(&mut checkers);
    checkers::test_usage::TestUsage::register(&mut checkers);
    checkers::no_positional::NoPositional::register(&mut checkers);
    checkers::no_stuttered_path::NoStutteredPath::register(&mut checkers);
    checkers::no_stuttered_type::NoStutteredType::register(&mut checkers);
    checkers::no_super_imports::NoSuperImports::register(&mut checkers);
    checkers::no_trivial_accessors::NoTrivialAccessors::register(&mut checkers);
    checkers::root_reexport::RootReexport::register(&mut checkers);
    checkers::domain_import::DomainImport::register(&mut checkers);
    checkers::atomic_delegates::AtomicDelegates::register(&mut checkers);
    checkers::constructor_no_skip::ConstructorNoSkip::register(&mut checkers);
    checkers::constants_placement::ConstantsPlacement::register(&mut checkers);
    checkers::container_placement::ContainerPlacement::register(&mut checkers);
    checkers::helper_count::HelperCount::register(&mut checkers);
    checkers::single_field_newtype::SingleFieldNewtype::register(&mut checkers);
    checkers::mod_rs_purity::ModRsPurity::register(&mut checkers);
    checkers::no_allow_attributes::NoAllowAttributes::register(&mut checkers);
    checkers::no_crate_paths::NoCratePaths::register(&mut checkers);
    checkers::no_collection_newtype::NoCollectionNewtype::register(&mut checkers);
    checkers::single_caller_type::SingleCallerType::register(&mut checkers);
    checkers::clippy_config_cargo::ClippyConfigCargo::register(&mut checkers);
    checkers::clippy_config_clippy::ClippyConfigClippy::register(&mut checkers);
    checkers::methods_layout::MethodsLayout::register(&mut checkers);
    checkers::no_comments::NoComments::register(&mut checkers);
    checkers::delegate_macro::DelegateMacro::register(&mut checkers);
    checkers
}

// no test_usage necessary

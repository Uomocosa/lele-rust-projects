// no test_usage necessary
use super::domain_import::DomainImport;

pub(crate) fn name(_self: &DomainImport) -> &'static str {
    "domain_import"
}

pub(crate) fn code(_self: &DomainImport) -> &'static str {
    "E011"
}

// no test_usage necessary
use super::no_cross_domain_reexport::NoCrossDomainReexport;

pub(crate) fn name(_self: &NoCrossDomainReexport) -> &'static str {
    "no_cross_domain_reexport"
}

pub(crate) fn code(_self: &NoCrossDomainReexport) -> &'static str {
    "E004"
}

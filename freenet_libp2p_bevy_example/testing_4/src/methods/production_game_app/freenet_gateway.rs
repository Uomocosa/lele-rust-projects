use crate::structs;

pub fn freenet_gateway(this: &structs::ProductionGameApp) -> String {
    this.gateway.clone()
}
// no test_usage necessary — exercised by the local e2e test

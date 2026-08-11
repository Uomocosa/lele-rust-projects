use crate::structs::production_game_app::ProductionGameApp;

pub fn freenet_gateway(this: &ProductionGameApp) -> String {
    this.gateway.clone()
}
// no test_usage necessary — exercised by the local e2e test

use freenet_stdlib::prelude::ContractKey;

#[derive(Debug)]
pub enum ClickerEvent {
    Init {
        contract_key: ContractKey,
        count: u64,
    },
    Notification {
        count: u64,
    },
    UpdateResponse {
        count: u64,
    },
}

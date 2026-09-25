use freenet_stdlib::prelude::ContractKey;

use super::catalogue::RoomCatalogue;
use crate::discovery;

pub struct IndexClient {
    pub(crate) client: discovery::link::Client,
    pub contract_key: ContractKey,
    pub(crate) slots: RoomCatalogue,
}
// no test_usage necessary

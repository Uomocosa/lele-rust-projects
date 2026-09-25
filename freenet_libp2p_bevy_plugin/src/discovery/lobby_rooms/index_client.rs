use freenet_stdlib::prelude::ContractKey;

use crate::discovery;

pub struct IndexClient {
    pub(crate) client: discovery::link::Client,
    pub contract_key: ContractKey,
}
// no test_usage necessary

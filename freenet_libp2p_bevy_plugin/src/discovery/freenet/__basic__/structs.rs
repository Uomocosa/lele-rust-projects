use freenet_stdlib::prelude::ContractKey;

use crate::discovery;

pub struct DirectoryClient {
    pub client: discovery::freenet::Client,
    pub key: ContractKey,
    pub directory: discovery::Directory,
}

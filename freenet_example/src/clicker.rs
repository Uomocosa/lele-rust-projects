use std::sync::Arc;
use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::FreenetClient;

const TIMEOUT_SECS: u64 = 10;

/// The role of a clicker client.
pub enum Role {
    /// Deploys the contract if not found, subscribes, and increments.
    Publish,
    /// Subscribes to an existing contract and increments.
    Subscribe,
}

/// High-level clicker client.
///
/// Wraps a [`FreenetClient`] with the clicker contract lifecycle:
/// deploy-or-subscribe, state tracking, and increment ticks.
pub struct ClickerClient {
    client: FreenetClient,
    contract_key: ContractKey,
    count: u64,
}

impl ClickerClient {
    /// Connect to a freenet node and initialize for the given role.
    ///
    /// For [`Role::Publish`]: tries to find the contract first; if not found,
    /// deploys it with initial state `0`.
    ///
    /// For [`Role::Subscribe`]: retries until the contract is found.
    pub async fn connect(
        host: &str,
        port: u16,
        contract_wasm: &[u8],
        role: Role,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut client = FreenetClient::connect(host, port).await?;

        let contract_code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
        let params = Parameters::from(Vec::new());
        let wrapped = WrappedContract::new(contract_code, params);
        let contract_key = wrapped.key;
        let instance_id = *contract_key.id();

        let (key, count) = match role {
            Role::Publish => {
                let result = recv_after_get(&mut client, instance_id).await;
                match result {
                    Ok(r) => r,
                    Err(_) => {
                        let put_req = ContractRequest::Put {
                            contract: ContractContainer::from(ContractWasmAPIVersion::V1(wrapped)),
                            state: WrappedState::new(bincode::serialize(&0u64)?),
                            related_contracts: RelatedContracts::default(),
                            subscribe: true,
                            blocking_subscribe: false,
                        };
                        client.send(ClientRequest::ContractOp(put_req)).await?;
                        match client.recv_response().await? {
                            HostResponse::ContractResponse(
                                ContractResponse::PutResponse { key }
                                | ContractResponse::SubscribeResponse { key, .. }
                                | ContractResponse::UpdateResponse { key, .. },
                            ) => {
                                info!(target: "freenet_example", key = %key, "contract deployed");
                            }
                            other => {
                                return Err(format!("unexpected response to put: {other:?}").into());
                            }
                        }
                        recv_after_get(&mut client, instance_id).await?
                    }
                }
            }
            Role::Subscribe => loop {
                match recv_after_get(&mut client, instance_id).await {
                    Ok(r) => break r,
                    Err(_) => {
                        info!(
                            target: "freenet_example",
                            %instance_id,
                            "contract not found, retrying in 1s"
                        );
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            },
        };

        Ok(Self {
            client,
            contract_key: key,
            count,
        })
    }

    /// The contract key this client is operating on.
    pub fn contract_key(&self) -> ContractKey {
        self.contract_key
    }

    /// The last-known count from local tracking or a `tick` call.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Read the current state from the node via explicit GET.
    pub async fn state(&mut self) -> Result<u64, Box<dyn std::error::Error>> {
        let get_req = ContractRequest::Get {
            key: *self.contract_key.id(),
            return_contract_code: false,
            subscribe: false,
            blocking_subscribe: false,
        };
        self.client.send(ClientRequest::ContractOp(get_req)).await?;
        match self.client.recv_response().await? {
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                let count = bincode::deserialize(state.as_ref()).unwrap_or(0);
                self.count = count;
                Ok(count)
            }
            other => Err(format!("unexpected GET response: {other:?}").into()),
        }
    }

    /// Run one tick: drain pending notifications, increment the counter,
    /// send the update, and wait for confirmation.
    ///
    /// Returns the new count after the tick.
    pub async fn tick(&mut self) -> Result<u64, Box<dyn std::error::Error>> {
        while let Some(result) = self.client.recv_timeout(Duration::from_millis(10)).await {
            if let HostResponse::ContractResponse(ContractResponse::UpdateNotification {
                update,
                ..
            }) = result?
            {
                self.count = match &update {
                    UpdateData::State(s) => bincode::deserialize(s.as_ref()).unwrap_or(0),
                    UpdateData::Delta(d) => bincode::deserialize(d.as_ref()).unwrap_or(0),
                    _ => 0,
                };
            }
        }

        self.count = self.count.wrapping_add(1);
        let new_state = State::from(bincode::serialize(&self.count)?);
        let update_req = ContractRequest::Update {
            key: self.contract_key,
            data: UpdateData::State(new_state),
        };
        self.client
            .send(ClientRequest::ContractOp(update_req))
            .await?;

        match self.client.recv_response().await? {
            HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => {}
            other => return Err(format!("unexpected UPDATE response: {other:?}").into()),
        }

        Ok(self.count)
    }
}

/// Send GET + subscribe and return the current `(ContractKey, count)`.
async fn recv_after_get(
    client: &mut FreenetClient,
    instance_id: ContractInstanceId,
) -> Result<(ContractKey, u64), Box<dyn std::error::Error>> {
    let get_req = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(ClientRequest::ContractOp(get_req)).await?;
    match recv_response(client).await? {
        HostResponse::ContractResponse(ContractResponse::GetResponse { key, state, .. }) => {
            let count = bincode::deserialize(state.as_ref()).unwrap_or(0);
            Ok((key, count))
        }
        HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {
            Err("contract not found".into())
        }
        other => Err(format!("unexpected response: {other:?}").into()),
    }
}

async fn recv_response(
    client: &mut FreenetClient,
) -> Result<HostResponse, Box<dyn std::error::Error>> {
    match client
        .recv_response_timeout(Duration::from_secs(TIMEOUT_SECS))
        .await
    {
        Some(result) => result,
        None => Err(format!("no response from node within {TIMEOUT_SECS}s").into()),
    }
}

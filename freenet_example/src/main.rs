use std::sync::Arc;
use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use freenet_example::FreenetClient;

enum Role {
    Publish,
    Subscribe,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let role = parse_role();

    let contract_wasm =
        std::fs::read("contract/target/wasm32-unknown-unknown/release/clicker_contract.wasm")?;

    info!(target: "freenet_example", size = contract_wasm.len(), "loaded contract wasm");

    let node_host = std::env::var("FREENET_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let node_port: u16 = std::env::var("FREENET_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7509);

    let mut client = FreenetClient::connect(&node_host, node_port).await?;
    info!(target: "freenet_example", "connected to freenet node");

    let contract_code = Arc::new(ContractCode::from(contract_wasm));
    let params = Parameters::from(Vec::new());
    let wrapped = WrappedContract::new(contract_code, params);
    let contract_key = wrapped.key;

    match role {
        Role::Publish => {
            let initial_state = WrappedState::new(bincode::serialize(&0u64)?);
            let related = RelatedContracts::default();

            let put_req = ContractRequest::Put {
                contract: ContractContainer::from(ContractWasmAPIVersion::V1(wrapped)),
                state: initial_state,
                related_contracts: related,
                subscribe: true,
                blocking_subscribe: false,
            };
            client.send(ClientRequest::ContractOp(put_req)).await?;

            let key = match recv_with_timeout(&mut client).await? {
                HostResponse::ContractResponse(response) => match response {
                    ContractResponse::PutResponse { key }
                    | ContractResponse::SubscribeResponse { key, .. }
                    | ContractResponse::UpdateResponse { key, .. } => key,
                    other => {
                        return Err(format!("unexpected response to put: {other:?}").into());
                    }
                },
                other => {
                    return Err(format!("unexpected response: {other:?}").into());
                }
            };

            info!(target: "freenet_example", key = %key, "contract deployed and subscribed");
            run_increment_loop(&mut client, key, 0).await?;
        }
        Role::Subscribe => {
            let instance_id = *contract_key.id();
            loop {
                let get_req = ContractRequest::Get {
                    key: instance_id,
                    return_contract_code: false,
                    subscribe: true,
                    blocking_subscribe: false,
                };
                client.send(ClientRequest::ContractOp(get_req)).await?;

                match recv_with_timeout(&mut client).await? {
                    HostResponse::ContractResponse(ContractResponse::GetResponse {
                        key,
                        state,
                        ..
                    }) => {
                        let initial_count: u64 = bincode::deserialize(state.as_ref()).unwrap_or(0);
                        info!(
                            target: "freenet_example",
                            key = %key,
                            count = initial_count,
                            "subscribed and got current state"
                        );
                        run_increment_loop(&mut client, key, initial_count).await?;
                        break;
                    }
                    HostResponse::ContractResponse(ContractResponse::NotFound { instance_id }) => {
                        info!(
                            target: "freenet_example",
                            %instance_id,
                            "contract not found yet, retrying in 1s"
                        );
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                    HostResponse::ContractResponse(other) => {
                        return Err(format!("unexpected response to get: {other:?}").into());
                    }
                    other => {
                        return Err(format!("unexpected response: {other:?}").into());
                    }
                }
            }
        }
    }

    Ok(())
}

fn parse_role() -> Role {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--role" {
            match args.next().as_deref() {
                Some("subscribe") => return Role::Subscribe,
                Some("publish") => return Role::Publish,
                Some(other) => {
                    eprintln!("unknown role: {other}. Use publish or subscribe");
                    std::process::exit(1);
                }
                None => {
                    eprintln!("--role requires an argument: publish or subscribe");
                    std::process::exit(1);
                }
            }
        }
    }
    Role::Publish
}

async fn recv_with_timeout(
    client: &mut FreenetClient,
) -> Result<HostResponse, Box<dyn std::error::Error>> {
    const TIMEOUT_SECS: u64 = 10;
    match tokio::time::timeout(Duration::from_secs(TIMEOUT_SECS), client.recv()).await {
        Ok(result) => result,
        Err(_) => Err(format!("no response from node within {TIMEOUT_SECS}s").into()),
    }
}

async fn run_increment_loop(
    client: &mut FreenetClient,
    contract_key: ContractKey,
    mut count: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        count = count.wrapping_add(1);
        let new_state = State::from(bincode::serialize(&count)?);
        let update_req = ContractRequest::Update {
            key: contract_key,
            data: UpdateData::State(new_state),
        };
        client.send(ClientRequest::ContractOp(update_req)).await?;
        info!(target: "freenet_example", count, "sent increment");

        tokio::time::sleep(Duration::from_secs(1)).await;

        while let Some(result) = client.recv_timeout(Duration::from_millis(100)).await {
            match result? {
                HostResponse::ContractResponse(ContractResponse::UpdateNotification {
                    key,
                    update,
                }) => {
                    let updated_count: u64 = match &update {
                        UpdateData::State(s) => bincode::deserialize(s.as_ref()).unwrap_or(0),
                        UpdateData::Delta(d) => bincode::deserialize(d.as_ref()).unwrap_or(0),
                        _ => 0,
                    };
                    info!(
                        target: "freenet_example",
                        key = %key,
                        count = updated_count,
                        "received update notification"
                    );
                }
                HostResponse::ContractResponse(ContractResponse::UpdateResponse {
                    key, ..
                }) => {
                    info!(target: "freenet_example", key = %key, "update confirmed");
                }
                other => {
                    info!(target: "freenet_example", ?other, "unexpected message");
                }
            }
        }
    }
}

use freenet_example::GlobalCounterClient;
use freenet_example::Role;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = std::env::var("FREENET_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("FREENET_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7509);

    println!("connecting to Freenet node at {host}:{port}");

    let wasm = include_bytes!("../contract/global_counter_contract.wasm").to_vec();
    let mut global_counter =
        GlobalCounterClient::connect(&host, port, &wasm, Role::Publish).await?;
    println!("connected, count: {}", global_counter.count());

    let count = global_counter.tick().await?;
    println!("tick: count = {count}");

    let state = global_counter.state().await?;
    println!("state: {state}");
    Ok(())
}

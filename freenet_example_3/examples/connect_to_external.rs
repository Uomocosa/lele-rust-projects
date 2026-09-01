use freenet_example_3::ClickerClient;
use freenet_example_3::Role;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = std::env::var("FREENET_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("FREENET_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7509);

    println!("connecting to Freenet node at {host}:{port}");

    let wasm = include_bytes!("../contract/clicker_contract.wasm").to_vec();
    let mut clicker = ClickerClient::connect(&host, port, &wasm, Role::Publish).await?;
    println!("connected, count: {}", clicker.count());

    let count = clicker.tick().await?;
    println!("tick: count = {count}");

    let state = clicker.state().await?;
    println!("state: {state}");
    Ok(())
}

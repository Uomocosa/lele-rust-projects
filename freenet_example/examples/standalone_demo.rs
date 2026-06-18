use freenet_example::ClickerClient;
use freenet_example::Role;
use freenet_example::testing::TestNode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node = TestNode::start().await?;
    let wasm = include_bytes!("../contract/clicker_contract.wasm").to_vec();
    let mut clicker =
        ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Publish).await?;

    println!("counter deployed, initial count: {}", clicker.count());

    for i in 1..=3 {
        let count = clicker.tick().await?;
        println!("tick {i}: count = {count}");
    }

    let state = clicker.state().await?;
    println!("final state: {state}");
    Ok(())
}

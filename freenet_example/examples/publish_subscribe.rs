use freenet_example::ClickerClient;
use freenet_example::Role;
use freenet_example::testing::TestNode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node = TestNode::start().await?;
    let wasm = include_bytes!("../contract/clicker_contract.wasm").to_vec();

    let mut publisher =
        ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Publish).await?;
    println!("publisher connected, initial count: {}", publisher.count());

    for i in 1..=3 {
        let count = publisher.tick().await?;
        println!("publisher tick {i}: count = {count}");
    }

    let mut subscriber =
        ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Subscribe).await?;
    let sub_state = subscriber.state().await?;
    println!("subscriber state after sync: {sub_state}");

    let count = publisher.tick().await?;
    println!("publisher tick 4: count = {count}");

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let sub_state = subscriber.state().await?;
    println!("subscriber state after update: {sub_state}");

    Ok(())
}

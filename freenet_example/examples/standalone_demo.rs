use freenet_example::GlobalCounterClient;
use freenet_example::Role;
use freenet_example::testing::TestNode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node = TestNode::start().await?;
    let wasm = include_bytes!("../contract/global_counter_contract.wasm").to_vec();
    let mut global_counter =
        GlobalCounterClient::connect("127.0.0.1", node.port, &wasm, Role::Publish).await?;

    println!(
        "counter deployed, initial count: {}",
        global_counter.count()
    );

    for i in 1..=3 {
        let count = global_counter.tick().await?;
        println!("tick {i}: count = {count}");
    }

    let state = global_counter.state().await?;
    println!("final state: {state}");
    Ok(())
}

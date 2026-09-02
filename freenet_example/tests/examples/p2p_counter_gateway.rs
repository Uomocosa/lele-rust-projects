use freenet_example::testing::assert_example_contains;

#[test]
fn example_p2p_counter_gateway_exists() {
    assert_example_contains("examples/p2p_counter_gateway.rs", "run_gateway");
}

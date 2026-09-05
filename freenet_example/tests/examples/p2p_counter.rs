use freenet_example::testing::assert_example_contains;

#[test]
fn example_p2p_counter_exists() {
    assert!(assert_example_contains(
        "examples/p2p_counter.rs",
        "detect_public_ip"
    ));
}

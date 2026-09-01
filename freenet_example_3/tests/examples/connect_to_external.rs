use freenet_example_3::testing::assert_example_contains;

#[test]
fn example_connect_to_external_exists() {
    assert_example_contains("examples/connect_to_external.rs", "FREENET_HOST");
}

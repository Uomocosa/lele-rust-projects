# Manual multi-machine scenarios

These scenarios require routable IPs or a running Freenet node. They are not
covered by automated tests — use them as reference for manual testing.

## `connect_to_external.rs`

Connects to a running Freenet node via WebSocket. Set `FREENET_HOST` and
`FREENET_PORT` environment variables:

```bash
FREENET_HOST=127.0.0.1 FREENET_PORT=7509 cargo run --example connect_to_external
```

## `p2p_counter_gateway.rs`

Gateway/peer pattern for real P2P between two machines:

**Machine A (gateway):**
```bash
cargo run --example p2p_counter_gateway -- --gateway --public-address <YOUR_IP>
```

**Machine B (peer):**
```bash
cargo run --example p2p_counter_gateway -- --connect <A_IP>:<PORT>,<PUBKEY>
```

**Both gateways (machine B also acts as gateway):**
```bash
cargo run --example p2p_counter_gateway -- --gateway --public-address <IP_B> --connect <A_IP>:<PORT>,<PUBKEY>
```

The gateway prints a `GATEWAY_CONNECT=...` line with its IP, port, and pubkey.
Copy this to construct the peer's `--connect` argument.

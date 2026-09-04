# freenet_libp2p_example — Freenet discovery + libp2p gossip letters

Fixed lobby `params=lobby` shards per session (`ContractKey=Blake3(WASM||lobby)`). No Bevy, Tokio headless only.

## Game

N peers gossip random letters fast over libp2p (`/letters/1.0.0` request_response). Each frame is `seq, prev, next, author, sig` where `prev` must equal prior `next` and `sig` over `(seq,author,prev,next)`. Gossip is any-to-any; fork at same `seq` resolved first-writer-wins. Chain is audited on Freenet (slow plane) via `State { peers, chain }` contract. `next` is client-chosen `rand('a'..'z')` truly random.

## Contract `contract/src/lib.rs`

- State: `peers: BTreeMap<Pubkey, PeerRecord{peer_id,addrs}>, chain: BTreeMap<u64, ChainEntry{author,prev,next,sig}>, sigs`
- Params: `String lobby`
- Merge: verify ed25519, first-writer per seq, continuity check, gap buffering. `MAX_CHAIN=2048`.

## Run

```
cargo run -- --lobby demo-lobby --host 127.0.0.1 --port 7509 --seed 1
```

CARGO_TARGET_DIR=/tmp/frt-build required if path has spaces.

# freenet_example

A shared counter that anyone can run — just download, execute, and you're
participating in a global shared state over the Freenet P2P network.

## Goal

**A single executable that works on any platform.** Anyone downloads it,
runs it, and their machine immediately becomes part of a shared counter
contract across the Freenet network. No install steps, no toolchain, no
dependencies.

The counter persists on the network — restarting the binary fetches the
current global state, not a local cache.

The contract guarantees CRDT convergence and contract-fork isolation
(via `ContractKey = Blake3(WASM || params)`), not anti-cheat.

## How it works

```
Your machine                   Friend's machine
    │                                │
    ├─127.0.0.1:{port}               ├─127.0.0.1:{port}
    ▼                                ▼
 Local freenet node ─── P2P ───► Local freenet node
```

Each machine runs its own Freenet node (embedded in the binary). The nodes
sync contract state via the global Freenet P2P network. The deterministic
`ContractKey` (hash of WASM + params) is the global address — no IP sharing,
no server, no configuration.

The subscriber:
1. Loads the same WASM, computes the same deterministic `ContractKey`
2. Sends `Get { subscribe: true }`
3. If the contract doesn't exist yet, retries every second
4. Once found, joins the increment loop alongside the publisher
5. Both see each other's updates via pub/sub notifications

## Threat model — what "cheating" means here

On the canonical WASM `contract/src/lib.rs:7 GlobalCounterContract`,
values are client-reported and unauthenticated (`freenet-contract-design §6`):

- **Primary cheat to solve (P0): single-update jump-to-`MAX`.**
  Trivially send `BTreeMap{tag: u64::MAX}` as `UpdateData::State` and honest
  peers `max`-merge to it. Fix in candidates `CANDIDATE_1 (+1 bound, O(users))`
  and hardened in `CANDIDATE_2`.

- **Primary cheat to solve (P0): cross-tag impersonation.**
  `tag: u64` has no owner binding today. Any key can advance any tag.
  Fix is `CANDIDATE_2` — `Parameters` allow-list or derived `tag = hash(pubkey)`
  and per-update `ed25519` signature verified in `update_state`.

- **Secondary goal (P1, deprioritized): fast self-spam `+1,+1,...` at network rate.**
  Attacker loops honest `+1` to outrun 1Hz ticks. This is *spam, not cheat*
  (`CANDIDATE_1:39`, `freenet-contract-design choose-2` guidance). At the moment
  we do not rate-limit users at the contract layer — each `+1` is still one
  message, one honest unit of work. Contract enforces `value ≤ cur+1` (P0) but
  not `updates per wall-clock second`.

Non-goals for this example: server-trusted counter, Sybil resistance, global
ordering, rate-limiting, monetary settlement. Harder anti-cheat options deferred
to `docs/candidates/{CANDIDATE_1,CANDIDATE_2,CANDIDATE_3}` with scale/WASM tradeoffs
per skill choose-2.

## Achieved

- A single executable that starts an in-process Freenet node, deploys the
  contract, joins the global P2P network, and increments every second
- Cross-platform: CI builds and validates on Linux, macOS, Windows
- Cross-platform deterministic contract: all three platform binaries embed
  the same WASM, producing the same `ContractKey` — Linux, macOS, and
  Windows users share one counter
- No dependencies: WASM embedded at compile time via `build.rs`, node runs
  in-process
- A `GlobalCounterContract` WASM contract with validate, update, summarize, and delta logic
  (commutative monoid — correct idempotent `update_state`); current contract is
  honest-but-cheatable on same key (forked-client `MAX`/cross-tag not yet blocked)
  — hardened variants staged as `docs/candidates/` worktrees
- A WebSocket client library (`FreenetClient`) for talking to a Freenet node
- A `GlobalCounterClient` that handles the full lifecycle (deploy, subscribe,
  update, pub/sub notification draining)
- Automated tests: 22 unit + integration tests running in CI on 3 platforms
- End-to-end tests: binary smoke test, two-instance P2P sync test, WebSocket
  bridge test — all via subprocess-based verification
- CI/CD pipeline: auto-builds release binaries on tag push, publishes to
  GitHub Releases

## Get it

Download the latest binary for your OS from
https://github.com/Uomocosa/lele-rust-projects/releases

```bash
chmod +x freenet-example-linux
./freenet-example-linux
```

Press Ctrl+C to stop. Re-run to rejoin the global counter.

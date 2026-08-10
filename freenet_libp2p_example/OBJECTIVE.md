# freenet_libp2p_example — Coexistence spike

Does a freenet node and a libp2p swarm actually run in the same process, on the
same tokio runtime, at the same time?

## Result: YES

Measured 2026-08-10 with freenet 0.2.70, libp2p 0.56.0, tokio 1.52.

```
=== coexistence spike: 60s ===
freenet responses      : 6
libp2p  time-to-1st-ping: Some(11.475905ms)
freenet probe ended    : false
freenet FIRST response : [1.305102ms] ContractResponse(NotFound { ... })
freenet last response  : [7.217237216s] ContractResponse(NotFound { ... })
libp2p listen addrs    : 2
  /ip4/127.0.0.1/tcp/35181
  /ip4/127.0.0.1/udp/53566/quic-v1
libp2p connections     : 4
libp2p pings ok / err  : 120 / 0
COEXISTENCE            : YES
```

Both stacks made continuous progress for the full 60s: the freenet node served
websocket client requests and simultaneously joined the real Freenet network
(gateways `100.27.151.80:31337` and `5.9.111.215:31337`), while two in-process
libp2p swarms exchanged 120 pings with zero failures over both TCP and QUIC.

## What `src/main.rs` does

- Starts an embedded freenet node in Network mode (`is_gateway: true`) over a
  tempdir — same startup path as `freenet_bevy_example_2`, minus its 20s sleep.
- Spawns a websocket client that Gets a contract that cannot exist, every 5s,
  timing each request at the call site. A structured response proves the node's
  client API and executor are live.
- Builds **two** libp2p swarms in the same process; A listens on TCP + QUIC,
  B dials both. Ping RTTs prove the swarm is being driven.
- Runs 60s, then prints the report above.

## Findings that matter for downstream projects

**1. One `#[tokio::main]` runtime is enough.** freenet spawns its own tasks onto
the ambient runtime; `SwarmBuilder::with_tokio()` does the same. No nested
`Runtime::new()`, no executor conflict. Here both were driven from a single
`tokio::select!` loop on the main task.

> **Caveat for the Bevy projects:** this says the two *runtimes* don't fight.
> It does not mean "run the swarm on the main task." In a Bevy app `App::run()`
> owns the main thread, so the swarm still belongs on its own thread with an
> mpsc bridge — for Bevy's sake, not freenet's.

**2. No UDP conflict between freenet's transport and libp2p QUIC.** freenet
bound `[::]:60302`, QUIC bound an ephemeral `127.0.0.1` port. Both fine. QUIC
and freenet coexisting on UDP is not a problem.

**3. `NotFound` is a FALSE readiness signal — this is the important one.**
The first Get returned in **1.3 ms** (client-side) / **0.6 ms** (node-side,
`Registered transaction` → `Delivering result` for the same transaction id),
while the node's first gateway handshake only completed **38 ms later**. The
response came back **before the node had joined the network**: a Get against an
empty ring short-circuits to `NotFound` locally.

Every subsequent Get for the same nonexistent contract — once the ring had
peers — took **6.4 s to 7.2 s**, because it then actually searched the network.
The 1000× gap between the first Get and the rest is the whole finding.

Consequences:

- This is why `freenet_bevy_example_2` sleeps 20s before using its node. The
  sleep papers over the absence of a real readiness signal, and a naive "a Get
  returned, so we're ready" check does not replace it.
- **Any contract read issued during startup can return an empty/absent state
  that is indistinguishable from "the contract really is empty."** For
  `freenet_boxes` that means a joining player can read an empty roster and
  conclude nobody else is online. Readiness must be gated on ring/connection
  state (or a retry-until-non-empty with a time floor), not on getting *a*
  response.

**4. Freenet contract ops cost seconds, not milliseconds.** ~7 s per Get on a
live ring, measured repeatedly. Any design that needs sub-second interaction
must not route it through a contract — which is exactly the case for using
libp2p to carry real-time data.

**5. The embedded "gateway" joins the public Freenet network.** With
`is_gateway: true` and a public address set, the node contacts real remote
gateways and starts relaying subscribe operations for unrelated contracts
(observed: `SUBSCRIBE relay: processing Request ... upstream_addr=116.193.128.70`).
It is not an isolated local node. Worth knowing before running many instances.

## Run it

```
CARGO_TARGET_DIR=/home/uomocosa/.cache/frt-build RUST_LOG=info cargo run
```

`CARGO_TARGET_DIR` is required: this repo's path contains spaces, which breaks
freenet's `tikv-jemalloc-sys` build (see root `AGENTS.md`).

## History

Until 2026-08-10 this project claimed to prove coexistence but did not: it
declared `freenet` as a dependency, never used it, and carried a TODO where the
node startup should have been. The stated proof was never performed.

# Cross-network Linux ↔ Windows test — 2026-08-13

Two release binaries from the first GitHub release of
`freenet-libp2p-bevy-example-1`, run on the pure mainnet path
(no `--freenet-local`, no `--freenet-gateway`) to test the stated target of
"2+ machines on different networks".

| | Linux instance | Windows instance |
|---|---|---|
| identity dir | `/tmp/fbx_linux_20260813` (explicit) | none passed → **ephemeral** |
| log | `~/Downloads/fbx_linux_run_20260813.log` (2800 lines / 1.2 MB) | user-supplied |
| freenet ring connections | **26** | **0** for the whole run |
| roster entries | **3** (joined, merged, held) | never reached the contract |
| libp2p peer id | `12D3KooWR2ja2h9pPfcN9HZKpqFaKsQacdUk1Yw2v3YK4cGwGsD7` | n/a |
| outcome | healthy, own box only | node crash-loops on `EADDRINUSE` |

Linux host: LAN `192.168.1.9`, Tailscale `100.113.107.37`, public `94.33.34.31`.

## Headline

The two instances never connected, and **the cause is not the one the
"same network only" symptom suggests**. The Windows instance died at Freenet
bootstrap — two layers below libp2p — in a **permanent, self-inflicted retry
deadlock**. It never reached the roster contract, so it never learned the Linux
peer's address, so libp2p was never asked to dial anything.

A second, independent defect (finding 2) means that even a healthy Windows node
would very likely have failed to connect across networks anyway. Both need
fixing; finding 1 blocks first and is the more serious bug.

---

## Finding 1 — the node-startup retry loop deadlocks itself on a leaked UDP port (critical)

This is the bug. Observed sequence on Windows:

```
14:49:26  roster: starting in-process network-mode node ws_port=56483 public_port=63221
14:49:28  freenet::ring: Zero ring connections detected — starting isolation timer
14:49:57  RING_TRANSPORT_DESYNC: transport has connections but ring topology is empty
          transport_connections=2 ring_connections=0
14:50:57  roster: retrying embedded node startup attempt=1 backoff=5
14:50:57  roster: freenet connection error reason="failed to start embedded node
          (will retry): connection timed out"
14:51:02  roster: starting in-process network-mode node ws_port=62209 public_port=63221
14:51:03  Failed to bind UDP socket to [::]:63221: ... (os error 10048)   ← WSAEADDRINUSE
14:51:15  roster: starting in-process network-mode node ws_port=57606 public_port=63221
14:51:15  Failed to bind UDP socket to [::]:63221: ... (os error 10048)
14:51:15  roster: retrying embedded node startup attempt=3 backoff=15
```

Note `public_port=63221` on **every** attempt, and `EADDRINUSE` on every attempt
after the first. Two distinct defects combine:

### 1a. The failed node is never shut down — it keeps the socket forever

`src/roster/start_embedded_node.rs:78-84` spawns the node **before** the
readiness check:

```rust
tokio::spawn(async move {
    if let Err(e) = ::freenet::run_network_node(node).await { ... }
});

let mut probe = freenet::FreenetClient::connect("127.0.0.1", ws_port).await?;
probe.wait_ready(min_active_connections, Duration::from_secs(90)).await?;   // ← `?`
```

When `wait_ready` times out, the `?` returns `Err` — but the spawned task is
never aborted and no `JoinHandle` is retained. `NodeInfo` (`node_info.rs`) has
no shutdown handle at all. The half-dead node keeps running, keeps its UDP
socket bound to `[::]:63221`, and keeps servicing connections.

`connect_and_run.rs:31-48` then loops and calls `start_embedded_node` again.
The new node tries to bind the same port. It cannot. Forever.

**So the retry is not merely ineffective — it is guaranteed to fail on every
attempt after the first, and it leaks an entire live Freenet node per attempt.**
The backoff (`5s, 10s, 15s, …`) just spaces out the guaranteed failures.

This is a regression in intent: `connect_and_run.rs`'s doc comment explains the
retry was added to survive transient mainnet refusal (per
`MAINNET_3_INSTANCE_TEST.md` finding 1). It makes that case strictly worse —
before, a timeout left one healthy-but-unjoined node; now it leaves an
unbounded pile of them and can never recover.

Observed through **attempt 10**, at which point the user closed the window. From
attempt 2 onward the failure reason changes from `"connection timed out"` to
`"send error"`: the node now dies on `bind` within ~1 s, so the websocket probe
fails immediately rather than waiting out the 90 s readiness timeout. The loop
therefore spins roughly every 15 s, leaking a fresh temp dir and re-provisioning
a KEK each time (`.tmpjVdTkJ`, `.tmpaxTw5R`, `.tmpu89xX0`, `.tmp2MRXlB`,
`.tmpP0RlCz`, `.tmpDNxezI`, `.tmpOTgYJH`, `.tmpn2xwup`, … one per attempt).

### 1b. `free_udp_port()` probes the wrong address and returns an occupied port

`start_embedded_node.rs:10-13`:

```rust
fn free_udp_port() -> Result<u16, ...> {
    let socket = UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    Ok(socket.local_addr()?.port())        // socket dropped here
}
```

This is wrong on **every** platform, not just Windows — Linux only escaped it
because it never needed a retry. Do not scope the fix to Windows.

Three problems in four lines:

- It binds **`127.0.0.1`**, but freenet binds **`[::]`** (dual-stack, all
  interfaces). A port free on IPv4 loopback is not necessarily free on `[::]` —
  which is exactly why the probe kept handing back `63221` while the old node
  held it.
- The socket is dropped before the port is used — a textbook TOCTOU race even
  without 1a.
- Because the probe is deterministic under these conditions, the retry never
  even stumbles onto a different free port by luck.

**The same file already does this correctly for TCP.** `start_embedded_node.rs:30-32`
binds a real `TcpListener`, keeps the object alive, and hands it to
`serve_client_api_with_listener(ws_config, listener)` — no gap between probe and
use. And indeed the websocket port advances cleanly across retries
(`56483 → 62209 → 57606 → 52207 → 52211 → 52215 → 52219 → 52228`) while the UDP
port is stuck at `63221` every single time. Two port-allocation strategies in one
function; only the UDP one is broken.

**Suggested fix (both parts):**

1. Store the `JoinHandle` in `NodeInfo` and `abort()` it on the error path
   before returning `Err`; ideally give `NodeInfo` a `Drop` that tears the node
   down. Await actual socket release before retrying.
2. Probe with `UdpSocket::bind("[::]:0")` to match what freenet binds, or
   better, pass `network_port: None` and let freenet pick and report the port
   so there is no gap between probe and bind.

### 1c. Why the *first* attempt failed (separate, likely upstream)

Before the deadlock set in, attempt 0 failed on its own merits:

```
RING_TRANSPORT_DESYNC: transport has connections but ring topology is empty -
connections are not being promoted or are being immediately pruned
transport_connections=2 ring_connections=0
WARN freenet::transport::connection_handler: Outbound handshake failed:
     max connection attempts reached peer_addr=73.77.78.243:58229 attempts=12
```

Freenet established 2 transport connections but promoted 0 into the ring, and
every NAT traversal hit `max connection attempts reached` after 12 tries. My
Linux node on the same code did the opposite — 26 ring connections in ~30 s.

**This conclusion rests only on the clean window 14:49:26–14:50:57** — the first
node's lifetime, before any `EADDRINUSE` existed and before any node was leaked.
Everything cited in this subsection comes from that uncontaminated period, or is
corroborated by it. (Later evidence is weaker: by 14:53 several leaked nodes were
competing on the same host, so packet loss and `RING_TRANSPORT_DESYNC` after that
point prove nothing on their own.)

Corroboration from that clean window: `min_active_connections = 1` on the mainnet
path with a 90 s `wait_ready`. The node started at 14:49:26 and the retry was
logged at 14:50:57 — exactly the timeout expiring. So `wait_ready` never observed
a single *active* connection across 90 seconds, even while the transport claimed
2. That is only consistent with those 2 "connections" being one-way.

The later log confirms the reading:

```
WARN freenet::operations::connect: Fully isolated: cleared stale gateway
     reservations to unblock recovery total_gateways=2
WARN freenet_core::transport::keepalive_timeout: CONNECTION TIMEOUT -
     no packets received for 121.04s remote=100.27.151.80:31337
WARN freenet_core::transport::keepalive_timeout: CONNECTION TIMEOUT -
     no packets received for 123.18s remote=5.9.111.215:31337
```

`total_gateways=2` — **the gateway list was not empty.** Both gateways
(`100.27.151.80:31337`, `5.9.111.215:31337`) were found and dialed. The node sent
packets to them and received **nothing back for over two minutes**. That is the
signature of **inbound UDP not reaching the machine** — Windows Firewall dropping
it on port 63221, or a symmetric NAT that the traversal could not punch. The
transport optimistically counted 2 "connections" that were in reality one-way,
which is exactly the state `RING_TRANSPORT_DESYNC` reports.

So finding 1c is **environmental, not a missing gateway list**: the Windows host
cannot receive inbound UDP on the freenet port. That alone would have been a
recoverable, retryable condition — freenet was still resetting backoff and
clearing reservations to recover. The port leak (1a/1b) is what converted it
into a permanent failure.

**Action:** on the Windows machine, allow inbound UDP for the executable in
Windows Defender Firewall (and pass `--p2p-port <fixed>` so the rule can target a
stable port), then re-test. Fixing 1a/1b is still required regardless.

---

## Finding 2 — the libp2p NAT-traversal stack is wired in but completely inert

`src/p2p/behaviour.rs` declares `autonat`, `dcutr`, and `relay::client`, and
`build_swarm.rs:22` calls `.with_relay_client(...)`. None of it does anything:

- `grep -rn "add_external_address" src/` → **no matches.** `identify`'s
  `observed_addr` is never promoted to an external address.
- `grep -rn "p2p-circuit" src/` → **no matches.** No relay server is ever
  dialed, no circuit reservation is ever made. `dcutr` coordinates hole punching
  *over a relay connection*; with no reservation it can never fire.
- `run.rs:85-114` matches only `NewListenAddr`, `Positions`,
  `ConnectionEstablished` and `ConnectionClosed`. Every `identify`, `autonat`,
  `dcutr` and `relay` event falls into `_ => {}`.
- The only listens are `run.rs:26-33`: `/ip4/0.0.0.0/udp/0/quic-v1` and
  `/ip4/0.0.0.0/tcp/0`.

Consequence: `main.rs:38-46` builds `own_entry.addrs` purely from
`NewListenAddr`, i.e. local interface addresses. On this machine that is
`127.0.0.1`, `192.168.1.9` and `100.113.107.37` — verified via `ss -tunlp`
(QUIC `udp/33361`, TCP `45789`). A peer on another network is handed only
unroutable addresses.

**This is the direct mechanical cause of "works on the same network only."**

Instructive contrast: freenet's own transport *does* discover the public
address — the Linux log shows `peer=94.33.34.31:58856`. The libp2p half has no
equivalent, despite having the three behaviours that would provide it.

`OBJECTIVE.md` lists M4 (relay pool) as future work, which is a fair scoping
call — but having `autonat`/`dcutr`/`relay` sitting in the `NetworkBehaviour`
derive makes the feature look shipped. Either wire them up or drop them from
the behaviour and leave a comment.

**Suggested fix:** handle `identify::Event::Received` → `add_external_address`,
then either configure a relay and `listen_on(relay_addr/p2p-circuit)` or
document that cross-NAT play needs port forwarding.

---

## Finding 3 — the published address list is frozen 250 ms after startup

`run.rs:52-58` emits `Event::Ready` on a 250 ms timer after the *first*
`NewListenAddr`, and `std::mem::take`s `listen_addrs`. `main.rs:31-46` consumes
it once and builds `own_entry` once; it is never recomputed or re-published.

Any address discovered later — a relay reservation, an AutoNAT-confirmed
external address, a NIC coming up, a VPN connecting — can never reach the
roster. Note this would silently defeat finding 2's fix: external addresses
arrive via `identify` seconds after startup, long past the window.

---

## Finding 4 — a failed dial is never retried, and never logged

`src/p2p/bevy_systems/dial_roster_peers.rs`:

```rust
if *player_id == **config || dialed.contains(&entry.peer_id) { continue; }
commands.send(p2p::Command::Dial { ... }).ok();
dialed.insert(entry.peer_id.clone());
```

`DialedPeers` is inserted into *before* the outcome is known and is never
removed from, so **one failed dial permanently blacklists that peer** for the
process lifetime. The system is additionally gated on `roster.is_changed()`, so
a peer with a stable entry is never reconsidered.

Compounding it, `SwarmEvent::OutgoingConnectionError` is **not matched** in
`run.rs` — it hits `_ => {}`. The only dial-related log is a `warn!` when the
synchronous `swarm.dial()` call returns `Err`, which is not the interesting
case. A dial that times out against an unroutable LAN address produces **zero
log output**. That is precisely why the Linux log says nothing at all about
dialing despite the roster carrying two other peers the whole run.

**Suggested fix:** match `OutgoingConnectionError` and `ConnectionEstablished`,
log both at `info`, and remove the peer from `DialedPeers` on failure so the
next roster refresh retries it.

---

## Finding 5 — `HOME` is Unix-only, so Windows always gets an ephemeral identity

```
WARN p2p: no HOME set, using ephemeral identity
```

`src/p2p/load_or_create_keypair.rs:13`:

```rust
std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share/..."))
```

Windows uses `USERPROFILE`, and `.local/share/` is not a Windows convention.
Without `--identity-dir`, every Windows launch gets a new PlayerId, colour and
spawn position. Identity persistence is silently broken on a platform we ship a
binary for.

**Suggested fix:** `directories::ProjectDirs::data_dir()`.

---

## Finding 6 — the node always cold-starts from a throwaway temp dir

`start_embedded_node.rs:69-73` sets `config_dir`, `data_dir` and `log_dir` all
to a fresh `tempfile::tempdir()`, unconditionally, every run and every retry.
The node therefore never has a persisted peer database or `gateways.toml` and
always bootstraps from nothing. Each Windows retry created *another* one
(`.tmpjVdTkJ`, `.tmpaxTw5R`, `.tmpu89xX0` are visible in the log), re-provisioning
the KEK each time and discarding anything learned.

**Suggested fix:** a real per-user data dir, so peer knowledge survives restarts.

---

## Finding 7 — DEBUG logging is impossible in any release build

`RUST_LOG=...,roster=debug,p2p=debug` was rejected at startup on both machines:

```
warning: some trace filter directives would enable traces that are disabled statically
 = note: the static max level is `info`
 = help: to enable DEBUG logging, remove the max_level_info feature from the tracing crate
```

Source: `freenet-0.2.123/Cargo.toml:562-564` declares
`tracing = { features = ["release_max_level_info"] }`. Cargo feature unification
applies that to the entire build, so **no release build of this app can ever
emit DEBUG**, whatever `RUST_LOG` says. Combined with finding 4, the libp2p
layer is effectively unobservable in a shipped binary.

**Suggested fix:** promote the app's own diagnostics to `info!` — `debug!` is
dead code in anything we ship.

---

## Finding 8 — freenet INFO output drowns the signal

The Linux log ran to **2800 lines / 1.2 MB**, of which **52 lines (1.9 %)** were
`roster`/`p2p`. The rest is freenet internals at INFO: NAT traversal attempts,
ring promotions, keepalive tasks, idle-stream sweeps for dozens of unrelated
third-party peers. A live monitor filtering on `connection|error|dial` was
unusable — it matched almost exclusively freenet churn.

This is *not* the app's `EnvFilter` default being wrong; `main.rs:15-16` already
defaults to `warn,roster=info,p2p=info`. Something overrides it, and finding 9
is the likely path.

---

## Finding 9 — Bevy's LogPlugin fights the app's subscriber

On both platforms, every run:

```
ERROR bevy_log: Could not set global logger and tracing subscriber as they are
                already set. Consider disabling LogPlugin.
```

`main.rs:15-20` installs a `tracing_subscriber` before `DefaultPlugins`. Taking
bevy_log's own advice — `DefaultPlugins.build().disable::<LogPlugin>()` — would
clear the error and make filter behaviour deterministic. Worth doing before
investigating finding 8, since it may be the cause.

---

## What worked

- **The Freenet roster contract on the public mainnet, from Linux.** `Get`
  returned 2 existing entries, the commutative merge produced `merged_len=3`,
  and both the pull refresh and push `UpdateNotification` held at 3 entries for
  the entire ~8-minute run with no drift. The M2 contract layer is sound.
- **`is_gateway=false` on the mainnet path** — the "every game client
  advertises itself as a public gateway" defect
  (`MAINNET_3_INSTANCE_TEST.md` finding 2) is **fixed**.
- **`public_address` is now `local.then_some(...)`** rather than hardcoded
  loopback — `MAINNET_3_INSTANCE_TEST.md` finding 3 is **fixed**.
- **World bounds exist** (walls visible in the screenshots) —
  `MAINNET_3_INSTANCE_TEST.md` finding 4 is **fixed**.
- **An on-screen `freenet: connected` status indicator exists** —
  `MAINNET_3_INSTANCE_TEST.md` finding 1's "failure is invisible to the player"
  gap is partly closed. It still does not distinguish *connected to freenet*
  from *connected to another player*, which is the state a player actually
  cares about; the Linux window read "connected" for the whole run while being
  alone.
- Local gameplay: input handled, box moved on `A`/`D`, physics and rendering
  fine on Vulkan/radv.

---

## Priority

1. **Finding 1** — the retry deadlock. Critical, self-inflicted, and it masks
   everything else. Nothing can be concluded about cross-network behaviour until
   a failed startup can actually recover.
2. **Finding 2** — the real "same network only" bug. Even a healthy node is
   handed unroutable addresses.
3. **Findings 4 + 7 + 8 + 9** — the debuggability cluster. Fix these next, or
   every future investigation costs another manual two-machine session. Finding
   4 in particular meant the single most important question of this test — "did
   libp2p even try to dial?" — could not be answered from the logs.
4. **Findings 3, 5, 6** — real but narrower.

## Re-test plan

1. Fix 1a/1b (abort the leaked node; probe `[::]:0` or let freenet pick).
2. On Windows, add an inbound UDP firewall rule for the exe and run with a fixed
   `--p2p-port` plus `--identity-dir` (working around finding 5).
3. Re-run. The success criterion for this stage is only that the Windows node
   reaches non-zero `ring_connections` and the roster shows 4 entries on both
   sides.
4. Only then is finding 2 testable — expect the boxes still not to sync, with
   the Linux peer advertising `192.168.1.9` to a machine that cannot route to it.

## Also worth fixing while in here

`connect_and_run.rs` retries `start_embedded_node` forever with a backoff capped
at 15 s and no attempt ceiling, and the failure never becomes visible in the
game window beyond the existing `freenet:` status line. Ten failed attempts in
four minutes produced no player-facing signal that anything was wrong. Surfacing
`roster::Event::ConnectionError` with the attempt count would have made this
diagnosable without reading logs at all.

## Unconfirmed premise

Whether the Windows machine was on a *different* network than the Linux one was
asked but never confirmed. No conclusion here depends on it: finding 2 is
established from source, and finding 1c's blocked-inbound-UDP reading holds
either way. But if the two machines were in fact on the same LAN, this run did
not test the cross-network case at all, and the re-test must ensure they aren't.

## Artifacts

- `~/Downloads/fbx_linux_run_20260813.log` — full Linux stdout+stderr. Was 2800
  lines / 1.2 MB at the time of the counts quoted above; **the process was still
  running when this report was written**, so the file has grown since. The 52
  app-lines figure in finding 8 is a snapshot, not a final count.
- `~/Downloads/fbx_windows_run_20260813.log` — the Windows instance's output,
  transcribed verbatim from the terminal. Covers ~14:49:25 to ~14:53:15, through
  `attempt=10`; `attempt=4` (~14:51:15–14:51:49) is missing from the capture.
  This file is the evidence base for findings 1a, 1b, 1c and 5.
- `deskctrl_mcp/artifacts/claude_code/1786632424.mp4` — 56 s window capture,
  local box responding to input, one box on screen.
- Three window screenshots: initial, post-input, and final (all identical in
  substance — one blue box, `freenet: connected`, no remote peer ever appeared).

## Cleanup still needed

The Linux instance is **still running** as pid 6183 (I was not permitted to kill
it). Stop it with `kill 6183`. Its Freenet node is an active mainnet participant
until then.

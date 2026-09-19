# MAINNET PROBLEMS 8 — turmoil connection fidelity: dial deadline, NAT model, discovery-driven dial

Follow-up to `MAINNET_PROBLEMS_7.md` §2 (next functionalities A–G) and
`TURMOIL_TO_EXPLAIN.md`. This session implemented §2A (dial failures +
ghost pruning), §2B (seed matrix) and §2C (partition-during-handshake /
dial deadline) **plus the §2D enabler** (address/NAT model and
discovery-driven dial), all in the fast turmoil rig. **No product
`src/` behaviour changed** — the work is `clicker/src/testing/turmoil/`
and `clicker/tests/turmoil/`.

## 0. Why this layer

The rig's *network* fidelity (latency, loss, partition/repair, hold,
star topology, crash=stream-death) was already good. The gap was the
**connection/discovery layer**: the rig never dialed — it hardcoded the
outcome (`link_app` pre-populated the roster and pushed synthetic
`PeerConnected` for all peers). Every real mainnet failure in
`_1`/`_2`/`_3` lived in exactly that layer: cold-start roster partitions
(`_1` P1), ghost dead ports (`_1` P2), simultaneous-dial storm / no
hairpin (`_2` F2, `_3` #2), stale-hint churn (`_3` #4). Rows #8–#13 of
the `_7` coverage matrix were ❌ for this reason.

## 1. What was built

**Slice 1 — bounded dial lifecycle (`_7` §2C, matrix #8/#12).**
`dial_with_deadline.rs`: retries `TcpStream::connect` until a per-profile
`dial_within` budget elapses, then returns `Err("timeout")` instead of
hanging the sim. `bring_up` surfaces failure as
`p2p::Event::DialFailed` and records the peer in `dial_failed`:

```rust
match dial_with_deadline(target, PORT, dial_within).await {
    Ok(mut stream) => { /* write_hello + register + PeerConnected */ }
    Err(reason)   => { dial_failed.push(target.to_string()); push_failed(&mut app, target, reason); }
}
```

Red test `tests/turmoil/connect/dial_deadline_surfaces_failure.rs`: a
ghost host `peer-9` is dialed and must fail while the three real peers
still converge to (1,5,17). This is the `_5.md:150-158 §9` product path
(route `DialFailed` into discovery) expressed as a fast test.

**Slice 2 — address/NAT model + ghost hints (`_7` §2A, matrix #9/#10/#13).**
`Nat` (`nat.rs`) with `Open` / `NoHairpin { public_ip }`, `public_ip.rs`
and `dialable.rs` decide whether a dial is allowed; `GhostHint` seeds a
previous-run address whose port is closed. `NetworkProfile` gained `nat`
and `ghosts`. Red test `hairpin_blocks_same_public_ip.rs`: under
`HAIRPIN` all three peers share one public IP, every outbound dial is
refused (`DialFailed`, zero connections) — the `_1` P1 / `_2` F2
same-public-IP no-hairpin case, now deterministic and fast.

**Slice 3 — discovery-driven dial (matrix #10/#12, the §2D enabler).**
Replaced `link_app`'s synthetic `PeerConnected` flood with **real TCP
handshakes**: a persistent `accept_loop` (listener lives for the host's
lifetime), a length-prefixed `write_hello`/`read_hello` name exchange,
and `register` wiring the accepted stream into `outbound` + `inbox`.
`plan_for.rs` replaces the old one-directional `lanes_for` dial pattern
with a deterministic tie-break: the **higher** own id dials, the lower
accepts. `redial_missing.rs` redials unreached dial targets on a
`redial_every` period, modelling the production 15s redial. Red test
`cold_start_stagger_converges.rs` (`COLD` profile, 1s stagger) proves
cold-start with real dials still converges.

**Seed matrix (`_7` §2B).** `total_retained_seed_matrix` and
`diverged_survivors_heal_after_repair` now sweep `[7,8,9]`.

## 2. Rig incident worth remembering

The first `bring_up` bound the listener, dialed, accepted the expected
inbound count, then **dropped the listener** — so any peer that dialed
later could never reconnect, and the slice-3 convergence test was
non-deterministic (the one-shot startup `Move` was published before an
inbound peer registered). Fix: a persistent `accept_loop` task fed into
`pump`, plus draining expected accepts in `bring_up` before returning so
the startup click is delivered with all links present. Lesson: *a
listener is part of a host's identity, not a setup step.*

## 3. Coverage matrix delta (from `_7` §1)

| # | Failure (production) | Before | After |
|---|---|---|---|
| 8 | Dial failure / timeout / `DialFailed` | ❌ | ✅ `dial_within` deadline + surfaced `DialFailed` |
| 9 | Stale roster / dead ephemeral ports | ❌ | ✅ `GhostHint` → `DialFailed` (prune path live) |
| 10 | Discovery (PEX/mDNS/gossip-mirror learning) | ❌ | ◑ real dials + tie-break; PEX/mDNS still offline |
| 12 | Simultaneous-dial collision / tie-break | ❌ | ✅ higher-id-dials deterministic tie-break |
| 13 | NAT hairpin (same-IP dial fail) | ❌ | ✅ `Nat::NoHairpin` |
| 2 | Stall / heal burst | ✅ | ✅ unchanged |
| 4 | Crash / process kill | ✅ | ✅ unchanged |

Still slow-gate-only (unchanged): real Freenet `Get` routing stalls, real
NAT traversal, cross-OS sockets, real-Bevy-time periodic paths, mDNS/PEX
end-to-end, contract replica splits.

## 4. Scoreboard

- Fast: **325 nextest green, 1 pre-existing DISPLAY failure, 11 skipped**
  (turmoil: **17/17 green**, was 11). `clicker` mesh total ~26s.
  `ui_test_plugin::tests::test_usage` fails only because no `DISPLAY` is
  set (winit event loop) — identical failure before this session's work.
- Static gates green: clippy `-D warnings`, fmt, `lele_lint`,
  `lele_bevy_lint`, `taxonomy_check`.
- The `tests/mesh → tests/offline + tests/turmoil` reorg and the
  `src/testing/turmoil/` extraction (in-flight from prior sessions, 100+
  paths) are included in this commit.

## 5. Next (from `_7` §2, remaining)

- **§2D duplicate connections / link counting** — second stream per pair,
  close one mid-test, assert exact-once roster (P4 regression guard).
  Needs sub-connection identity in the envelope.
- **§2F join handshake over loss** — `WantJoin`/`Welcome` over
  `flaky-nat` (the rig's `flaky-nat` profile exists; `join.rs` is
  in-memory only).
- **PEX/mDNS end-to-end over real streams** — matrix #10's remaining ◑.
- Slow gate `freenet:run-rooms-rejoin` still the only oracle for the
  `_7` §4 RED (roster `Get` stall past the 5s click-path budget on a
  churned ring) — none of the rig work touches that path.

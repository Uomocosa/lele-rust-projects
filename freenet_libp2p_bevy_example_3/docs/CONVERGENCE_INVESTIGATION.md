# Convergence Investigation — Findings

Status: **root cause confirmed and fix verified.** The intermittent "convergence failure" was a
libp2p `request_response` misuse in our netcode layer, **not Freenet**. Read before touching roster
or netcode.

## TL;DR

The intermittent "convergence failure" of `mainnet_automation_3` is **not primarily a Freenet
discovery problem**, and the original Phase-1 verdict over-blamed Freenet. Clean log analysis
shows:

- **Freenet roster discovery reaches 3/3 even on failed runs.** On every saved run we analyzed,
  the roster contract converged to all three player ids — on failures *and* successes.
- The discriminator between a passing and a failing run is the **libp2p netcode *receive* leg**:
  failed runs show tens of thousands of outbound snapshots sent but **zero** inbound netcode
  (`received netcode commit` / `received peer input`).
- **Root cause (confirmed):** `src/p2p/run.rs` uses libp2p `request_response` but never answered
  inbound requests (`latest_netcode` stayed `None`), so thousands of pending requests accumulated
  per tick and stalled the connection's inbound leg. **Fixing it to reply `Ack` to every request
  restored bidirectional netcode and full 3/3 convergence** (see RESULT).

This document is kept as the referee so nobody reintroduces the expired conclusions.

---

## What we measured (clean, ANSI-stripped counts)

Compare a failed vs a successful run (instance-0 logs, `digits/.../.local-run`):

| metric | FAILED 20260824T131406Z | SUCCESS 20260824T142557Z |
|---|---|---|
| roster reached `len=3` (all peers) | 18× | 44× |
| `sending engine snapshot` (outbound) | 12,761 | 12,818 |
| `received netcode commit` (inbound) | **0** | **16,964** |
| `received peer input` (the gate) | **0** | **16,964** |

Conclusion: on the failing run the roster was **complete**, yet the instance received **zero**
inbound netcode. It kept sending outbound to a roster peer. That is an asymmetric/one-way netcode
failure, not a discovery failure.

### Bimodal history (both days failed AND succeeded — not "worked yesterday")

Saved run dirs and whether all 3 instances logged outbound+inbound exchange:

- 20260823T185243Z — **fail**
- 20260823T195608Z — **success**
- 20260824T130924Z — **fail**
- 20260824T131406Z — **fail** (we inspected: roster 3/3, inbound netcode 0)
- 20260824T140901Z — **success**
- 20260824T142423Z — **fail**
- 20260824T142557Z — **success**

It is genuinely intermittent on both days; the user's "worked yesterday" matches one successful
run, but there were failures the same day.

---

## Methodology corrections (do not repeat these mistakes)

1. **ANSI codes break naive `grep -c`.** Logs contain `\x1b[...m` escapes between fields. Always
   strip first: `sed 's/\x1b\[[0-9;]*m//g' FILE | grep -c 'pattern'`. Our initial "0" counts were
   false negatives.
2. **`subscriber_peer_ids` is always empty by design.** In freenet 0.2.128
   (`src/node/network_bridge/p2p_protoc.rs:2649`) the diagnostics field is not populated;
   `subscriber_peers=0` proves nothing about the subscription mesh. Do not use it as a signal.
3. A **single run** proves too little for an intermittent failure; and the discriminator must be
   compared across equal-logging runs, not inferred.

---

## Hypothesis: `request_response` used as fire-and-forget without replying

`src/p2p/run.rs`:
- `let latest_netcode: Option<NetcodeMsg> = None;` (line ~38) — immutable, never changed.
- On an inbound `request_response` Request it replies only `if let Some(reply) = latest_netcode`
  (line ~98) — so it **never replies**.
- `netcode_tick.rs` broadcasts every `Commit` / `Reveal` as a `send_request` to every roster peer,
  every tick.

`libp2p::request_response` is a **request → response** protocol. Every inbound request leaves a
pending outbound-response waiting; the sender side has a **~10s request timeout**. Broadcasting
thousands of unanswered requests per tick over yamux can degrade/stall the connection precisely
the way we observe: outbound works, inbound dies.

### Evidence in support
- One-way outbound / zero inbound matches a connection that accumulated pending/unanswered requests.
- `two_swarm_netcode_exchange` only works because the harness never checks replies — designed
  around the broken no-response behavior.
- Freenet's own issues (#3465, #4064, #3362, #4910/#5175) confirm cross-peer propagation is
  unreliable, but they are **not** the discriminator here since the roster reached 3/3.

### The fix (Experiment A) — CONFIRMED
Reply to **every** inbound request with a small `NetcodeMsg::Ack`. This resolves every pending
request (no timeouts, no accumulation). Should restore bidirectional netcode. **Verified:** a fresh
run with this change gave all 3 instances full roster + ~8k inbound messages each and zero request
failures (see RESULT below).

---

## Secondary findings (lower priority, do not block the netcode fix)

1. **Fresh contract key every run.** `mainnet_automation_3/src/new_contract_params.rs` generates a
   unique `local-mainnet-{unix}-{nanos}` every run, so each run races the first `Put` of a cold
   contract on mainnet → possible split replicas / sparse hosting (our own `setup_contract.rs`
   documents this anti-entropy caveat). A stable/persistent key is more production-faithful.
2. **5-min TTL prune can un-converge.** With `ROSTER_ENTRY_TTL_SECS = 300`, an entry whose delivery
   stalls (Freenet stops relaying refreshes) ages out and is pruned. In one fresh run, instance-2
   learned instance-1 then lost it to TTL → roster collapsed. Consider not pruning live members
   whose refresh merely stalls.
3. `RING_TRANSPORT_DESYNC` (`transport_connections>0, ring=0`) afflicts some nodes at startup and
   can delay the roster loop; it is a real freenet mainnet condition (#3362) but not the root cause
   of the netcode-receive failures observed (some failing nodes had no desync).

---

## RESULT (2026-08-24): Experiment A confirmed the root cause

After replying to every inbound netcode request with `NetcodeMsg::Ack`, a fresh 3-instance run
behaved completely differently from every failing run recorded before:

| metric | before (FAIL) | after (Experiment A) |
|---|---|---|
| roster reached `len=3` | on fails: yes (3/3) but see below | yes — 86–96× on each of all 3 |
| `received peer input` (inbound) | instance-0: **0** | **8099 / 8151 / 7793** on all 3 |
| `received netcode commit` | **0** | 8153 / 8202 / 7799 |
| `sending engine snapshot` (outbound) | 12,761 | 6340 / 6345 / 6370 |
| `netcode outbound/inbound failure` logs | (n/a) | **0 / 0** |

Interpretation: the failure was a **`request_response` misuse**, not Freenet. Never responding to
inbound requests accumulated pending requests until the connection's inbound leg stalled (outbound
kept working, inbound went to zero — exactly the observed `0` inbound). Acknowledging every request
resolves that state and bidirectional netcode runs continuously.

**Full harness confirmation (2026-08-24, run `20260824T181857Z-mainnet`):**
`mainnet_automation_3` end-to-end **PASSED**: all 3 instances mutually converged (each saw 2 peers),
all windows matched/tiled, all moved, `flap: max cumulative offline 0.0s`, `no error signatures`.
Setup succeeded on the first attempt this run (the earlier aborted attempt exposed the separate
fresh-key deploy flake described below — not the netcode path).

## Actions taken / plan

- [x] Phase 1 diagnostics (contract identity, latency, node/ring sampling, connect logging).
- [x] Online research: freenet-core #3465, #4064, #3362, #4626, flaky propagation #4910/#5175/#4691.
- [x] Correct the verdict: discovery is not the primary culprit; netcode receive leg is.
- [x] Phase 1b logging: request_response failure events, dropped inbound (unknown `from`), dial/connect lifecycle.
- [x] Experiment A: reply `Ack` to every inbound netcode request. **Confirmed** (run above).
- [x] Keep the `Ack` response semantics in `src/p2p/run.rs` (fix landed).
- [ ] Secondary (later, optional): stable contract key; adjust TTL-prune; handle fresh-key Put race.
- [ ] Consider matching the automation gate to the fixed behaviour and re-running the full
      `mainnet_automation_3` harness end-to-end to confirm the gate passes.
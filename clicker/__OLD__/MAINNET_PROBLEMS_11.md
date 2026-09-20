# MAINNET PROBLEMS 11 — slow RED: instance-1 never merges player 3's score

Follow-up to `MAINNET_PROBLEMS_10.md`. Slow gate
`freenet:run-rooms-rejoin` on the leave-sticks + dedupe + 500ms-budget tree:
**RED** — `phase converge: no baseline agreement`
(run `.local-run/rooms-rejoin-20260919-124436/`, 332s, clip on Telegram).

## Evidence

After the 15-clicks-each drive, two instances agreed at 45 while the
creator never saw player 3's score at all:

```
instance-2.log tail  sync lobby=room-1789821893 p1=15 p2=15 p3=15 players=1,2,3 global=45
instance-3.log tail  sync lobby=room-1789821893 p1=15 p2=15 p3=15 players=1,2,3 global=45
instance-1.log tail  sync lobby=room-1789821893 p1=15 p2=15 p3=0  players=1,2   global=30
```

Stronger than `_9 §2` (41s lag, then converged): instance-1's `p3=0`
for the **entire run** — zero of player 3's 15 clicks ever merged.
Identity was not the blocker: instance-1 resolved `player=3` at
12:47:39, before the drive (~12:48:00), and held `players=1,2,3` for
33 sync lines. It learned player 2's score fine (`p2=15`); only
player 3's entries never landed in a labeled slot.

The new `baseline-converge-ms` soft check never got to report: the
outer 60s `agree_within` found no agreement, so the phase errored first.

## Reading

Suspect path is the libp2p sync leg only (layer rule, `_10.md`):
`send_sync_req` / heartbeat → `SyncAck` carrying p3 entries →
`absorb_sync` merge into instance-1's labeled p3 slot. Open whether
(a) no `SyncAck` with p3 entries ever arrived (transport: idempotency
guard stuck, heartbeat never fired for that peer, roster-growth never
triggered), or (b) entries arrived but the provenance-gated merge
dropped them (`never_seen_tombstone_excluded` family). `RUST_LOG=info`
cannot distinguish (a) from (b); the fast repro must log the decision.

## Next (TDD, one slice)

FAST RED in the offline/turmoil rig: app1 with a labeled p3 slot at 0,
a peer holding p3=15, one `SyncReq`/`SyncAck` round → assert app1
reaches 15 within N ticks, with logging at the suspect functions
(requester set, reply entries, merge decision) BEFORE any fix.
System-level, not unit: `sync_ack_merges_labeled_slot` already passes,
so the repro must include the request/heartbeat trigger leg.

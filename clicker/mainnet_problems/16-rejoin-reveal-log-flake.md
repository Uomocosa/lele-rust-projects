# 16. Rejoin reveal log flake (`join reveal` missing) — OPEN

Follow-up to: `mainnet_problems/13-converge-budget-500ms.md` verification.
Related: `mainnet_problems/10-join-handshake-welcome.md`, `mainnet_problems/12-leave-sticks-dead-cursor.md`.

## Symptom

After the app-3 and creator rejoins, the `rejoin-3` / `rejoin-1`
checks "a survivor logged the shared reveal after the rejoin" fail:
`instance-1.log` / `instance-2.log` carry no `join reveal` line from
`reveal_on_join` (`src/clicker/bevy_systems/reveal_on_join.rs:53`),
even though every agreement/owner check passes.

`phase_finish` promotes the soft ❌ to fatal, so the slow gate REDs on
it alone.

## Evidence (logs)

Absence of the line (`clicker/.local-run/`, `instance-{1,2}.log`):

- `rooms-rejoin-20260921-081242/` (Phase 4A run 3): `join reveal` = 0.
- `rooms-rejoin-20260921-085608/` (HEAD A/B run): `join reveal` = 0.

Rate (before this slice): **9/20** —
`grep "❌ rejoin-3 a survivor logged the shared reveal"` over the last
20 `telegram_bot/.local-run/rooms_rejoin-*.log` summaries.

## Root cause

Not yet established. Candidate: the rejoining peer is labeled by
`resolve_player` (Move gossip) instead of the commit-reveal path, so
`reveal_on_join` takes the already-labeled branch and skips its
`join reveal` log. `resolve_player` defers to reveal only when the peer
is present in `JoinGate.pending` (`armed_pending_defers_label_to_reveal`);
the flake suggests the pending entry is sometimes already gone/never
armed when the identity arrives.

Confirmed **independent** of the #13 probe fix: identical on HEAD and
on the fixed build.

## Fix

None yet (open).

## Verification

- Fast suite green (does not exercise the mainnet rejoin timing).
- Slow: RED only on this check; all converge/owner/agreement checks
  pass (this check fires in 5/5 post-#13 runs and the HEAD A/B run).

## Open / next slice

- Instrument `reveal_on_join` / `resolve_player` decision (decision log:
  `CLICKER_DECISION_LOG`) to capture whether `gate.pending` held the
  rejoiner at label time.
- Reproduce deterministically in `tests/offline/join/` (withhold the
  Move gossip while the commit is in flight) before changing behavior.

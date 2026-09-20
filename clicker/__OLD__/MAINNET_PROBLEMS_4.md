# MAINNET PROBLEMS 4 — TDD loop to first full `rooms_rejoin` pass

Follow-up to `MAINNET_PROBLEMS_3.md`. The `_3` tree (all uncommitted) is
committed here together with this session's work. **First full
`rooms_rejoin` pass on record**, plus `mainnet_local` green.

## 1. Method: `definition-tdd` skill + fast-red/slow-gate loop

New global skill `definition-tdd`
(`~/.config/opencode/skills/definition-tdd/SKILL.md`, project-agnostic and
language-agnostic, definitions + pseudocode only), wired into
`projects/opencode.json: instructions[]` (covered by the existing
`"definition-*": "allow"` glob — no permission change needed).

Loop per iteration: FAST RED (one fast failing test) → LOG FIRST
(`tracing!` before fixing) → GREEN (minimal fix) → FAST FULL
(`lele:build/clippy/fmt/nextest/lint/taxonomy_check/bevy-lint`, all green)
→ SLOW GATE (one `rooms_rejoin` run, stop on first red) → next iteration.
Six slow runs total, each yielding exactly one fast-testable defect.

## 2. Iterations (evidence → fast test → fix)

**Iter-0 — gossip-learned peers invisible.**
Bevy `roster::Roster` (read by `spawn_on_join`) was fed ONLY by
`PeerConnected` events (`poll_roster`); gossip roster only produced `Dial`
commands. A joiner who learned occupants via gossip/directory but had no
libp2p connection yet saw nobody.
Fast test `room_presence::gossip_learned_peer_visible_without_connection`
RED in 0.078s. Fix: `absorb_roster` records fresh gossip entries into the
roster resource (blake3 key, same as `poll_roster`) + `tracing::debug`
logging of gossip decisions.

**Iter-1 — drive clicks miss instance-1 (`p1=1`, baseline gate).**
Video frame showed floating `clicker-xterm-*` windows covering
clicker-1's center; `xdotool click 1` lands on the topmost window, so all
15 drive clicks went into a terminal.
Fix (harness, `testing/mouse_click_at.rs`): `windowactivate` + 300ms
settle before every click. Proven by experiment: `xwininfo -root -tree`
stacking order is invariant under `windowraise` (no-op under this WM) and
flips under `windowactivate`. Next run: `baseline: p1=16 p2=16 p3=16`.

**Iter-2 — ghost slot after rejoin (`global=64`).**
Iter-0's add resurrected dead peers: the killed instance's contract entry
stays fresh for `STALE_ENTRY_SECS` (300s), so gossip re-added it, the
slot respawned, and the tombstone restored 16 → duplicate PlayerNo(3),
`p3=16` but `global=64`, and `agree_within` (which requires
`global == p1+p2+p3`) could never pass.
Fast test `gossip_does_not_resurrect_removed_peer` RED. Fix: presence
memory in `absorb_roster` — gossip never re-adds a vanished member; only
`PeerConnected` (via `poll_roster`) may reintroduce one.

**Iter-3 — dial-connected storm (cause #2 of `_3`).**
Instance force-dialed an 11×-connected peer every ~19s. Root cause: the
5s roster heartbeat bumps `updated_at`, defeating the dialed-map, so
every heartbeat re-dialed every peer (QUIC opens a new sub-connection
per dial).
Fast test `absorb_roster::connected_peer_never_redialed` RED. Fix:
dial-once per gossip id in `absorb_roster`; retries belong to
discovery's 15s `redial_missing` (which is connected-aware).

**Iter-4 — telegram caption too long (harness).**
Game fully green, but the checklist caption overflowed Telegram's
1024-char photo/video caption limit → `sendVideo 400`.
Fix in `telegram_bot` (central): new `truncate_caption` (1000 chars,
char-boundary safe) applied in `send_video`/`send_photo`, covering all
callers (`rooms_rejoin`, `mainnet`, previews).

**Iter-5 — rejoin frozen at `p3=1`.**
`publish_snapshot::snapshot_entries` preferred a live labeled slot over a
tombstone even when the tombstone was higher: the rejoin's own slot
(count 1, from the UI-join click) shadowed the parked (3,16), so every
fetched history carried (3,1) and restoration was impossible.
Fast test `publish_snapshot::tombstone_wins_over_stale_live_slot` RED.
Fix: max-merge live slots with tombstones per id.

## 3. Incidental cleanups (were blocking fast-full)

- Clippy `-D warnings` failures in the uncommitted tree
  (`needless_continue` in `absorb_sync`/`apply_delta`, `redundant_guards`
  + `match_same_arms` in `main.rs`, `too_many_arguments`/`needless_collect`/
  `collapsible_if`/`needless_pass_by_ref_mut`/`too_many_lines` in
  `discovery/run.rs` — extracted `connect_directory_retry`, `DialHub`
  bundle; `apply_room` 8→6 params by dropping the despawn loop, already
  covered by `despawn_menu` on `OnExit(Menu)`).
- `lele_lint` E001/E015 in `testing/click_window.rs` → split into
  `click_fraction.rs`, `click_at_y.rs`, `drive_center.rs`,
  `window_id.rs`, `window_size.rs`, `mouse_click_at.rs`
  (note: the linter's own hint suggested `click_window_*` names, but E001
  requires exact stem match — renamed to exact).
- `mouse_click_at` err-path test initially failed (xdotool accepts bogus
  wid); the raise/activate-first change made it fail properly again.

## 4. Results

- Fast: 230 nextest green + clippy/fmt/lint/taxonomy/bevy-lint;
  `telegram_bot` gates green.
- Slow: `rooms_rejoin` **PASS** (546s, incl. Telegram clip),
  `mainnet_local` **PASS** (103s). Final states `p1=16 p2=16 p3=16
  global=48` everywhere, app3 rejoin + creator rejoin both intact.

## 5. Still open (from `_3`, unchanged)

1. `freenet:e2e-matrix` overnight run (tcp-only/quic-only/both/lan-final)
   — never run.
2. Contract change to per-(room,peer) directory entries (LWW flapping,
   `_3` §3.4) — unblocked (harness validates), not yet attempted.
3. Per-peer (not per-line) resolution counting for the converge gate
   (`_3` §4.4) — the line-count gate is still weak.
4. Mid-chain despawn race: `despawn_on_leave` vs queued commands on the
   same entity panics in tests (`Entity despawned ... 34v0` seen while
   writing the iter-2 test; same WARN class as the menu-teardown lines
   in e2e logs). Candidate for the next fast test.

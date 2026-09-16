# MAINNET PROBLEMS 3 — UI rooms, PEX, mDNS, matrix campaign

Follow-up to `MAINNET_PROBLEMS.md` and `MAINNET_PROBLEMS_2.md`.
Covers the work since `_2` (all currently **uncommitted** in the tree)
and where the `rooms_rejoin` e2e test fails. No full pass yet.

## 1. What was built since `_2`

All changes are in the working tree, verified with
`cargo build/clippy/fmt/nextest/lele_lint/taxonomy/bevy-lint`
(219 lib tests green, plugin 35 green) but **not committed**.

**Track A — PEX discovery + shared hint store** (`clicker/src/discovery/`,
`clicker/src/clicker/`):
- `CursorMsg::{PexAsk, PexResp}` ride the existing request_response
  channel (`clicker/cursor_msg.rs`) — no new swarm behaviour.
- `PeerHint` gained `rooms: Vec<String>`; new `HintStore`
  (`peer_hint_store.rs` + `peer_hint_store_insert.rs` (LWW, 128 cap)
  + `peer_hint_store_prune.rs` (300s expiry)).
- Discovery task asks on every new connection and re-asks every 30s
  (`PEX_INTERVAL_SECS`); replies merge into the hint store and unknown
  ids go through the existing tie-break dial path; PEX rooms merge into
  the directory view sent to the menu (`send_merged_directory`).
- Gossip/directory-learned hints are now **inserted into the hint
  store** (previously dial-once-and-forget, unredialable — the run-5
  hole from `_2`).
- Bevy game systems explicitly **drop** PEX messages (`apply_delta.rs`,
  `absorb_sync.rs`); mesh tests `pex::{ask,resp}_dropped_by_game_systems`
  prove no credit/park leaks.
- LWW merge unions room lists (`merge_peer_hints.rs`).
- New `clicker/README.md` discovery chapter (genesis/PEX/gossip/mDNS,
  merge rules, caps, tie-break).

**Track C — mDNS** (`freenet_libp2p_bevy_plugin`, clicker CLI/tests):
- `libp2p` gains the `mdns` feature; `Behaviour.mdns` is a runtime
  `Toggle<mdns::tokio::Behaviour>` (no second swarm type).
- `Command::SetMdns`, `Discovered → Dial` (transport-mode filtered) +
  `AddKadPeer`; `Expired` ignored.
- `--disable-mdns` CLI (mdns **on** unless disabled); `spawn_runner`
  takes the flag; `spawn_xterm` threads it; both e2e tests read
  `CLICKER_MDNS=on` (default off); captions carry `mdns={bool}`.

**Track B — UI-driven `rooms_rejoin`** (`tests/e2e_local/rooms_rejoin.rs`,
`src/testing/click_window.rs`):
- Instances boot to the Menu (no `--lobby`), create/join/rejoin through
  real buttons via xdotool (`click_at_y`, `drive_center`, sweep ±30px
  around the measured first-button center y≈84px).
- Staggered spawns: instance-1 creates, 2/3 spawn later and gate on
  `directory listed rooms` (late-Get pull); rejoins re-pull the same way
  (`spawn_puller` respawns once on a missed pull).
- `CLICKER_NO_AUTOJOIN=1` guard in `resolve_room` (+ `auto_join.rs`
  pure helper) so menu-booted instances don't self-join.
- `join_room` now also switches `roster::Lobby` (previously Bevy roster
  stayed on the empty startup lobby, so menu-joined peers never spawned
  placeholders — zero accounting).
- Fresh `blackboard-test-<ts>` namespace per run (stale rooms pushed the
  create button off-screen).
- Checklist caption (`ui-create-room`, `ui-join-{2,3}`, `ui-rejoin-*`,
  `directory-pull-*`, …) + 240s recording → x4 → ~60s Telegram clip.
- `drive_center` replaces `drive_random` for the agreement drive
  (deterministic in-window clicks).
- Anchored window matching (`^clicker-N \[`) in `drive_random`,
  `tile_three`, `click_window`, `tile_one`: the old substring matched
  `clicker-xterm-N` first and drove the terminal.
- `clicker-log` spam fix: `poll_directory` no longer calls
  `into_inner()` (marks changed) without new data — this rebuilt the
  whole 70-node menu **every frame** (57 rebuilds/s, ~9k error lines/s
  in menu state). Now: identical directory → early return, rebuild only
  on real change; plus a `directory listed rooms` info log on change.

**Track D — matrix** (scaffolded, **not yet run**):
- `freenet:e2e-matrix` task + `e2e_matrix.sh` (4 cells × 2 suites,
  sequential): tcp-only / quic-only / both (mdns off) + lan-final
  (both, mdns on); per-cell `.local-run/matrix-*` dirs, markdown table,
  one Telegram text summary via new `send_text_msg` example.

## 2. Where the tests fail (evidence, `.local-run/rooms-rejoin-*`)

Six UI-era runs, all FAIL (no full pass):

| run (suffix) | shape |
|---|---|
| 083029 | UI create works; instance-2 never pulls room in 180s (directory split) |
| 083720 | create parsed only after ANSI fix; join-3 misfired onto CREATE (fractional coords on the 1080px window), stranding instance-3 in a stray room |
| 084339 | total isolation: p2p 0/0 on all instances, res=0, run died at converge |
| 084919 | **near miss**: full mesh, all owners everywhere, counts (1,16,16,33) — then baseline-agreement timeout (p1 under-driven: xterm-window mismatch, since fixed) |
| 085533 | isolation again (run died early) |
| 090321 | partial: inst-1 sees 2 remotes but converges only with one side (45 vs 60/60); inst-3 labels flap (res=21); 1↔2 never usable |

Converge gate (`tick + room + accounting + resolved>=2` on all 3) is the
consistent killer; exactly one instance is always starved.

## 3. Root causes (confirmed vs suspected)

1. **Freenet cold-start partition lottery (confirmed, dominant).**
   Fresh namespace per run ⇒ all instances race `Put` on empty keys ⇒
   disjoint singleton replica groups. Late-Get pulls sometimes heal it
   (084919), usually not. Runs split ~50/50 between "mesh forms" and
   "total isolation" with no code change in between. Staggering and the
   subscribe+re-put bridge do not reliably heal within run timeouts.
2. **Simultaneous-dial connect/disconnect storm (confirmed).**
   Instance-1 logs show 20–44 connects / 7–17 drops per run, often the
   same peer flapping within one second. Tie-break reduced but did not
   eliminate it; desperation was halved 60s→15s (uncommitted, unproven).
3. **Label flapping (suspected, still visible).** `res=21` on instance-3
   (090321) despite the no-relabel guard: repeated `label_slot` calls
   for the same slot (re-drains log every time). Attribution survives,
   but the log gate counts lines, not peers.
4. **Directory single-entry LWW flapping (known, deferred).** All
   instances publish their own entry under the same room key every 5s;
   whoever is newest wins, so rosters churn and redial constantly.
   Harmless but noisy; needs per-(room,peer) keys (contract change).
5. **Menu-rebuild log spam (fixed, verified).** 235k lines/25s → 370
   lines/25s in a local menu repro. Residual: ~6 one-time startup
   command errors (pre-existing emit_flash-style race).

## 4. Open questions / next steps

1. Re-run `rooms_rejoin` now that UI create/join, `roster::Lobby`
   switching, deterministic drive, and anchored window matching are all
   in — the last three runs each fixed one blocker; the current tree
   has never had a clean run.
2. Then launch the matrix (`freenet:e2e-matrix`, ~2–2.5h, overnight):
   expect tcp-only/quic-only/both cells to show the partition lottery
   and lan-final (mdns on) to show whether LAN discovery bypasses it.
3. If isolation persists: seed the first contact out-of-band for the
   test only (documented harness help, not prod behavior) OR give the
   directory bridge real teeth (periodic re-`Get`, not just subscribe).
4. Contract change (separate track): per-(room,peer) directory entries
   to end LWW flapping; per-peer label counting instead of line
   counting for the resolution gate.

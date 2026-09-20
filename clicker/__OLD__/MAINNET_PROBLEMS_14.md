# MAINNET PROBLEMS 14 — dev-profile debug (proven at binary level) + frozen-gate streak

Follow-up to `MAINNET_PROBLEMS_13.md`. The C1 slice (dev profile
with optimized deps) is implemented and its core mechanism is
proven; no full-green slow run this round (4 attempts). The blocker
is now a 3×-identical `create-frozen-1` failure needing its own TDD
slice.

## 1. C1 implemented — debug works, mechanism proven

- `Cargo.toml`: `[profile.dev.package."*"] opt-level = 3` (deps
  optimized, our code + `tracing` keep `debug_assertions` → static
  max TRACE).
- `src/testing/build_game.rs`: `CLICKER_BUILD_PROFILE` env (default
  `release`) replaces hardcoded `--release`; `dev` maps to
  `target/debug/`.
- `devenv.nix`: new `freenet:run-rooms-rejoin-dev` task (dev binary
  + `clicker=debug` filter); release task untouched.
- Proof (run `.local-run/rooms-rejoin-20260919-180456/`, fresh
  `/tmp/frt-build/debug/clicker`): the ``static max level is `info` ``
  warning is **gone** (0 occurrences) — the static cap is lifted in
  the dev binary. Zero code changes were needed for any `debug!`
  site; all 15 `sync:` + 3 `leave:` + gossip lines are now eligible.
- Side benefit: fast suite runs in ~12s instead of ~36s (optimized
  deps in dev); 371 passed, all lints clean.
- Still outstanding: decision *content* under real traffic. Every
  run so far died before drive phase, so `sync-1.log` (created
  correctly each run) is still empty and no `sync: entry merged`
  line has been observed in any instance log. The next green gate
  must assert both.

## 2. Slow runs this round (4 attempts, 0 green)

- `...-170811/`: creator setup, `directory connect failed: peer has
  not joined the network yet`. Bootstrap flake, untouched paths.
- `...-171734/`, `1924`-log run, `...-180456/` (dev): **identical**
  signature all three —
  `fresh joiner create-frozen-1 registered clicks while loading:
  [1, 1, 1, ...]` (`assert_slot_frozen`, hard `assert!` at
  `rooms_rejoin.rs:425`). Room creates fine, reveal + roster settle,
  then one stray gameplay click from the create-button interaction
  trips the frozen gate at ~33–110s. Always `tee` slow output to
  `/tmp/opencode/` — the message is only in nextest stdout.

## 3. Flake streak or regression? (unresolved, with evidence)

For regression (my slices did it): 3 consecutive identical
failures after the green run is a pattern, and the decision-logging
slice added per-event work in the frame loop.
Against: the menu→room transition is network-bound (directory-listed
→ reveal measured 16–18s in both green and failed runs — my µs of
`format!`+lock cannot shift it); nothing in the diff touches input,
BRP, clicking, or window layout; the frozen gate exists precisely
because this leak predates all current work (loading-gate commit
history). The leak needs only the BRP down/up to straddle the
transition — a timing coin flip the network decides.
Decisive experiment (not yet run): fast repro of the straddle —
menu-button down with `JoinPending` unset/clearing mid-click — then
see if it fails deterministically. That is the next RED either way.

## 4. Next RED (named)

`detect_click` drops clicks only while `JoinPending` is Some
(`detect_click.rs:22-24`, fast-covered). The creator's create click
can land with the gate open: down in menu, up in room (or
`detect_click` ordered before the menu system arms pending in the
same frame). Candidates: clear `just_pressed` on room enter; keep
pending armed until the first full in-room frame; ignore clicks
within N ms of transition. Separate slice — deliberately not
started here.

## 5. Standing rules carried forward

- Release gate stays the final oracle; dev-gate is for
  iteration/debugging (timing numbers indicative only).
- Never run a slow test twice without an intervening fast change —
  the gate is paused until the §4 repro lands.
- Do NOT re-attempt tracing feature flags while freenet pins
  `release_max_level_info` (most-restrictive-wins, proven in `_13`).

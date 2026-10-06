# lele_bevy_lint

A standalone CLI tool that enforces the Bevy-specific conventions of the
lele-syntax-rs skill. Split out of `lele_lint` because these two rules only
matter for projects that use Bevy — `lele_lint` itself stays generic and has
no knowledge of Bevy. Running `lele_bevy_lint` at all *is* "bevy mode"; there
is no separate flag or config toggle to enable it.

`lele_bevy_lint` depends on `lele_lint` as a path dependency and reuses its
`Checker`/`Project`/`Config`/`Diagnostic` types and CLI shape —
it is a second, independent set of checkers over the same `Project`
representation, not a fork.

## Rule Coverage

**bevy_export (E005) — bevy_systems/ not re-exported at domain root**
   - A domain may contain a `bevy_systems/` subfolder for Bevy system
     functions.
   - The domain's `mod.rs` declares `pub mod bevy_systems;` but does NOT
     `pub use` individual systems at the domain root.
   - `bevy_systems/mod.rs` must flatten via `pub use` so the consumer path
     is `domain::bevy_systems::system_name` (not
     `domain::bevy_systems::system_name::system_name`).

**bevy_folder (E008) — Bevy systems live in bevy_systems/ only**
   - Functions registered with `app.add_systems()` must live in
     `<domain>/bevy_systems/`.
   - A system function is identified by BOTH: its signature containing a
     parameter whose type's last path segment is `Res`, `ResMut`, `Query`,
     `Commands`, `MessageWriter`, or `MessageReader` (any parameter, not
     just the first), AND its ident appearing in an `add_systems(...)`
     call after the schedule argument (tuples and `.chain()` members
     included). Unregistered helpers that merely take `Commands`/`Query`
     (builders, click handlers) are exempt.
   - Registration is collected globally across `src/` and `methods/` (so a
     delegate such as `methods/<plugin>/build_plugin.rs` still counts), then
     every `pub fn` system definition is checked against it. `examples/` is
     exempt, and `#[cfg(test)]` registrations never count.
   - Blind spots, accepted and documented: systems passed via variables
     or function pointers instead of paths, and `add_systems` calls nested
     inside other `add_systems` arguments.

**bevy_ui (E029) — files that spawn UI must ship an ignored png preview**
   - A file counts as UI when a non-test spawn bundle it owns contains a
     UI visual: `Sprite`, `Text2d`, `Text`, `Mesh2d`, `MeshMaterial2d`,
     `Node`, or `ImageNode`. Cameras are not UI and no longer trigger this
     rule.
   - Reachability is computed over non-test items from the `main`/`build`/
     `setup` roots through direct calls, `add_systems` registrations and
     `impl` methods; `#[cfg(test)]` modules are skipped.
   - The file must define an `#[ignore]`d `*_ui_png_preview` test. If the
     file declares any `#[derive(Component)]` type, the preview must
     reference at least one of them, so a no-op shell cannot satisfy the
     rule.
   - The preview must route through `lele_bevy_preview::run(...)` — that is
     enforced separately by E039.
   - Deliberately strict-spawn: files that only mutate visuals
     (`Query<&mut Text2d>`) or build meshes without spawning stay exempt.

**bevy_ui_mp4 (E037) — UI driven over time or input needs an mp4 preview**
   - When a UI-spawning file also matches a time/input driver token
     (`Res<Time>`, `delta_secs`, `elapsed_secs`, `Timer`,
     `Animatable`, `AnimationClip`, `AnimationPlayer`, `tween`,
     `keyframe`, `Interaction`,
     `ButtonInput`, `MouseButton`, `KeyCode`), it must define an
     `#[ignore]`d `*_ui_mp4_preview` test.
   - Input-driven tokens are included deliberately: a purely discrete
     handler over UI is still expected to show its press/hover clip.

**bevy_plugin_scene (E038) — a UI-spawning Plugin needs a scene preview**
   - A file defining `impl Plugin for ...` whose own directory contains a
     file that spawns UI must define an `#[ignore]`d `*_ui_scene_preview` test
     in that same file.
   - Directory-scoped rather than call-graph-scoped: a crate-wide call graph
     keys callees by bare name and collides on common names like `build` and
     `setup`, which flagged P2P plugins as UI plugins. Same-directory scoping
     avoids flagging aggregate plugins that merely live above the UI folder;
     it is a presence rule — E029/E037 and the harness pixel gate cover
     per-file rendering.

**preview_routing (E039) — previews render through the harness**
   - Every `*_ui_png_preview`, `*_ui_mp4_preview` and `*_ui_scene_preview`
     test must call `lele_bevy_preview::run(...)`. A preview test that
     renders and asserts nothing is now a diagnostic instead of a
     convention someone must remember.

## Usage

```bash
# From a Bevy project's own directory, or via a sibling relative path:
cargo run --manifest-path ../lele_bevy_lint/Cargo.toml -- .
cargo run --manifest-path ../lele_bevy_lint/Cargo.toml -- --checker-list
cargo run --manifest-path ../lele_bevy_lint/Cargo.toml -- --explain E029

# Include example crates (UI often lives in examples/):
cargo run --manifest-path ../lele_bevy_lint/Cargo.toml -- --scan-folder src,examples

# Report the UI/preview gap map for the crate:
cargo run --manifest-path ../lele_bevy_lint/Cargo.toml -- --scan-folder src,examples --ui-inventory
```

`--ui-inventory` prints one row per file that has UI or previews, with its
marker components, visual idents, animation/input drivers, and whether a
png/mp4/scene preview exists. Every `x` is a gap the rules can act on.

## Configuration

`lele_bevy_lint` reads the same `lele.toml` as `lele_lint` for boundaries and
clippy allow-lists. Every Bevy rule is always on; there are no per-checker
toggles.

# lele_bevy_lint

A standalone CLI tool that enforces the Bevy-specific conventions of the
lele-syntax-rs skill. Split out of `lele_lint` because these two rules only
matter for projects that use Bevy — `lele_lint` itself stays generic and has
no knowledge of Bevy. Running `lele_bevy_lint` at all *is* "bevy mode"; there
is no separate flag or config toggle to enable it.

`lele_bevy_lint` depends on `lele_lint` as a path dependency and reuses its
`Checker`/`Project`/`Config`/`Diagnostic`/`Severity` types and CLI shape —
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
   - Blind spots, accepted and documented: systems passed via variables
     or function pointers instead of paths, and `add_systems` calls nested
     inside other `add_systems` arguments.

**bevy_ui (E029) — files that define UI must ship ignored preview tests**
   - A file counts as a UI definition when a visual spawn it owns
     (method named `spawn` with a bundle containing last path segment
     `Sprite`, `Text2d`, `Text`, `Mesh2d`, `MeshMaterial2d`, `Camera2d`,
     `Camera`, `Node`, or `ImageNode`) is reachable from production code.
     Reachability is computed over non-test items from the `main`/`build`/
     `setup` roots through direct calls and `add_systems` registrations;
     `#[cfg(test)]` modules are skipped, so `testing/` helpers used only by
     preview tests are excluded as a consequence (no path blocklist).
   - Kind is method-detected, never guessed: screenshot tokens
     (`Screenshot`, `save_to_disk`) require a `*_ui_png_preview` test;
     recorder tokens (`start_record_at`, `drive_cursor`, `place_window`,
     `x11grab`, `ffmpeg`) require a `*_ui_mp4_preview` test. One
     diagnostic per missing kind.
   - Each preview test must: end with `_ui_png_preview`/`_ui_mp4_preview`,
     carry `#[ignore]`, return `()` (libtest only accepts `()` or
     `Result<(), E>`), keep a preceding `assert!(<artifact>.exists())`,
     and end with `println!("PREVIEW_ARTIFACT={}", path.display())` as its
     last statement. The file must also reference `<file_stem>.png` /
     `<file_stem>.mp4` matching the kinds present.
   - Deliberately strict-spawn: files that only mutate visuals
     (`Query<&mut Text2d>`) or build meshes without spawning stay exempt.

## Usage

```bash
# From a Bevy project's own directory, or via a sibling relative path:
cargo run --manifest-path ../lele_bevy_lint/Cargo.toml -- .
cargo run --manifest-path ../lele_bevy_lint/Cargo.toml -- --checker-list
```

## Configuration (lele_lint.toml)

Reuses the same `lele_lint.toml` file and `[lele_lint.checkers]` toggles as
`lele_lint` (e.g. `bevy_folder = false` to disable one checker), read via
`lele_lint::config::Config::load`.

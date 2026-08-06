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
   - A system function is identified by its signature containing a
     parameter whose type's last path segment is `Res`, `ResMut`, `Query`,
     `Commands`, `MessageWriter`, or `MessageReader` (any parameter, not
     just the first).

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

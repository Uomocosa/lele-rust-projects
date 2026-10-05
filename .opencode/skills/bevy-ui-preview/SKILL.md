---
name: bevy-ui-preview
description: Use when a Bevy file spawns UI and needs a headless PNG/MP4 preview. Covers the lele_bevy_lint rules (E029/E037/E038/E039), the lele_bevy_preview scene DSL and test template, the change gate, and how to find preview gaps with --ui-inventory.
---

# Bevy UI previews

Every UI file must ship an ignored preview test that routes through
`lele_bevy_preview::run(...)`. This skill tells you where your UI lives, which
preview it needs, and how to write the test. The rules below are generated from
`lele_bevy_lint`, so they cannot drift from the linter.

## 1. Where is my UI defined?

Run the inventory for the crate:

```bash
cargo run --manifest-path ../lele_bevy_lint/Cargo.toml -- --scan-folder src,examples --ui-inventory
```

Each row is a file with UI. `markers` lists the components declared in that
file, `visual` the UI idents it spawns, `anim driver` the time/input signals it
reads, and `png`/`mp4`/`scene` whether a preview exists. Every `-` is a gap.

## 2. Which preview do I need?

- spawns UI (`Node`, `Text`, `ImageNode`, `Sprite`, `Text2d`, `Mesh2d`,
  `MeshMaterial2d`) -> an ignored `*_ui_png_preview`
- also reads a time/input driver (`Res<Time>`, `delta_secs`, `elapsed_secs`,
  `Timer`, `Local`, `Animatable`, `AnimationClip`, `AnimationPlayer`, `tween`,
  `keyframe`, `is_changed`, `Changed<Interaction>`, `Interaction`,
  `ButtonInput`, `MouseButton`, `KeyCode`) -> an ignored `*_ui_mp4_preview`
- defines `impl Plugin` and its domain spawns UI -> an ignored
  `*_ui_scene_preview`

Preview tests are `#[ignore]`d so the default test suite stays GPU-free; run
them with `--run-ignored all`.

## 3. The test template

Copy this into the file that owns the UI, next to the systems:

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use lele_bevy_preview::preview::Config;
    use lele_bevy_preview::run;
    use lele_bevy_preview::scene::Scene;
    use lele_bevy_preview::scene::State;

    fn out() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview")
    }

    fn build_scene(app: &mut App) {
        // spawn the real UI, or add the real plugin
    }

    fn empty_lobby(_world: &mut World) {}

    #[test]
    #[ignore = "headed preview"]
    fn spawn_root_ui_png_preview() {
        run(
            &Scene {
                name: String::from("spawn_root"),
                build: build_scene,
                states: vec![State {
                    label: String::from("empty lobby"),
                    apply: empty_lobby,
                }],
                timeline: None,
            },
            &Config { out_dir: out(), ..Config::default() },
            env!("CARGO_PKG_NAME"),
        )
        .expect("preview");
    }
}
```

## 4. The scene DSL

- `State` = one PNG; `apply: fn(&mut World)` mutates the world before capture.
- `Timeline` = N frames -> one MP4; `frames`, `fps`, `apply`.
- `build: fn(&mut App)` assembles the scene with typed code — no reflection, no JSON.
- `Config`: `width`, `height`, `warmup_frames`, `max_capture_frames`,
  `min_distinct_colors`, `out_dir`.

`run` renders every state, encodes the timeline if present, and fails with
`Error::NoVisual` when a frame has fewer than `min_distinct_colors` distinct
colours — a blank frame is an error, not a green test.

## 5. The change gate

`run` hashes every frame and persists a manifest. On a re-run, `FirstRun` /
`New` / `Changed` are sendable and `Same` is not. Telegram delivery fires only
for sendable artifacts when `TELEGRAM_BOT_TOKEN` / `TELEGRAM_CHAT_ID` are set;
missing credentials are not an error.

## 6. Run commands

```bash
devenv tasks run lele:bevy-lint 2>&1    # the rules
devenv tasks run lele:nextest 2>&1
cargo nextest run ui_png_preview ui_mp4_preview ui_scene_preview --run-ignored all -- --nocapture
```

## Rules

### E005 `bevy_export`

A domain declares `pub mod bevy_systems;` but never re-exports its systems at the domain root.

**Why:** Consumers reach systems as `domain::bevy_systems::system`, so the folder is always visible in the path.

**Bad** (reports E005):

`src/inventory/mod.rs`

```rust
pub mod bevy_systems;

pub use bevy_systems::poll_inv;
```

`src/inventory/bevy_systems/mod.rs`

```rust
pub mod poll_inv;
pub use poll_inv::poll_inv;
```

**Good:**

`src/inventory/mod.rs`

```rust
pub mod bevy_systems;
```

`src/inventory/bevy_systems/mod.rs`

```rust
pub mod poll_inv;
pub use poll_inv::poll_inv;
```


### E008 `bevy_folder`

A `pub fn` registered with `app.add_systems()` whose parameters read Bevy system types must live in the domain's `bevy_systems/` folder.

**Why:** Systems are the domain's Bevy edge; keeping them in one folder makes the plugin surface readable.

**Bad** (reports E008):

`src/enemy/tick_enemies.rs`

```rust
pub struct Query;

pub fn tick_enemies(_q: Query) {}

pub fn register_enemies() {
    let app = App;
    app.add_systems(Update, tick_enemies);
}
```

**Good:**

`src/enemy/mod.rs`

```rust
pub mod bevy_systems;
```

`src/enemy/bevy_systems/mod.rs`

```rust
pub mod tick_enemies;
pub use tick_enemies::tick_enemies;
```

`src/enemy/bevy_systems/tick_enemies.rs`

```rust
pub struct Query;

pub fn tick_enemies(_q: Query) {}
```


### E029 `bevy_ui`

A file whose visual spawn is reachable from production must ship an ignored `*_ui_png_preview` (and, when it drives a recorder, a `*_ui_mp4_preview`) test ending with `assert!(exists)` plus `println!("PREVIEW_ARTIFACT=...")`.

**Why:** UI that is never rendered cannot be reviewed; the named preview test is the contract that forces an artifact for every production visual.

**Bad** (reports E029):

`src/discovery/ui/spawn_root.rs`

```rust
pub struct Sprite;
pub struct Spawner;

pub fn setup(spawner: &mut Spawner) {
    spawner.spawn((Sprite,));
}
```

**Good:**

`src/discovery/ui/spawn_root.rs`

```rust
pub struct Sprite;
pub struct Spawner;

pub fn setup(spawner: &mut Spawner) {
    spawner.spawn((Sprite,));
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "headed preview"]
    fn spawn_root_ui_png_preview() {
        let shot = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spawn_root.png");
        assert!(shot.exists());
        println!("PREVIEW_ARTIFACT={}", shot.display());
    }
}
```


### E037 `bevy_ui_mp4`

A file that spawns UI reachable from production and drives it over time or input must ship an ignored `*_ui_mp4_preview` test.

**Why:** Static frames cannot show motion, hover or press states; a time/input-driven UI needs a recorded clip to be reviewable.

**Bad** (reports E037):

`src/discovery/ui/sync_room_list.rs`

```rust
pub fn setup(s: &mut S) { s.spawn((Node,)); }

pub fn tick(time: Res<Time>, q: Query<&Interaction>) {
    let _ = time.delta_secs();
    if q.is_changed() {}
}
```

**Good:**

`src/discovery/ui/sync_room_list.rs`

```rust
pub fn setup(s: &mut S) { s.spawn((Node,)); }

pub fn tick(time: Res<Time>) { let _ = time.delta_secs(); }

#[cfg(test)]
mod tests {
    use lele_bevy_preview::{run, scene::Scene};

    #[test]
    #[ignore = "headed recording"]
    fn sync_room_list_ui_mp4_preview() {
        let scene = Scene { name: String::from("sync_room_list") };
        let _ = run(&scene, &Config::default(), "x");
    }
}
```


### E038 `bevy_plugin_scene`

A file defining a `Plugin` whose build spawns UI must ship an ignored `*_ui_scene_preview` test.

**Why:** A plugin is the assembly point for a whole screen; its scene preview proves the assembled screen renders.

**Bad** (reports E038):

`src/discovery/ui/default_ui_plugin.rs`

```rust
pub fn spawn_ui(s: &mut S) { s.spawn((Node,)); }

pub struct DefaultUiPlugin;

impl Plugin for DefaultUiPlugin {
    fn build(&self, app: &mut A) { spawn_ui(app); }
}
```

**Good:**

`src/discovery/ui/default_ui_plugin.rs`

```rust
pub fn spawn_ui(s: &mut S) { s.spawn((Node,)); }

pub struct DefaultUiPlugin;

impl Plugin for DefaultUiPlugin {
    fn build(&self, app: &mut A) { spawn_ui(app); }
}

#[cfg(test)]
mod tests {
    use lele_bevy_preview::{run, scene::Scene};

    #[test]
    #[ignore = "headed scene"]
    fn default_ui_plugin_ui_scene_preview() {
        let scene = Scene { name: String::from("default_ui_plugin") };
        let _ = run(&scene, &Config::default(), "x");
    }
}
```


### E039 `preview_routing`

Every `*_ui_png_preview`, `*_ui_mp4_preview` and `*_ui_scene_preview` test must call `lele_bevy_preview::run(...)`.

**Why:** A preview test that renders and asserts nothing can pass while producing a blank frame; routing through the harness makes empty frames fail.

**Bad** (reports E039):

`src/discovery/ui/spawn_root.rs`

```rust
pub fn setup(s: &mut S) { s.spawn((Node,)); }

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "headed"]
    fn spawn_root_ui_png_preview() {
        let shot = std::path::PathBuf::from("spawn_root.png");
        assert!(shot.exists());
    }
}
```

**Good:**

`src/discovery/ui/spawn_root.rs`

```rust
pub fn setup(s: &mut S) { s.spawn((Node,)); }

#[cfg(test)]
mod tests {
    use lele_bevy_preview::{run, scene::Scene};

    #[test]
    #[ignore = "headed"]
    fn spawn_root_ui_png_preview() {
        let scene = Scene { name: String::from("spawn_root") };
        let _ = run(&scene, &Config::default(), "x");
    }
}
```



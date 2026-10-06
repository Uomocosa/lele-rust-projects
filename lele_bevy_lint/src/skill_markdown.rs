use std::fmt::Write;

use lele_lint::render_rule;

use crate::checkers;

const FRONTMATTER: &str = r"---
name: bevy-ui-preview
description: Use when a Bevy file spawns UI and needs a headless PNG/MP4 preview. Covers the lele_bevy_lint rules (E029/E037/E038/E039), the lele_bevy_preview scene DSL and test template, the change gate, and how to find preview gaps with --ui-inventory.
---

";

const BODY: &str = r#"# Bevy UI previews

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
  `Timer`, `Animatable`, `AnimationClip`, `AnimationPlayer`, `tween`,
  `keyframe`, `Interaction`,
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
    use lele_bevy_preview::scene::Kind;
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
                kind: Kind::System,
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

- `Kind` = `System` | `Component` | `Plugin` | `App`; required, it prefixes the Telegram
  message (`[system] spawn_root`).
- `State` = one PNG; `apply: fn(&mut World)` mutates the world before capture.
- `Timeline` = N frames -> one MP4; `frames`, `fps`, `apply`. `Config::lead_in_frames`
  baseline frames are captured before `apply`, and a clip whose frames are all
  identical fails with `Error::StaticClip`.
- A change that lands in one tick (`is_changed`, `Changed<T>`, `Local`) is not motion:
  show it as two `State`s (before, after) instead of an mp4.
- `build: fn(&mut App)` assembles the scene with typed code — no reflection, no JSON.
- `Config`: `width`, `height`, `warmup_frames`, `lead_in_frames`, `max_capture_frames`,
  `min_distinct_colors`, `out_dir`.

`run` renders every state, encodes the timeline if present, and fails with
`Error::NoVisual` when a frame has fewer than `min_distinct_colors` distinct
colours — a blank frame is an error, not a green test.

## 5. The change gate

`run` hashes every frame and persists a manifest. On a re-run, `FirstRun` /
`New` / `Changed` are sendable and `Same` is not. Telegram delivery fires only
for sendable artifacts when `TELEGRAM_BOT_TOKEN` / `TELEGRAM_CHAT_ID` are set;
missing credentials are not an error. When anything in a scene is sendable, the
whole scene (every state PNG plus the clip) goes out as one Telegram album with a
single numbered caption, so before/after images stay together.

## 6. Run commands

```bash
devenv tasks run lele:bevy-lint 2>&1    # the rules
devenv tasks run lele:nextest 2>&1
cargo nextest run ui_png_preview ui_mp4_preview ui_scene_preview --run-ignored all -- --nocapture
```

## Rules

"#;

#[must_use]
pub fn skill_markdown() -> String {
    let mut out = String::from(FRONTMATTER);
    out.push_str(BODY);
    for checker in checkers::build_checkers() {
        out.push_str(&render_rule(checker.as_ref()));
        let _ = writeln!(out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::skill_markdown;

    #[test]
    fn test_usage() {
        let text = skill_markdown();
        assert!(text.starts_with("---"));
        assert!(text.contains("name: bevy-ui-preview"));
        assert!(text.contains("### E029 `bevy_ui`"));
        assert!(text.contains("### E039 `preview_routing`"));
    }
}

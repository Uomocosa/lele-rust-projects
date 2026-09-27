# Lele Code Viewer — UI Style Spec

This is the look-and-feel spec for **Lele Code Viewer** (the app formerly known
as `lele_docs`). It is written from the user's description; it describes intent,
not implementation. When in doubt, match this document, not the current code.

## 1. Identity

- App name shown to the user: **Lele Code Viewer**.
- Subtitle / one-liner: *mobile-first, read-only Rust source, file-tree and
  dependency-tree viewer*.
- Audience: the developer's phone and desktop, self-hosted over Tailscale.
- Read-only: the viewer never edits a project. It only shows it.

## 2. Overall Layout

```
┌───────────────────────────────────────────────────────────┐
│ ☰   Lele Code Viewer            [ search… ]               │  <- top bar
├───────────────────────────────────────────────────────────┤
│                                                           │
│                     main content                          │
│        (source file / item / markdown / a tree)           │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

- A single sticky **top bar**, always visible.
- Left: a **hamburger button** (`☰`) — the primary navigation control.
- Centre: the current project name, linking home.
- Right: the search box.
- Below: one scrollable main content area.
- The **side menu** is a drawer that slides in from the left, over or beside
  the content, and is dismissed by tapping the hamburger again, tapping the
  scrim, or pressing `Esc`.

## 3. The Hamburger Side Menu (Drawer)

Clicking the hamburger opens a left drawer containing, top to bottom:

1. **Project switcher** — a searchable/filterable list of *every* Rust project
   found on this PC (any directory containing a `Cargo.toml`, skipping build
   and VCS directories). The active project is highlighted. Selecting a project
   switches the whole viewer to it.
2. **View switcher** — exactly **two large icons**, side by side or stacked:
   - 🗂 **Files** — the file tree.
   - 🕸 **Dependencies** — the dependency tree.
   The currently active view is highlighted.
3. A muted footer with the crate name / path of the active project.

The drawer is the *only* place project switching and view switching happen. The
main content area never grows a second navigation metaphor.

## 4. View: File Tree

The **Files** icon shows the project as a plain filesystem tree — the same
shape a file manager would show:

- Directories first, then files, alphabetically within each directory.
- Folders expand/collapse; the root starts expanded down to the first level.
- Every path is shown exactly as on disk; directories end in `/`.
- Build and VCS noise (`target/`, `.git/`, `.devenv/`, `node_modules/`) is
  hidden.
- Clicking a `.rs` file opens its source page; clicking a `.md` file opens the
  rendered markdown; other files are listed but inert.
- Monospace path text, subtle connectors or indentation so nesting is obvious
  on a phone.

## 5. View: Dependency Tree

The **Dependencies** icon shows an internal **module dependency graph**, layered
**bottom = base, top = most important / most code**:

- Nodes are the crate's own modules (e.g. `index`, `server::router`).
- Base layer (bottom): modules that depend on **nothing internal** — or only on
  **outside crates** (std, axum, serde, …). External crates appear as a muted
  foundation strip beneath the base modules.
- Higher layers: modules that depend on more internal modules — the "more
  important / more code" aggregating layers.
- Edges link a module to the modules it depends on (`A → B` means *A uses B*),
  drawn so the direction of dependency reads clearly.
- Layering rule: a module with no internal dependencies sits at level 0; every
  other module sits one level above the highest level among its internal
  dependencies.
- Nodes expand/collapse so a large graph stays navigable on a phone.

A flawed-but-pretty rendering is acceptable; correctness of the *layers*
(base at bottom, aggregators at top) matters more than pixel perfection.

## 6. Visual Language

- Dark, GitHub-ish palette (already the app baseline): dark page, slightly
  lighter panels, blue accent, muted grey secondary text, thin borders.
- Monospace (`ui-monospace`, Menlo, SF Mono) for paths, signatures and tree
  structure; the system UI sans for prose and headings.
- Rounded corners (~8–10px) on panels, chips, buttons and tree nodes.
- The two view icons are large, tappable targets (mobile-first), with a clear
  active state.
- Everything must be usable one-handed on a narrow phone screen: no horizontal
  scrolling for navigation, big hit areas, the drawer full-height.

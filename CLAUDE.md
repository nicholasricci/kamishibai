# Kamishibai

AI-first, open source game engine for 2.5HD games (pixel-art billboard sprites
in lit 3D dioramas), built as a library on top of Bevy 0.20.

## Layout

- `src/edit.rs`: `EditCommand` trait, `EditHistory` resource, `WorldEditExt` /
  `CommandsEditExt` for undoable edits. All project changes go through here.
- `src/billboard.rs`: `Billboard` component and the system that faces the camera.
- `src/sprite.rs`: `pixel_sprite` helper (lit, alpha-masked, feet at origin).
- `src/camera.rs`: `DioramaCamera` + `CameraTarget` follow system.
- `src/lib.rs`: `KamishibaiPlugin` and `prelude`.
- `examples/diorama.rs`: playable demo with procedurally generated pixel art.

## Rules

- Bevy APIs change between releases: check the source in `~/.cargo/registry`
  rather than relying on memory (e.g. `shadow_maps_enabled`, `FontSize::Px`).
- Any user-visible change to the world must be an `EditCommand`.
- Logic that doesn't need a GPU gets a unit test.
- Before committing: `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

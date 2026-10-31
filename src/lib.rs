//! # Kamishibai
//!
//! An open source, AI-first game engine for 2.5HD games, built on
//! [Bevy](https://bevy.org): pixel-art sprites in lit 3D dioramas.
//!
//! This crate is at a very early stage. Today it provides:
//!
//! - [`billboard`]: sprites that turn toward the camera;
//! - [`sprite`]: helpers to place lit, shadow-casting pixel-art sprites;
//! - [`camera`]: a diorama camera that follows a target;
//! - [`edit`]: undoable edit commands, shared by the editor, scripts and AI.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use kamishibai::prelude::*;
//!
//! App::new()
//!     .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()), KamishibaiPlugin))
//!     .run();
//! ```

pub mod billboard;
pub mod camera;
pub mod edit;
pub mod sprite;

use bevy::prelude::*;
use bevy::transform::TransformSystems;

/// Everything you usually need, in one import.
pub mod prelude {
    pub use crate::KamishibaiPlugin;
    pub use crate::billboard::Billboard;
    pub use crate::camera::{CameraTarget, DioramaCamera};
    pub use crate::edit::{
        CommandsEditExt, EditCommand, EditHistory, SetName, SetTransform, WorldEditExt,
    };
    pub use crate::sprite::{DEFAULT_PIXELS_PER_UNIT, pixel_sprite};
}

/// Adds Kamishibai's core systems and resources to a Bevy app.
pub struct KamishibaiPlugin;

impl Plugin for KamishibaiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<edit::EditHistory>()
            .register_type::<billboard::Billboard>()
            .register_type::<camera::DioramaCamera>()
            .register_type::<camera::CameraTarget>()
            .add_systems(
                PostUpdate,
                (camera::follow_target, billboard::face_camera)
                    .chain()
                    .before(TransformSystems::Propagate),
            );
    }
}

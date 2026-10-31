//! The diorama camera: a narrow-angle perspective camera that smoothly
//! follows a target from a fixed offset, giving the miniature-set look.

use bevy::prelude::*;

/// A camera that follows the entity marked with [`CameraTarget`].
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
#[require(Camera3d)]
pub struct DioramaCamera {
    /// Position of the camera relative to the target.
    pub offset: Vec3,
    /// How quickly the camera catches up with the target (higher = snappier).
    pub follow_speed: f32,
}

impl Default for DioramaCamera {
    fn default() -> Self {
        Self {
            offset: Vec3::new(0.0, 11.0, 20.0),
            follow_speed: 6.0,
        }
    }
}

impl DioramaCamera {
    /// Vertical field of view (in radians) suggested for the diorama look:
    /// narrow enough to flatten perspective without becoming orthographic.
    pub const SUGGESTED_FOV: f32 = 25.0_f32.to_radians();

    /// Components for a ready-to-use diorama camera looking at `focus`.
    pub fn bundle(self, focus: Vec3) -> impl Bundle {
        (
            self,
            Projection::Perspective(PerspectiveProjection {
                fov: Self::SUGGESTED_FOV,
                ..default()
            }),
            Transform::from_translation(focus + self.offset).looking_at(focus, Vec3::Y),
        )
    }
}

/// Marks the entity the [`DioramaCamera`] follows.
#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component, Default)]
pub struct CameraTarget;

pub(crate) fn follow_target(
    time: Res<Time>,
    target: Query<&Transform, (With<CameraTarget>, Without<DioramaCamera>)>,
    mut cameras: Query<(&DioramaCamera, &mut Transform)>,
) {
    let Some(target) = target.iter().next() else {
        return;
    };
    let focus = target.translation;
    for (camera, mut transform) in &mut cameras {
        let desired = focus + camera.offset;
        let t = 1.0 - (-camera.follow_speed * time.delta_secs()).exp();
        transform.translation = transform.translation.lerp(desired, t);
        // Keep a constant viewing angle, like a camera on a rail.
        transform.look_to(-camera.offset, Vec3::Y);
    }
}

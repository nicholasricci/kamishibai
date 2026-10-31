//! Billboards: flat sprites that always turn toward the camera.
//!
//! This is the core trick of the 2.5HD look: characters and props are 2D pixel
//! art drawn on quads placed in a real 3D world, lit and shadowed like
//! everything else.

use bevy::prelude::*;

/// Makes an entity rotate every frame to face the active 3D camera.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[reflect(Component, Default)]
pub enum Billboard {
    /// Rotates only around the vertical (Y) axis, so sprites stay upright.
    /// This is the classic 2.5HD look.
    #[default]
    Cylindrical,
    /// Turns fully toward the camera, including tilting up and down.
    Spherical,
}

/// Rotation that makes a quad facing +Z look like it faces a camera with
/// the given rotation.
pub fn billboard_rotation(mode: Billboard, camera_rotation: Quat) -> Quat {
    match mode {
        Billboard::Spherical => camera_rotation,
        Billboard::Cylindrical => {
            // Direction from the scene back toward the camera, flattened on XZ.
            let back = camera_rotation * Vec3::Z;
            if back.x.abs() < f32::EPSILON && back.z.abs() < f32::EPSILON {
                // Camera looking straight down or up: keep the current yaw undefined
                // and fall back to facing +Z.
                Quat::IDENTITY
            } else {
                Quat::from_rotation_y(back.x.atan2(back.z))
            }
        }
    }
}

pub(crate) fn face_camera(
    cameras: Query<&GlobalTransform, (With<Camera3d>, Without<Billboard>)>,
    mut billboards: Query<(&Billboard, &mut Transform)>,
) {
    // With several cameras the first one wins; a split-screen setup
    // would need per-camera rendering, which is out of scope for now.
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    let camera_rotation = camera.compute_transform().rotation;
    for (mode, mut transform) in &mut billboards {
        transform.rotation = billboard_rotation(*mode, camera_rotation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_vec_near(a: Vec3, b: Vec3) {
        assert!(a.distance(b) < 1e-5, "{a} != {b}");
    }

    #[test]
    fn cylindrical_faces_camera_and_stays_upright() {
        // Camera behind +Z, tilted down 30 degrees, looking at the origin.
        let camera = Transform::from_xyz(0.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y);
        let rotation = billboard_rotation(Billboard::Cylindrical, camera.rotation);

        assert_vec_near(rotation * Vec3::Y, Vec3::Y);
        assert_vec_near(rotation * Vec3::Z, Vec3::Z);
    }

    #[test]
    fn cylindrical_follows_camera_yaw() {
        let camera = Transform::from_xyz(8.0, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y);
        let rotation = billboard_rotation(Billboard::Cylindrical, camera.rotation);

        assert_vec_near(rotation * Vec3::Z, Vec3::X);
    }

    #[test]
    fn spherical_matches_camera() {
        let camera = Transform::from_xyz(3.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y);
        let rotation = billboard_rotation(Billboard::Spherical, camera.rotation);

        assert_vec_near(rotation * Vec3::Z, camera.rotation * Vec3::Z);
    }
}

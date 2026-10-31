//! Pixel-art sprites living in the 3D world.

use bevy::prelude::*;

use crate::billboard::Billboard;

/// How big a sprite pixel is in world units. 16 pixels per unit means a
/// 16x24 character is 1 unit wide and 1.5 units tall.
pub const DEFAULT_PIXELS_PER_UNIT: f32 = 16.0;

/// Builds the components for a billboarded pixel-art sprite whose bottom edge
/// sits on the entity's origin (so `Transform::from_xyz(x, 0.0, z)` puts its
/// feet on the ground).
///
/// The sprite is lit, casts shadows and uses alpha cut-out, so transparent
/// pixels let light through. Use
/// [`ImagePlugin::default_nearest`](bevy::image::ImagePlugin::default_nearest)
/// to keep the pixels crisp.
pub fn pixel_sprite(
    image: Handle<Image>,
    size_in_pixels: UVec2,
    pixels_per_unit: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> impl Bundle {
    let size = size_in_pixels.as_vec2() / pixels_per_unit;
    let mesh = Mesh::from(Rectangle::new(size.x, size.y)).translated_by(Vec3::Y * size.y / 2.0);
    let material = StandardMaterial {
        base_color_texture: Some(image),
        alpha_mode: AlphaMode::Mask(0.5),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        double_sided: true,
        cull_mode: None,
        ..default()
    };
    (
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        Billboard::Cylindrical,
    )
}

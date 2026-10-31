//! A tiny 2.5HD diorama: a pixel-art hero walking in a lit 3D scene.
//!
//! Run with: `cargo run --example diorama`
//!
//! Controls:
//! - WASD / arrow keys: walk
//! - Space: place a stone block (an undoable edit command)
//! - Z: undo, Y: redo

use bevy::asset::RenderAssetUsages;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use kamishibai::prelude::*;

const HERO: [&str; 24] = [
    "................",
    ".....kkkkkk.....",
    "....khhhhhhk....",
    "...khhhhhhhhk...",
    "...khhhhhhhhk...",
    "...khssssshhk...",
    "...ksskssksk....",
    "...ksskssksk....",
    "....kssssssk....",
    ".....kssssk.....",
    "....kkrrrrkk....",
    "...krrrrrrrrk...",
    "..ksrrryyrrrsk..",
    "..ksrrryyrrrsk..",
    "..kskrrrrrrksk..",
    "...k.krrrrk.k...",
    ".....kbbbbk.....",
    ".....kbbbbk.....",
    ".....kbkkbk.....",
    ".....kbk.kbk....",
    "....kbbk.kbbk...",
    "....kkkk.kkkk...",
    "................",
    "................",
];

const TREE: [&str; 24] = [
    "......kkkk......",
    "....kkggggkk....",
    "...kggGGggggk...",
    "..kggGGggggggk..",
    "..kgGGgggggdgk..",
    ".kgggggggggddgk.",
    ".kggGGgggggdggk.",
    ".kgGGgggggggdgk.",
    ".kgggggggddgggk.",
    "..kggggggddggk..",
    "..kgggGGgggggk..",
    "...kggGgggddk...",
    "....kkggggkk....",
    "......kttk......",
    "......kttk......",
    "......kttk......",
    "......kttk......",
    ".....ktttTk.....",
    ".....kttttk.....",
    "....ktttttTk....",
    "................",
    "................",
    "................",
    "................",
];

fn palette(c: char) -> [u8; 4] {
    match c {
        'k' => [34, 28, 40, 255],    // outline
        'h' => [92, 58, 38, 255],    // hair
        's' => [240, 196, 160, 255], // skin
        'r' => [176, 48, 56, 255],   // tunic
        'y' => [232, 192, 72, 255],  // belt buckle
        'b' => [70, 62, 90, 255],    // boots
        'g' => [72, 140, 72, 255],   // leaves
        'G' => [120, 184, 96, 255],  // leaves, lit
        'd' => [44, 96, 60, 255],    // leaves, shade
        't' => [110, 74, 50, 255],   // trunk
        'T' => [80, 52, 36, 255],    // trunk, shade
        _ => [0, 0, 0, 0],
    }
}

/// Turns ASCII art into a texture, so this example needs no asset files.
fn image_from_art(art: &[&str]) -> Image {
    let width = art[0].len() as u32;
    let height = art.len() as u32;
    let data = art
        .iter()
        .flat_map(|row| row.chars().flat_map(palette))
        .collect();
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

#[derive(Component)]
struct Hero;

#[derive(Component)]
struct HelpText;

#[derive(Resource)]
struct BlockAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Kamishibai - diorama".into(),
                        ..default()
                    }),
                    ..default()
                }),
            KamishibaiPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.55, 0.70, 0.85)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.75, 0.80, 1.0),
            brightness: 300.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (move_hero, place_blocks, undo_redo, update_help))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let matte = |color: Color| StandardMaterial {
        base_color: color,
        perceptual_roughness: 1.0,
        ..default()
    };

    // Ground.
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(24.0, 24.0))),
        MeshMaterial3d(materials.add(matte(Color::srgb(0.42, 0.62, 0.36)))),
    ));

    // A few blocks to give the scene some depth.
    let block = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let stone = materials.add(matte(Color::srgb(0.62, 0.60, 0.58)));
    let wood = materials.add(matte(Color::srgb(0.55, 0.38, 0.24)));
    for (x, y, z, material) in [
        (-4.0, 0.5, -3.0, &stone),
        (-3.0, 0.5, -3.0, &stone),
        (-3.0, 1.5, -3.0, &stone),
        (3.0, 0.5, -1.0, &wood),
        (3.0, 0.5, 0.0, &wood),
        (5.0, 0.5, 3.0, &stone),
    ] {
        commands.spawn((
            Name::new("Block"),
            Mesh3d(block.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(x, y, z),
        ));
    }
    commands.insert_resource(BlockAssets {
        mesh: block,
        material: stone,
    });

    // Trees: billboard sprites, lit and casting shadows like 3D objects.
    let tree = images.add(image_from_art(&TREE));
    for (x, z) in [
        (-6.0, -6.0),
        (-1.5, -7.0),
        (6.0, -5.0),
        (-7.0, 2.0),
        (7.5, 1.0),
    ] {
        commands.spawn((
            Name::new("Tree"),
            pixel_sprite(
                tree.clone(),
                UVec2::new(16, 24),
                8.0,
                &mut meshes,
                &mut materials,
            ),
            Transform::from_xyz(x, 0.0, z),
        ));
    }

    // The hero.
    let hero = images.add(image_from_art(&HERO));
    commands.spawn((
        Name::new("Hero"),
        Hero,
        CameraTarget,
        pixel_sprite(
            hero,
            UVec2::new(16, 24),
            DEFAULT_PIXELS_PER_UNIT,
            &mut meshes,
            &mut materials,
        ),
        Transform::from_xyz(0.0, 0.0, 2.0),
    ));

    // Warm late-afternoon sun with soft shadows.
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.92, 0.80),
            illuminance: 9_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-6.0, 10.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            maximum_distance: 40.0,
            ..default()
        }
        .build(),
    ));

    // The diorama camera, with a bit of haze in the distance.
    commands.spawn((
        DioramaCamera::default().bundle(Vec3::new(0.0, 0.0, 2.0)),
        DistanceFog {
            color: Color::srgb(0.55, 0.70, 0.85),
            falloff: FogFalloff::Linear {
                start: 22.0,
                end: 45.0,
            },
            ..default()
        },
    ));

    commands.spawn((
        HelpText,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn move_hero(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut hero: Query<&mut Transform, With<Hero>>,
) {
    let Ok(mut transform) = hero.single_mut() else {
        return;
    };
    let mut direction = Vec3::ZERO;
    if keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        direction.z -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        direction.z += 1.0;
    }
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        direction.x -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        direction.x += 1.0;
    }
    let speed = 4.0;
    transform.translation += direction.normalize_or_zero() * speed * time.delta_secs();
    transform.translation = transform
        .translation
        .clamp(Vec3::splat(-11.0), Vec3::splat(11.0));

    // Face the walking direction by mirroring the sprite.
    if direction.x != 0.0 {
        transform.scale.x = direction.x.signum();
    }
}

/// Places a block in front of the hero, as an undoable command.
struct PlaceBlock {
    position: Vec3,
    entity: Option<Entity>,
}

impl EditCommand for PlaceBlock {
    fn label(&self) -> String {
        format!(
            "Place block at ({:.0}, {:.0})",
            self.position.x, self.position.z
        )
    }

    fn apply(&mut self, world: &mut World) {
        let assets = world.resource::<BlockAssets>();
        let (mesh, material) = (assets.mesh.clone(), assets.material.clone());
        let entity = world
            .spawn((
                Name::new("Placed block"),
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(self.position),
            ))
            .id();
        self.entity = Some(entity);
    }

    fn undo(&mut self, world: &mut World) {
        if let Some(entity) = self.entity.take() {
            world.despawn(entity);
        }
    }
}

fn place_blocks(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    hero: Query<&Transform, With<Hero>>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    let Ok(hero) = hero.single() else {
        return;
    };
    let position =
        (hero.translation + Vec3::new(hero.scale.x * 1.5, 0.0, 0.0)).round() + Vec3::Y * 0.5;
    commands.edit(PlaceBlock {
        position,
        entity: None,
    });
}

fn undo_redo(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyZ) {
        commands.undo_edit();
    }
    if keys.just_pressed(KeyCode::KeyY) {
        commands.redo_edit();
    }
}

fn update_help(history: Res<EditHistory>, mut text: Query<&mut Text, With<HelpText>>) {
    if !history.is_changed() && !history.is_added() {
        return;
    }
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let last = history
        .undo_labels()
        .last()
        .cloned()
        .unwrap_or_else(|| "-".into());
    text.0 = format!(
        "WASD / frecce: cammina\nSpazio: piazza un blocco\nZ: annulla   Y: ripristina\n\nUltimo comando: {last}\nAnnullabili: {}   Ripristinabili: {}",
        history.undo_labels().len(),
        history.redo_labels().len(),
    );
}

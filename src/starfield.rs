use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use crate::Player;

pub struct StarfieldPlugin;
#[derive(Component)]
struct Starfield;

impl Plugin for StarfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<StarfieldMaterial>::default())
            .add_startup_system(setup)
            .add_system(move_starfield);
    }
}

fn setup(
    mut commands: Commands,
    mut mat2d: ResMut<Assets<StarfieldMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // TODO size of screen is hardcoded
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(1280., 720.)).into())
                .into(),
            material: mat2d.add(StarfieldMaterial {
                pos: Vec2::new(0., 0.),
            }),
            transform: Transform::from_translation(Vec3::new(0., 0., crate::layer::BACKGROUND)),
            ..default()
        })
        .insert(Starfield);
}

fn move_starfield(
    query: Query<&Transform, With<Player>>,
    mut starfield_query: Query<&mut Transform, (With<Starfield>, Without<Player>)>,
    mut materials: ResMut<Assets<StarfieldMaterial>>,
) {
    let player = query.single();

    for mat in materials.iter_mut() {
        mat.1.pos = player.translation.truncate();
    }

    let mut starfield = starfield_query.single_mut();
    starfield.translation.x = player.translation.x;
    starfield.translation.y = player.translation.y;
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material2d for StarfieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/starfield.wgsl".into()
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "721097c0-7368-453f-a95f-0731d6724689"]
pub struct StarfieldMaterial {
    #[uniform(0)]
    pub pos: Vec2,
}

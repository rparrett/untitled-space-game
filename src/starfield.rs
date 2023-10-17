use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    window::PrimaryWindow,
};
use interpolation::Ease;

use crate::{warp_node::WarpAnimation, DespawnOnRestart, GameState, Player, Velocity};

pub struct StarfieldPlugin;
#[derive(Component)]
struct Starfield;

impl Plugin for StarfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<StarfieldMaterial>::default());
        app.add_system(setup.in_schedule(OnEnter(GameState::Playing)));
        app.add_system(move_starfield.in_set(OnUpdate(GameState::Playing)));
        app.add_system(warp_animation.in_set(OnUpdate(GameState::Warping)));
    }
}

fn setup(
    mut commands: Commands,
    mut mat2d: ResMut<Assets<StarfieldMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(Vec2::new(window.width(), window.height())).into())
                .into(),
            material: mat2d.add(StarfieldMaterial::default()),
            transform: Transform::from_translation(Vec3::new(0., 0., crate::layer::BACKGROUND)),
            ..default()
        },
        DespawnOnRestart,
        Starfield,
    ));
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

fn warp_animation(
    mut materials: ResMut<Assets<StarfieldMaterial>>,
    query: Query<(&Transform, &Velocity), With<Player>>,
    mut starfield_query: Query<&mut Transform, (With<Starfield>, Without<Player>)>,
    time: Res<Time>,
    animation: Res<WarpAnimation>,
) {
    let (player_transform, player_velocity) = query.single();

    for mat in materials.iter_mut() {
        mat.1.pos += player_velocity.0
            * time.delta_seconds()
            * Vec2::splat(1. + Ease::quadratic_in(animation.starfield_timer.percent()) * 100.);
    }

    let mut starfield = starfield_query.single_mut();
    starfield.translation.x = player_transform.translation.x;
    starfield.translation.y = player_transform.translation.y;
}

impl Material2d for StarfieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/starfield.wgsl".into()
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Default, Clone)]
#[uuid = "721097c0-7368-453f-a95f-0731d6724689"]
pub struct StarfieldMaterial {
    #[uniform(0)]
    pub pos: Vec2,
    #[uniform(0)]
    pub _wasm_padding: Vec2,
}

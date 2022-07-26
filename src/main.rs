use basic_laser::{BasicLaser, BasicLaserPlugin};
use bevy::prelude::*;
use enemy::EnemyPlugin;
use fuel::FuelPlugin;
use leafwing_input_manager::prelude::*;
use starfield::StarfieldPlugin;

mod basic_laser;
mod enemy;
pub mod fuel;
mod layer;
mod starfield;
mod util;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        // This plugin maps inputs to an input-type agnostic action-state
        // We need to provide it with an enum which stores the possible actions a player could take
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugin(StarfieldPlugin)
        .add_plugin(BasicLaserPlugin)
        .add_plugin(EnemyPlugin)
        .add_plugin(FuelPlugin)
        .add_startup_system(spawn_player)
        // Read the ActionState in your systems using queries!
        .add_system(player_input)
        .add_system(thruster.before(acceleration))
        .add_system(acceleration.before(apply_acceleration))
        .add_system(apply_acceleration.before(movement))
        .add_system(rotation.before(movement))
        .add_system(movement)
        .add_system(move_camera.after(movement))
        .run();
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    TurnLeft,
    TurnRight,
    Forward,
    Reverse,
}

#[derive(Component)]
struct Player;
#[derive(Component)]
struct PlayerThruster;
#[derive(Component, Default)]
struct Acceleration(Vec2);
#[derive(Component, Default)]
struct Velocity(Vec2);
#[derive(Component, Default)]
struct AngularVelocity(f32);
#[derive(Component, Default)]
struct Rotation(f32);
#[derive(Component)]
struct RotationSpeed(f32);
#[derive(Component)]
struct Thrust(f32);
#[derive(Component, PartialEq)]
enum ThrusterStatus {
    Forward,
    Reverse,
    None,
}
#[derive(Component)]
struct MaxVelocity(f32);
#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    let mut input_map = InputMap::new([
        (KeyCode::A, Action::TurnLeft),
        (KeyCode::Left, Action::TurnLeft),
        (KeyCode::D, Action::TurnRight),
        (KeyCode::Right, Action::TurnRight),
        (KeyCode::W, Action::Forward),
        (KeyCode::Up, Action::Forward),
        (KeyCode::S, Action::Reverse),
        (KeyCode::Down, Action::Reverse),
    ]);

    input_map.insert_multiple([
        (GamepadButtonType::DPadLeft, Action::TurnLeft),
        (GamepadButtonType::DPadRight, Action::TurnRight),
        (GamepadButtonType::DPadUp, Action::Forward),
        (GamepadButtonType::DPadDown, Action::Reverse),
    ]);

    commands
        .spawn_bundle(SpatialBundle::default())
        .insert(Player)
        //.insert_bundle(SpatialBundle::default())
        .insert(Acceleration::default())
        .insert(Velocity::default())
        .insert(AngularVelocity::default())
        .insert(Rotation(std::f32::consts::FRAC_PI_2))
        .insert(RotationSpeed(1.))
        .insert(Thrust(50.))
        .insert(ThrusterStatus::None)
        .insert(MaxVelocity(50.))
        .insert(BasicLaser {
            timer: Timer::from_seconds(1., true),
            damage: 1.,
        })
        .insert_bundle(InputManagerBundle::<Action> {
            // Stores "which actions are currently pressed"
            action_state: ActionState::default(),
            // Describes how to convert from player inputs into those actions
            input_map,
        })
        .with_children(|parent| {
            // ship body
            parent.spawn_bundle(ColorMesh2dBundle {
                mesh: meshes.add(shape::RegularPolygon::new(20., 3).into()).into(),
                material: materials.add(Color::RED.into()),
                transform: Transform::from_rotation(Quat::from_rotation_z(
                    -std::f32::consts::FRAC_PI_2,
                ))
                .with_translation(Vec3::new(0., 0., layer::SHIP)),
                ..default()
            });
            // thruster
            parent
                .spawn_bundle(ColorMesh2dBundle {
                    mesh: meshes.add(shape::RegularPolygon::new(10., 3).into()).into(),
                    material: materials.add(Color::ORANGE.into()),
                    transform: Transform::from_rotation(Quat::from_rotation_z(
                        std::f32::consts::FRAC_PI_2,
                    ))
                    .with_translation(Vec3::new(-10., 0., layer::THRUSTER)),
                    ..default()
                })
                .insert(PlayerThruster);
        });

    commands.spawn_bundle(ColorMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(60.).into()).into(),
        material: materials.add(Color::AQUAMARINE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(0., 0., layer::PLANET)),
        ..default()
    });
}

fn player_input(
    query: Query<&ActionState<Action>, With<Player>>,
    mut player_query: Query<(&mut AngularVelocity, &mut ThrusterStatus), With<Player>>,
) {
    let action_state = query.single();
    let (mut angular, mut thruster_status) = player_query.single_mut();

    let mut new_angular = 0.;

    if action_state.pressed(Action::TurnLeft) {
        new_angular += 1.0;
    }

    if action_state.pressed(Action::TurnRight) {
        new_angular -= 1.0;
    }

    let new_thruster =
        if action_state.pressed(Action::Forward) && !action_state.pressed(Action::Reverse) {
            ThrusterStatus::Forward
        } else if action_state.pressed(Action::Reverse) && !action_state.pressed(Action::Forward) {
            ThrusterStatus::Reverse
        } else {
            ThrusterStatus::None
        };

    if angular.0 != new_angular {
        angular.0 = new_angular;
    }

    if *thruster_status != new_thruster {
        *thruster_status = new_thruster;
    }
}

fn acceleration(mut query: Query<(&mut Acceleration, &Thrust, &ThrusterStatus, &Rotation)>) {
    for (mut acceleration, thrust, thruster_status, rotation) in query.iter_mut() {
        match thruster_status {
            ThrusterStatus::Forward => {
                let sin_cos = rotation.0.sin_cos();
                acceleration.0.x = sin_cos.1;
                acceleration.0.y = sin_cos.0;
            }
            ThrusterStatus::Reverse => {
                let sin_cos = (rotation.0 + std::f32::consts::PI).sin_cos();
                acceleration.0.x = sin_cos.1;
                acceleration.0.y = sin_cos.0;
            }
            ThrusterStatus::None => {
                acceleration.0 = Vec2::ZERO;
            }
        }

        acceleration.0 *= thrust.0;
    }
}

fn apply_acceleration(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Acceleration, &MaxVelocity)>,
) {
    for (mut velocity, acceleration, max_velocity) in query.iter_mut() {
        velocity.0 += acceleration.0 * time.delta_seconds();
        velocity.0 = velocity.0.clamp_length_max(max_velocity.0);
    }
}

fn rotation(
    time: Res<Time>,
    mut query: Query<(
        &mut Transform,
        &AngularVelocity,
        &mut Rotation,
        &RotationSpeed,
    )>,
) {
    for (mut transform, angular, mut rotation, rotation_speed) in query.iter_mut() {
        rotation.0 += angular.0 * time.delta_seconds() * rotation_speed.0;

        transform.rotation = Quat::from_rotation_z(rotation.0);
    }
}

fn movement(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += (velocity.0 * time.delta_seconds()).extend(0.);
    }
}

fn thruster(
    mut thruster_query: Query<&mut Visibility, With<PlayerThruster>>,
    status_query: Query<&ThrusterStatus, (Changed<ThrusterStatus>, With<Player>)>,
) {
    for status in status_query.iter() {
        let mut thruster = thruster_query.single_mut();

        thruster.is_visible = matches!(status, ThrusterStatus::Forward);
    }
}

fn move_camera(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player = player_query.single();
    let mut camera = camera_query.single_mut();
    camera.translation.x = player.translation.x;
    camera.translation.y = player.translation.y;
}

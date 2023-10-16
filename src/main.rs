#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use basic_laser::{BasicLaser, BasicLaserPlugin};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_spatial::{AutomaticUpdate, SpatialStructure};
use commodity::{CommodityInventory, CommodityPlugin};
use direction_indicator::{
    DirectionIndicator, DirectionIndicatorPlugin, DirectionIndicatorSettings,
};
use enemy::EnemyPlugin;
use fuel::FuelPlugin;
use leafwing_input_manager::prelude::*;
use scanner::ScannerPlugin;
use starfield::StarfieldPlugin;
use ui::UiPlugin;
use warp_node::{WarpNodePlugin, WarpedTo};

mod basic_laser;
mod commodity;
mod direction_indicator;
mod enemy;
pub mod fuel;
mod layer;
mod scanner;
mod starfield;
mod ui;
mod util;
mod warp_node;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..default()
        }))
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Playing),
        )
        .add_collection_to_loading_state::<_, Fonts>(GameState::Loading)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugin(
            AutomaticUpdate::<SpatialIndex>::new().with_spatial_ds(SpatialStructure::KDTree2),
        )
        .add_plugin(StarfieldPlugin)
        .add_plugin(BasicLaserPlugin)
        .add_plugin(EnemyPlugin)
        .add_plugin(FuelPlugin)
        .add_plugin(DirectionIndicatorPlugin)
        .add_plugin(CommodityPlugin)
        .add_plugin(ScannerPlugin)
        .add_plugin(WarpNodePlugin)
        .add_plugin(UiPlugin)
        .add_system(spawn_player.in_schedule(OnExit(GameState::Loading)))
        .add_system(spawn_level.in_schedule(OnEnter(GameState::Playing)))
        .add_systems(
            (
                player_input,
                thruster.before(acceleration),
                acceleration.before(apply_acceleration),
                apply_acceleration.before(movement),
                rotation.before(movement),
                movement,
                move_camera.after(movement),
            )
                .in_set(OnUpdate(GameState::Playing))
                .in_set(MovementSet),
        )
        .add_systems(
            (warp_movement, move_camera.after(warp_movement))
                .in_set(OnUpdate(GameState::Warping))
                .in_set(MovementSet),
        )
        .add_systems((cleanup, sell, reset_player).in_schedule(OnExit(GameState::Playing)))
        .run();
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct MovementSet;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
    Warping,
}

#[derive(Resource, AssetCollection)]
struct Fonts {
    #[asset(path = "fonts/Orbitron-Medium.ttf")]
    main: Handle<Font>,
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
pub struct Player;
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
#[derive(Component)]
struct FuelTank {
    current: u32,
    max: u32,
}
#[derive(Component)]
struct Credits(u32);

#[derive(Component)]
struct DespawnOnRestart;

#[derive(Component)]
struct SpatialIndex;

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

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
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0., 0., layer::SHIP),
                ..default()
            },
            Player,
            Acceleration::default(),
            Velocity::default(),
            AngularVelocity::default(),
            Rotation(std::f32::consts::FRAC_PI_2),
            RotationSpeed(2.),
            Thrust(100.),
            ThrusterStatus::None,
            MaxVelocity(100.),
            FuelTank {
                current: 0,
                max: 40,
            },
            Credits(0),
            BasicLaser {
                timer: Timer::from_seconds(1., TimerMode::Repeating),
                damage: 1.,
            },
            CommodityInventory::default(),
            InputManagerBundle::<Action> {
                // Stores "which actions are currently pressed"
                action_state: ActionState::default(),
                // Describes how to convert from player inputs into those actions
                input_map,
            },
        ))
        .with_children(|parent| {
            // ship body
            parent.spawn(ColorMesh2dBundle {
                mesh: meshes.add(shape::RegularPolygon::new(20., 3).into()).into(),
                material: materials.add(Color::RED.into()),
                transform: Transform::from_rotation(Quat::from_rotation_z(
                    -std::f32::consts::FRAC_PI_2,
                )),
                ..default()
            });
            // thruster
            parent
                .spawn(ColorMesh2dBundle {
                    mesh: meshes.add(shape::RegularPolygon::new(10., 3).into()).into(),
                    material: materials.add(Color::ORANGE.into()),
                    transform: Transform::from_rotation(Quat::from_rotation_z(
                        std::f32::consts::FRAC_PI_2,
                    ))
                    .with_translation(Vec3::new(-10., 0., -0.1)),
                    ..default()
                })
                .insert(PlayerThruster);
        });
}

fn spawn_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("spawn_level");
    let planet = commands
        .spawn((
            ColorMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(60.).into()).into(),
                material: materials.add(Color::AQUAMARINE.into()),
                transform: Transform::from_xyz(0., 0., layer::PLANET),
                ..default()
            },
            DespawnOnRestart,
        ))
        .id();

    commands.spawn((
        DirectionIndicator {
            target: planet,
            settings: DirectionIndicatorSettings {
                color: Color::AQUAMARINE,
                label: None,
            },
        },
        DespawnOnRestart,
    ));
}

fn sell(
    mut query: Query<(&mut Credits, &mut CommodityInventory), With<Player>>,
    warped_to: Option<Res<WarpedTo>>,
) {
    if warped_to.is_none() {
        return;
    }
    let warped_to = warped_to.unwrap();

    let (mut credits, mut inventory) = query.single_mut();

    for (commodity, quantity) in inventory.0.drain() {
        let multiplier = warped_to.0 .0.get(&commodity).unwrap_or(&1.);
        let price = (quantity as f32 * multiplier).round() as u32;
        credits.0 += price;
    }
}

fn reset_player(
    mut query: Query<
        (
            &mut Transform,
            &mut FuelTank,
            &mut Velocity,
            &mut AngularVelocity,
        ),
        With<Player>,
    >,
) {
    for (mut transform, mut fuel, mut velocity, mut angular) in query.iter_mut() {
        transform.translation = Vec2::ZERO.extend(layer::SHIP);
        fuel.current = 0;
        velocity.0 = Vec2::ZERO;
        angular.0 = 0.;
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<DespawnOnRestart>>) {
    println!("cleanup");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
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

fn warp_movement(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += (velocity.0 * time.delta_seconds() / 7.).extend(0.);
    }
}

fn thruster(
    mut thruster_query: Query<&mut Visibility, With<PlayerThruster>>,
    status_query: Query<&ThrusterStatus, (Changed<ThrusterStatus>, With<Player>)>,
) {
    for status in status_query.iter() {
        let mut thruster = thruster_query.single_mut();

        *thruster = if matches!(status, ThrusterStatus::Forward) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        }
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

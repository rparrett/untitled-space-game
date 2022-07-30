use std::time::Duration;

use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    fuel::SpawnFuelPelletEvent, util, DespawnOnRestart, GameState, Health, MaxVelocity, Player,
    Velocity,
};

struct RampUpTimer(Timer);
struct SpawnTimer(Timer);
#[derive(Component)]
pub struct Enemy;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer(Timer::from_seconds(4., true)))
            .insert_resource(RampUpTimer(Timer::from_seconds(30., true)))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(spawn_enemy)
                    .with_system(move_enemy)
                    .with_system(ramp_up)
                    .with_system(despawn),
            );
    }
}

fn spawn_enemy(
    mut commands: Commands,
    mut timer: ResMut<SpawnTimer>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let player = player_query.single();

    // TODO hardcoded based on screen size
    let spawn_bounds = Vec2::new(700., 410.);

    let theta = thread_rng().gen_range(0.0..std::f32::consts::TAU);

    let pos =
        util::project_onto_bounding_rectangle(Vec2::from_angle(theta), -spawn_bounds, spawn_bounds)
            .unwrap()
            .0
            + player.translation.truncate();

    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(pos.extend(crate::layer::SHIP)),
            sprite: Sprite {
                color: Color::PURPLE,
                custom_size: Some(Vec2::new(20., 20.)),
                ..default()
            },
            ..default()
        })
        .insert(Enemy)
        .insert(Health {
            current: 1.,
            max: 1.,
        })
        .insert(MaxVelocity(30.))
        .insert(Velocity::default())
        .insert(DespawnOnRestart);
}

fn move_enemy(
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&mut Velocity, &MaxVelocity, &Transform), With<Enemy>>,
) {
    let player = player_query.single();

    for (mut velocity, max_velocity, transform) in enemy_query.iter_mut() {
        let diff = player.translation.truncate() - transform.translation.truncate();

        velocity.0 = diff.normalize() * max_velocity.0;
    }
}

fn ramp_up(time: Res<Time>, mut spawn: ResMut<SpawnTimer>, mut ramp: ResMut<RampUpTimer>) {
    ramp.0.tick(time.delta());
    if !ramp.0.just_finished() {
        return;
    }

    let new = (spawn.0.duration().as_secs_f32() / 1.5).max(0.5);
    spawn.0.set_duration(Duration::from_secs_f32(new));
}

fn despawn(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Transform), With<Enemy>>,
    mut events: EventWriter<SpawnFuelPelletEvent>,
) {
    for (entity, health, transform) in query.iter() {
        if health.current < health.max {
            commands.entity(entity).despawn();
            events.send(SpawnFuelPelletEvent {
                location: transform.translation.truncate(),
            });
            continue;
        }
    }
}

use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{Player, Velocity};

struct SpawnTimer(Timer);
#[derive(Component)]
struct Enemy;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpawnTimer(Timer::from_seconds(5., true)))
            .add_system(spawn_enemy)
            .add_system(move_enemy);
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

    let pos = (Vec2::from_angle(theta) * spawn_bounds.max_element())
        .clamp(-spawn_bounds, spawn_bounds)
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
        .insert(Velocity::default());
}

fn move_enemy(
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&mut Velocity, &Transform), With<Enemy>>,
) {
    let player = player_query.single();

    for (mut velocity, transform) in enemy_query.iter_mut() {
        let diff = player.translation.truncate() - transform.translation.truncate();
        velocity.0 = diff.normalize() * 30.;
    }
}

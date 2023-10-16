use bevy::{prelude::*, utils::HashSet};

use crate::{enemy::Enemy, layer, DespawnOnRestart, GameState, Health, Velocity};

pub struct BasicLaserPlugin;

impl Plugin for BasicLaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((fire, collide, despawn).in_set(OnUpdate(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct BasicLaser {
    pub timer: Timer,
    pub damage: f32,
}
#[derive(Component)]
struct Bullet {
    damage: f32,
    piercing: bool,
}
#[derive(Component)]
struct Range(f32);
#[derive(Component)]
struct Origin(Vec2);

fn fire(mut commands: Commands, mut query: Query<(&mut BasicLaser, &Transform)>, time: Res<Time>) {
    for (mut gun, transform) in query.iter_mut() {
        gun.timer.tick(time.delta());
        if !gun.timer.just_finished() {
            continue;
        }

        let rot = transform.rotation;
        let trans = rot.mul_vec3(Vec3::new(25., 0., 0.)) + transform.translation;

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::YELLOW,
                    custom_size: Some(Vec2::new(10., 2.)),
                    ..default()
                },
                transform: Transform::from_translation(trans.truncate().extend(layer::BULLET))
                    .with_rotation(rot),
                ..default()
            },
            Origin(trans.truncate()),
            Range(200.),
            Bullet {
                damage: gun.damage,
                piercing: false,
            },
            Velocity(rot.mul_vec3(Vec3::new(1., 0., 0.)).truncate() * 100.),
            DespawnOnRestart,
        ));
    }
}

fn collide(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform, &Bullet)>,
    mut enemy_query: Query<(&Transform, &mut Health), With<Enemy>>,
) {
    let mut used_bullets = HashSet::new();

    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        if used_bullets.contains(&bullet_entity) {
            continue;
        }
        for (enemy_transform, mut health) in enemy_query.iter_mut() {
            if enemy_transform
                .translation
                .truncate()
                .distance(bullet_transform.translation.truncate())
                < 10.
            {
                health.current -= bullet.damage;

                if !bullet.piercing {
                    used_bullets.insert(bullet_entity);
                    commands.entity(bullet_entity).despawn();
                    break;
                }
            }
        }
    }
}

fn despawn(mut commands: Commands, query: Query<(Entity, &Range, &Transform, &Origin)>) {
    for (entity, range, transform, origin) in query.iter() {
        let dist = origin.0.distance(transform.translation.truncate());
        if dist > range.0 {
            commands.entity(entity).despawn();
        }
    }
}

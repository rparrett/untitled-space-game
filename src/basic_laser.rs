use bevy::prelude::*;

use crate::{enemy::Enemy, layer, Velocity};

pub struct BasicLaserPlugin;

impl Plugin for BasicLaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(fire).add_system(collide).add_system(despawn);
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

        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::YELLOW,
                    custom_size: Some(Vec2::new(10., 2.)),
                    ..default()
                },
                transform: Transform::from_translation(trans.truncate().extend(layer::BULLET))
                    .with_rotation(rot),
                ..default()
            })
            .insert(Origin(trans.truncate()))
            .insert(Range(200.))
            .insert(Bullet { damage: gun.damage })
            .insert(Velocity(
                rot.mul_vec3(Vec3::new(1., 0., 0.)).truncate() * 100.,
            ));
    }
}

fn collide(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform, &Bullet)>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            if enemy_transform
                .translation
                .truncate()
                .distance(bullet_transform.translation.truncate())
                < 10.
            {
                commands.entity(bullet_entity).despawn();
                commands.entity(enemy_entity).despawn();
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

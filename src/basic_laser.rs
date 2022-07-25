use bevy::prelude::*;

use crate::{layer, Velocity};

pub struct BasicLaserPlugin;

impl Plugin for BasicLaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(fire);
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
            .insert(Bullet { damage: gun.damage })
            .insert(Velocity(
                rot.mul_vec3(Vec3::new(1., 0., 0.)).truncate() * 100.,
            ));
    }
}

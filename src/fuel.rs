use bevy::prelude::*;

use crate::{layer, Acceleration, MaxVelocity, Player, Velocity};

pub struct FuelPlugin;
impl Plugin for FuelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement)
            .add_system(spawn)
            .add_event::<SpawnFuelPelletEvent>();
    }
}

#[derive(Component)]
struct FuelPellet;
pub struct SpawnFuelPelletEvent {
    pub location: Vec2,
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut events: EventReader<SpawnFuelPelletEvent>,
) {
    for event in events.iter() {
        commands
            .spawn_bundle(ColorMesh2dBundle {
                mesh: meshes
                    .add(
                        shape::RegularPolygon {
                            sides: 6,
                            radius: 5.,
                        }
                        .into(),
                    )
                    .into(),
                material: materials.add(Color::GREEN.into()),
                transform: Transform::from_translation(event.location.extend(layer::OBJECT)),
                ..default()
            })
            .insert(Velocity::default())
            .insert(Acceleration::default())
            .insert(MaxVelocity(300.))
            .insert(FuelPellet);
    }
}

fn movement(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut query: Query<(Entity, &mut Velocity, &Transform), With<FuelPellet>>,
) {
    let player = player_query.single();

    for (entity, mut acceleration, transform) in query.iter_mut() {
        let diff = player.translation.truncate() - transform.translation.truncate();
        let dist = diff.length();

        // TODO Player should have a PickupRange
        // TODO Maybe try applying velocity directly proportional to square distance
        if dist <= 60. {
            acceleration.0 = diff.normalize() * 100.;
        } else {
            acceleration.0 = Vec2::ZERO;
        }

        // TODO Player should have a collider for this
        if dist <= 10. {
            commands.entity(entity).despawn();
        }
    }
}

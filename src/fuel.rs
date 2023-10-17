use bevy::prelude::*;

use crate::{
    layer, Acceleration, DespawnOnRestart, FuelTank, GameState, MaxVelocity, Player, Velocity,
};

pub struct FuelPlugin;
impl Plugin for FuelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnFuelPelletEvent>();
        app.add_systems(
            Update,
            (movement, spawn).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
struct FuelPellet;
#[derive(Event)]
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
        commands.spawn((
            ColorMesh2dBundle {
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
            },
            Velocity::default(),
            Acceleration::default(),
            MaxVelocity(300.),
            FuelPellet,
            DespawnOnRestart,
        ));
    }
}

fn movement(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut FuelTank), With<Player>>,
    mut query: Query<(Entity, &mut Velocity, &Transform), With<FuelPellet>>,
) {
    let (player, mut fuel_tank) = player_query.single_mut();

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
            fuel_tank.current = (fuel_tank.current + 1).min(fuel_tank.max);
            commands.entity(entity).despawn();
        }
    }
}

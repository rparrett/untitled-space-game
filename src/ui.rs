use bevy::prelude::*;

use crate::FuelTank;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(update_fuel);
    }
}

#[derive(Component)]
pub struct FuelLabel;

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands
        .spawn_bundle(TextBundle {
            text: Text::from_section(
                "FUEL 0 / 50",
                TextStyle {
                    font: assets.load("fonts/Orbitron-Medium.ttf"),
                    font_size: 20.,
                    color: Color::GREEN,
                },
            ),
            ..default()
        })
        .insert(FuelLabel);
}

fn update_fuel(
    query: Query<&FuelTank, Changed<FuelTank>>,
    mut label_query: Query<&mut Text, With<FuelLabel>>,
) {
    for fuel in query.iter() {
        for mut label in label_query.iter_mut() {
            label.sections[0].value = format!("FUEL {} / {}", fuel.current, fuel.max);
        }
    }
}

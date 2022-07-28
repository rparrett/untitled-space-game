use bevy::prelude::*;

use crate::{commodity::CommodityInventory, scanner::Scanner, FuelTank, Player};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(update_fuel)
            .add_system(update_commodity_inventory)
            .add_system(update_scanner);
    }
}

#[derive(Component)]
struct FuelLabel;
#[derive(Component)]
struct CommodityInventoryLabel;
#[derive(Component)]
struct ScannerLabel;

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let container = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .id();

    let fuel = commands
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
        .insert(FuelLabel)
        .id();

    let comm = commands
        .spawn_bundle(TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font: assets.load("fonts/Orbitron-Medium.ttf"),
                    font_size: 20.,
                    color: Color::BEIGE,
                },
            )
            .with_alignment(TextAlignment::CENTER_RIGHT),
            ..default()
        })
        .insert(CommodityInventoryLabel)
        .id();

    let scanner = commands
        .spawn_bundle(TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font: assets.load("fonts/Orbitron-Medium.ttf"),
                    font_size: 20.,
                    color: Color::BEIGE,
                },
            )
            .with_alignment(TextAlignment::CENTER_RIGHT),
            ..default()
        })
        .insert(ScannerLabel)
        .id();

    commands
        .entity(container)
        .push_children(&[fuel, comm, scanner]);
}

fn update_fuel(
    query: Query<&FuelTank, (Changed<FuelTank>, With<Player>)>,
    mut label_query: Query<&mut Text, With<FuelLabel>>,
) {
    for fuel in query.iter() {
        for mut label in label_query.iter_mut() {
            label.sections[0].value = format!("FUEL {} / {}", fuel.current, fuel.max);
        }
    }
}

fn update_commodity_inventory(
    query: Query<&CommodityInventory, (Changed<CommodityInventory>, With<Player>)>,
    mut label_query: Query<&mut Text, With<CommodityInventoryLabel>>,
) {
    for inventory in query.iter() {
        for mut label in label_query.iter_mut() {
            label.sections[0].value = inventory
                .0
                .iter()
                .map(|(k, v)| format!("{:?}: {}\n", k, v))
                .collect::<String>()
        }
    }
}

fn update_scanner(scanner: Res<Scanner>, mut query: Query<&mut Text, With<ScannerLabel>>) {
    if !scanner.is_changed() {
        return;
    }

    for mut text in query.iter_mut() {
        if !scanner.timer.paused() {
            text.sections[0].value = "Scanning...".to_string();
        } else {
            text.sections[0].value = "".to_string();
        }
    }
}

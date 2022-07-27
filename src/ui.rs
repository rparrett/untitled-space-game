use bevy::prelude::*;

use crate::{commodity::CommodityInventory, FuelTank, Player};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(update_fuel)
            .add_system(update_commodity_inventory);
    }
}

#[derive(Component)]
pub struct FuelLabel;
#[derive(Component)]
pub struct CommodityInventoryLabel;

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
            .with_alignment(TextAlignment::TOP_RIGHT),
            ..default()
        })
        .insert(CommodityInventoryLabel)
        .id();

    commands.entity(container).push_children(&[fuel, comm]);
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

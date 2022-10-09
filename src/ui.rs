use bevy::prelude::*;

use crate::{
    commodity::{CommodityInventory, CommodityPrices},
    direction_indicator::{DirectionIndicator, DirectionIndicatorSettings},
    scanner::{self, Scanner},
    warp_node::WarpNode,
    Credits, DespawnOnRestart, Fonts, FuelTank, GameState, Player,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WarpNodeDisplayOrder::default())
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(update_fuel)
                    .with_system(update_credits)
                    .with_system(update_commodity_inventory)
                    .with_system(update_scanner.after(scanner::update))
                    .with_system(track_warp_nodes)
                    .with_system(update_warp_nodes),
            );
    }
}

#[derive(Component)]
struct FuelLabel;
#[derive(Component)]
struct CreditsLabel;
#[derive(Component)]
struct CommodityInventoryLabel;
#[derive(Component)]
struct ScannerLabel;
#[derive(Component)]
struct WarpNodesLabel;

#[derive(Default)]
struct WarpNodeDisplayOrder(Vec<Entity>);

fn setup(mut commands: Commands, fonts: Res<Fonts>) {
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
        .insert(DespawnOnRestart)
        .id();

    let fuel = commands
        .spawn_bundle(TextBundle {
            text: Text::from_section(
                "FUEL 0 / 50",
                TextStyle {
                    font: fonts.main.clone(),
                    font_size: 20.,
                    color: Color::GREEN,
                },
            ),
            style: Style {
                margin: UiRect {
                    right: Val::Px(5.),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(FuelLabel)
        .id();

    let credits = commands
        .spawn_bundle(TextBundle {
            text: Text::from_section(
                "CREDS 0",
                TextStyle {
                    font: fonts.main.clone(),
                    font_size: 20.,
                    color: Color::WHITE,
                },
            ),
            style: Style {
                margin: UiRect {
                    right: Val::Px(5.),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(CreditsLabel)
        .id();

    let comm = commands
        .spawn_bundle(TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font: fonts.main.clone(),
                    font_size: 20.,
                    color: Color::BEIGE,
                },
            )
            .with_alignment(TextAlignment::CENTER_RIGHT),
            style: Style {
                margin: UiRect {
                    top: Val::Px(10.),
                    right: Val::Px(5.),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(CommodityInventoryLabel)
        .id();

    let warp_nodes = commands
        .spawn_bundle(TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font: fonts.main.clone(),
                    font_size: 20.,
                    color: Color::ORANGE,
                },
            )
            .with_alignment(TextAlignment::CENTER_RIGHT),
            style: Style {
                margin: UiRect {
                    top: Val::Px(10.),
                    right: Val::Px(5.),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(WarpNodesLabel)
        .id();

    let scanner = commands
        .spawn_bundle(TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font: fonts.main.clone(),
                    font_size: 20.,
                    color: Color::BEIGE,
                },
            )
            .with_alignment(TextAlignment::CENTER_RIGHT),
            style: Style {
                margin: UiRect {
                    top: Val::Px(10.),
                    right: Val::Px(5.),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(ScannerLabel)
        .id();

    commands
        .entity(container)
        .push_children(&[fuel, credits, comm, warp_nodes, scanner]);
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

fn update_credits(
    query: Query<&Credits, (Changed<Credits>, With<Player>)>,
    mut label_query: Query<&mut Text, With<CreditsLabel>>,
) {
    for credits in query.iter() {
        for mut label in label_query.iter_mut() {
            label.sections[0].value = format!("CREDS {}", credits.0);
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
                .map(|(k, v)| format!("{:?} {}\n", k, v))
                .collect::<String>()
        }
    }
}

fn track_warp_nodes(
    query: Query<&DirectionIndicator, Added<DirectionIndicator>>,
    node_query: Query<Entity, With<WarpNode>>,
    mut display_order: ResMut<WarpNodeDisplayOrder>,
) {
    for indicator in query.iter() {
        if let Ok(entity) = node_query.get(indicator.target) {
            display_order.0.push(entity);
        }
    }
}

fn update_warp_nodes(
    query: Query<(&CommodityPrices, &DirectionIndicatorSettings)>,
    mut text_query: Query<&mut Text, With<WarpNodesLabel>>,
    display_order: Res<WarpNodeDisplayOrder>,
    fonts: Res<Fonts>,
) {
    if !display_order.is_changed() {
        return;
    }

    let style_good = TextStyle {
        font: fonts.main.clone(),
        font_size: 20.,
        color: Color::GREEN,
    };
    let style_bad = TextStyle {
        font: fonts.main.clone(),
        font_size: 20.,
        color: Color::RED,
    };
    let style_label = TextStyle {
        font: fonts.main.clone(),
        font_size: 20.,
        color: Color::ORANGE,
    };
    let style_neutral = TextStyle {
        font: fonts.main.clone(),
        font_size: 20.,
        color: Color::BEIGE,
    };

    for mut text in text_query.iter_mut() {
        let mut sections = vec![];

        for entity in &display_order.0 {
            if let Ok((prices, settings)) = query.get(*entity) {
                sections.push(TextSection::new("Node ".to_string(), style_label.clone()));
                sections.push(TextSection::new(
                    settings
                        .label
                        .as_ref()
                        .map_or_else(|| "?".to_string(), |l| l.clone()),
                    style_label.clone(),
                ));
                sections.push(TextSection::new("\n".to_string(), style_label.clone()));

                for (kind, price) in prices.0.iter() {
                    let (price_style, sign) = if *price < 1.0 {
                        (style_bad.clone(), "-")
                    } else {
                        (style_good.clone(), "+")
                    };

                    sections.push(TextSection::new(
                        format!("{:?} ", kind),
                        style_neutral.clone(),
                    ));
                    sections.push(TextSection::new(
                        format!("{}{:.0}%\n", sign, (1. - price).abs() * 100.),
                        price_style,
                    ));
                }

                sections.push(TextSection::new("\n".to_string(), style_neutral.clone()));
            }
        }
        text.sections = sections;
    }
}

fn update_scanner(scanner: Res<Scanner>, mut query: Query<&mut Text, With<ScannerLabel>>) {
    for mut text in query.iter_mut() {
        if !scanner.timer.paused() {
            let pct = scanner.timer.percent() * 100.;

            text.sections[0].value = format!("Scanning {:.0}%", pct);
        } else {
            text.sections[0].value = "".to_string();
        }
    }
}

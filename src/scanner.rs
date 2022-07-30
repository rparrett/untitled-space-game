use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    commodity::Commodity,
    direction_indicator::{DirectionIndicator, DirectionIndicatorSettings},
    DespawnOnRestart, GameState,
};

pub struct ScannerPlugin;
impl Plugin for ScannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_exit(GameState::Warping).with_system(reset))
            .init_resource::<Scanner>()
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(update)
                    .with_system(unpause),
            );
    }
}

pub struct Scanner {
    pub timer: Timer,
    pub commodities: VecDeque<Entity>,
    pub warp_nodes: VecDeque<Entity>,
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(45., true),
            commodities: VecDeque::new(),
            warp_nodes: VecDeque::new(),
        }
    }
}

pub fn reset(mut commands: Commands) {
    commands.init_resource::<Scanner>();
}

pub fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut scanner: ResMut<Scanner>,
    target_query: Query<&DirectionIndicatorSettings>,
) {
    scanner.timer.tick(time.delta());

    if !scanner.timer.just_finished() {
        return;
    }

    let entities = if !scanner.commodities.is_empty() {
        &mut scanner.commodities
    } else {
        &mut scanner.warp_nodes
    };

    // Insert a DirectionIndicator for the first entity that still
    // exists. The player may have already collected the commodity
    // before it was revealed by the scanner.

    while let Some(entity) = entities.pop_front() {
        if let Ok(settings) = target_query.get(entity) {
            commands
                .spawn()
                .insert(DirectionIndicator {
                    target: entity,
                    settings: (*settings).clone(),
                })
                .insert(DespawnOnRestart);
            break;
        }
    }

    if entities.len() == 0 {
        scanner.timer.reset();
        scanner.timer.pause();
    }
}

/// Unpauses the scanner timer after all commodities are collected
///
/// TODO: this is pretty janky and does a lot of unnecessary unpausing.
pub fn unpause(commodity_query: Query<&Commodity>, mut scanner: ResMut<Scanner>) {
    if commodity_query.iter().len() != 0 {
        return;
    }

    if scanner.warp_nodes.is_empty() {
        return;
    }

    scanner.timer.unpause();
}

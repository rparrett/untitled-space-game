use std::collections::VecDeque;

use bevy::prelude::*;

use crate::direction_indicator::{DirectionIndicator, DirectionIndicatorColor};

pub struct ScannerPlugin;
impl Plugin for ScannerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Scanner {
            timer: Timer::from_seconds(60., true), // XXX
            commodities: VecDeque::new(),
            warp_nodes: VecDeque::new(),
        })
        .add_system(update);
    }
}

pub struct Scanner {
    pub timer: Timer,
    pub commodities: VecDeque<Entity>,
    pub warp_nodes: VecDeque<Entity>,
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut scanner: ResMut<Scanner>,
    target_query: Query<&DirectionIndicatorColor>,
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
        if let Ok(color) = target_query.get(entity) {
            commands.spawn().insert(DirectionIndicator {
                target: entity,
                color: color.0,
            });
            break;
        }
    }

    if entities.len() == 0 {
        scanner.timer.pause();
        scanner.timer.reset();
        return;
    }
}

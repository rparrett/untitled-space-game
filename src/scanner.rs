use std::collections::VecDeque;

use bevy::prelude::*;

use crate::direction_indicator::{DirectionIndicator, DirectionIndicatorColor};

pub struct ScannerPlugin;
impl Plugin for ScannerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Scanner {
            timer: Timer::from_seconds(60., true),
            entities: VecDeque::new(),
        })
        .add_system(update);
    }
}

pub struct Scanner {
    pub timer: Timer,
    pub entities: VecDeque<Entity>,
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

    if let Some(entity) = scanner.entities.pop_front() {
        if let Ok(color) = target_query.get(entity) {
            commands.spawn().insert(DirectionIndicator {
                target: entity,
                color: color.0,
            });
        }
    } else {
        scanner.timer.pause();
        scanner.timer.reset();
    }
}

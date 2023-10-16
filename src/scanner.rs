use bevy::prelude::*;

use crate::{
    commodity::Commodity,
    direction_indicator::{DirectionIndicator, DirectionIndicatorSettings},
    DespawnOnRestart, GameState, Player,
};

pub struct ScannerPlugin;
impl Plugin for ScannerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Scanner>();
        app.add_system(reset.in_schedule(OnExit(GameState::Warping)));
        app.add_systems((proximity, update, unpause).in_set(OnUpdate(GameState::Playing)));
    }
}

#[derive(Resource)]
pub struct Scanner {
    pub timer: Timer,
    pub commodities: Vec<Entity>,
    pub warp_nodes: Vec<Entity>,
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(45., TimerMode::Repeating),
            commodities: Vec::new(),
            warp_nodes: Vec::new(),
        }
    }
}

pub fn reset(mut scanner: ResMut<Scanner>) {
    *scanner = Scanner::default();
}

pub fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut scanner: ResMut<Scanner>,
    player_query: Query<&Transform, With<Player>>,
    target_query: Query<(Entity, &Transform, &DirectionIndicatorSettings)>,
) {
    scanner.timer.tick(time.delta());
    if !scanner.timer.just_finished() {
        return;
    }

    let player_transform = player_query.single();

    let entities = if !scanner.commodities.is_empty() {
        &mut scanner.commodities
    } else {
        &mut scanner.warp_nodes
    };

    // Insert a DirectionIndicator for the first entity that still
    // exists. The player may have already collected the commodity
    // before it was revealed by the scanner.

    let closest_target = target_query.iter_many(entities.iter()).min_by(|a, b| {
        let (_, a_transform, _) = a;
        let (_, b_transform, _) = b;
        let a_dist = a_transform
            .translation
            .distance_squared(player_transform.translation);
        let b_dist = b_transform
            .translation
            .distance_squared(player_transform.translation);
        a_dist.partial_cmp(&b_dist).unwrap()
    });

    if let Some((entity, _, settings)) = closest_target {
        commands.spawn((
            DirectionIndicator {
                target: entity,
                settings: (*settings).clone(),
            },
            DespawnOnRestart,
        ));

        entities
            .iter()
            .position(|e| *e == entity)
            .map(|e| entities.swap_remove(e));
    }

    if entities.is_empty() {
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

pub fn proximity(
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    query: Query<(Entity, &DirectionIndicatorSettings, &Transform)>,
    mut scanner: ResMut<Scanner>,
) {
    let (camera, gt) = camera_query.single();

    let mut remove: Option<Entity> = None;

    for (entity, settings, transform) in query
        .iter_many(&scanner.commodities)
        .chain(query.iter_many(&scanner.warp_nodes))
    {
        let ndc = camera.world_to_ndc(gt, transform.translation);
        if ndc.is_none() {
            continue;
        }
        let ndc = ndc.unwrap();

        let visible = ndc.x < 1. && ndc.y < 1. && ndc.x > -1. && ndc.y > -1.;

        if visible {
            commands.spawn((
                DirectionIndicator {
                    target: entity,
                    settings: (*settings).clone(),
                },
                DespawnOnRestart,
            ));

            remove = Some(entity);
        }
    }

    if let Some(entity) = remove {
        scanner
            .commodities
            .iter()
            .position(|e| *e == entity)
            .map(|e| scanner.commodities.swap_remove(e));
        scanner
            .warp_nodes
            .iter()
            .position(|e| *e == entity)
            .map(|e| scanner.warp_nodes.swap_remove(e));
    }
}

use bevy::{prelude::*, utils::HashMap};
use indexmap::IndexMap;
use itertools::izip;
use rand::{distributions::Uniform, seq::IteratorRandom, thread_rng, Rng};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    direction_indicator::DirectionIndicatorSettings, layer, scanner::Scanner, util,
    DespawnOnRestart, GameState, Player,
};

#[derive(EnumIter, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CommodityKind {
    Tungsten,
    Gallium,
    Thorium,
    Water,
    Organic,
    Crystal,
    Nitrate,
    Goods,
    Food,
}

#[derive(Component)]
pub struct Commodity {
    kind: CommodityKind,
    amount: u32,
}

#[derive(Component, Default)]
pub struct CommodityInventory(pub HashMap<CommodityKind, u32>);

#[derive(Component, Default, Clone)]
pub struct CommodityPrices(pub IndexMap<CommodityKind, f32>);

impl CommodityPrices {
    pub fn new_random() -> Self {
        let mut rng = thread_rng();

        let num = rng.gen_range(2..=3);

        let mut prices = IndexMap::new();

        let mut commodities = CommodityKind::iter().choose_multiple(&mut rng, num);

        for commodity in commodities.drain(0..) {
            let sign = if rng.gen() { 1. } else { -1. };

            let pct = rng.gen_range(1..=5) as f32 / 10.;

            let multiplier = 1. + pct * sign;

            prices.insert(commodity, multiplier);
        }

        Self(prices)
    }
}

pub struct CommodityPlugin;
impl Plugin for CommodityPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(GameState::Playing)));
        app.add_system(pickup.in_set(OnUpdate(GameState::Playing)));
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut scanner: ResMut<Scanner>,
) {
    let mut rng = thread_rng();

    let num = 3;

    let dist_range = Uniform::from(1500.0..2500.0);
    //let dist_range = Uniform::from(500.0..501.0);

    let amounts = util::random_u32_subdivisions(num, 100, 20);
    let kinds = CommodityKind::iter().choose_multiple(&mut rng, num as usize);
    let distances = rng.sample_iter(&dist_range).take(num as usize);
    let angles = util::random_circular_f32_distribution(num, 80., 360.);

    for (angle, distance, amount, kind) in izip!(angles, distances, amounts, kinds) {
        let angle = angle.to_radians();
        let (y, x) = angle.sin_cos();
        let pos = Vec3::new(x * distance, y * distance, layer::OBJECT);

        let entity = commands
            .spawn((
                ColorMesh2dBundle {
                    mesh: meshes.add(shape::RegularPolygon::new(10., 5).into()).into(),
                    material: materials.add(Color::BEIGE.into()),
                    transform: Transform::from_translation(pos),
                    ..default()
                },
                Commodity { kind, amount },
                DirectionIndicatorSettings {
                    color: Color::BEIGE,
                    label: None,
                },
                DespawnOnRestart,
            ))
            .id();

        scanner.commodities.push(entity);
    }
}

fn pickup(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Commodity)>,
    mut player_query: Query<(&Transform, &mut CommodityInventory), With<Player>>,
) {
    let (player_transform, mut inventory) = player_query.single_mut();
    for (entity, transform, commodity) in query.iter() {
        if transform
            .translation
            .truncate()
            .distance(player_transform.translation.truncate())
            < 20.
        {
            inventory
                .0
                .entry(commodity.kind.clone())
                .and_modify(|e| *e += commodity.amount)
                .or_insert(commodity.amount);

            commands.entity(entity).despawn();
        }
    }
}

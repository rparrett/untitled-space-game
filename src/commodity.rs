use bevy::{prelude::*, utils::HashMap};
use indexmap::IndexMap;
use itertools::izip;
use rand::{seq::IteratorRandom, thread_rng, Rng};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    direction_indicator::DirectionIndicatorSettings, layer, scanner::Scanner, util, Player,
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

#[derive(Component, Default)]
pub struct CommodityPrices(pub IndexMap<CommodityKind, f32>);

impl CommodityPrices {
    pub fn new_random() -> Self {
        let mut rng = thread_rng();
        let num = rng.gen_range(2..=3);

        let mut prices = IndexMap::new();

        let mut commodities = CommodityKind::iter().choose_multiple(&mut rng, num);

        for commodity in commodities.drain(0..) {
            let sign = if rng.gen::<bool>() { 1. } else { -1. };
            let multiplier = 1. + rng.gen_range(1..=5) as f32 / 10. * sign;

            prices.insert(commodity, multiplier);
        }

        Self(prices)
    }
}

pub struct CommodityPlugin;
impl Plugin for CommodityPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(pickup);
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut scanner: ResMut<Scanner>,
) {
    let mut rng = thread_rng();

    let angular_variance_range = -40.0..40.0;
    let angular_offset_range = -30.0..30.0;
    let dist_range = 1500.0..2500.0;

    let offset = rng.gen_range(angular_offset_range);

    let base_angles: [f32; 3] = [0., 120., 240.];

    let distances = [
        rng.gen_range(dist_range.clone()),
        rng.gen_range(dist_range.clone()),
        rng.gen_range(dist_range.clone()),
    ];

    let amounts = util::random_u32_subdivisions(3, 100, 20);
    let kinds = CommodityKind::iter().choose_multiple(&mut rng, 3);

    for (base_angle, distance, amount, kind) in izip!(base_angles, distances, amounts, kinds) {
        let angle = base_angle + offset + rng.gen_range(angular_variance_range.clone());

        let pos = Vec3::new(
            distance * angle.to_radians().cos(),
            distance * angle.to_radians().sin(),
            layer::OBJECT,
        );

        let entity = commands
            .spawn_bundle(ColorMesh2dBundle {
                mesh: meshes.add(shape::RegularPolygon::new(10., 5).into()).into(),
                material: materials.add(Color::BEIGE.into()),
                transform: Transform::from_translation(pos),
                ..default()
            })
            .insert(Commodity { kind, amount })
            .insert(DirectionIndicatorSettings {
                color: Color::BEIGE,
                label: None,
            })
            .id();

        scanner.commodities.push_back(entity);
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

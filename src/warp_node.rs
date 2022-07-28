use bevy::prelude::*;
use itertools::izip;
use rand::{thread_rng, Rng};

use crate::{
    commodity::Commodity, direction_indicator::DirectionIndicatorColor, layer, scanner::Scanner,
};

pub struct WarpNodePlugin;
impl Plugin for WarpNodePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system(restart_scanner);
    }
}

#[derive(Component)]
struct WarpNode;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut scanner: ResMut<Scanner>,
) {
    let mut rng = thread_rng();

    let angular_variance_range = -40.0..40.0;
    let angular_offset_range = -30.0..30.0;
    let dist_range = 2600.0..3000.0;

    let offset = rng.gen_range(angular_offset_range);

    let base_angles: [f32; 3] = [0., 120., 240.];

    let distances = [
        rng.gen_range(dist_range.clone()),
        rng.gen_range(dist_range.clone()),
        rng.gen_range(dist_range.clone()),
    ];

    for (base_angle, distance) in izip!(base_angles, distances) {
        let angle = base_angle + offset + rng.gen_range(angular_variance_range.clone());

        let pos = Vec3::new(
            distance * angle.to_radians().cos(),
            distance * angle.to_radians().sin(),
            layer::PLANET,
        );

        let entity = commands
            .spawn_bundle(ColorMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(80.).into()).into(),
                material: materials.add(
                    Color::Rgba {
                        red: 0.15,
                        blue: 0.15,
                        green: 0.15,
                        alpha: 0.2,
                    }
                    .into(),
                ),
                transform: Transform::from_translation(pos),
                ..default()
            })
            .insert(WarpNode)
            .insert(DirectionIndicatorColor(Color::ORANGE))
            .id();

        scanner.warp_nodes.push_back(entity);
    }
}

fn restart_scanner(mut scanner: ResMut<Scanner>, query: Query<&Commodity>) {
    if query.is_empty() && scanner.timer.paused() {
        scanner.timer.unpause();
    }
}

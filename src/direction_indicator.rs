use bevy::prelude::*;

use crate::{layer, util, Player};

pub struct DirectionIndicatorPlugin;
impl Plugin for DirectionIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update.after(crate::movement))
            .add_system(decorate);
    }
}

#[derive(Component)]
pub struct DirectionIndicator {
    pub target: Entity,
    pub color: Color,
}

#[derive(Component)]
pub struct DirectionIndicatorColor(pub Color);

fn update(
    mut query: Query<(&DirectionIndicator, &mut Transform, &mut Visibility)>,
    transform_query: Query<&Transform, Without<DirectionIndicator>>,
    player_query: Query<Entity, With<Player>>,
) {
    let player = player_query.single();
    let player_transform = transform_query.get(player).unwrap();

    let indicator_rect = Vec2::new(625., 345.);
    let on_screen_rect = Vec2::new(640., 360.);

    for (indicator, mut transform, mut visibility) in query.iter_mut() {
        if let Ok(target_transform) = transform_query.get(indicator.target) {
            let diff =
                target_transform.translation.truncate() - player_transform.translation.truncate();

            // TODO this should be a proper collision with the object geometry and the screen.
            if util::point_in_rect(diff, -on_screen_rect, on_screen_rect) {
                let diff = target_transform.translation.truncate()
                    - player_transform.translation.truncate();

                let pos =
                    util::project_onto_bounding_rectangle(diff, -indicator_rect, indicator_rect)
                        + player_transform.translation.truncate();

                transform.translation.x = pos.x;
                transform.translation.y = pos.y;

                transform.rotation =
                    Quat::from_rotation_z(diff.y.atan2(diff.x) + std::f32::consts::FRAC_PI_2);

                visibility.is_visible = true;
            } else {
                visibility.is_visible = false;
            }
        }
    }
}

fn decorate(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &DirectionIndicator), Added<DirectionIndicator>>,
) {
    for (entity, indicator) in query.iter() {
        commands.entity(entity).insert_bundle(ColorMesh2dBundle {
            mesh: meshes.add(util::chevron(20., 20., 8.).into()).into(),
            material: materials.add(indicator.color.into()),
            transform: Transform::from_xyz(0., 0., layer::UI),
            visibility: Visibility { is_visible: false },
            ..default()
        });
    }
}

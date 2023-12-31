use bevy::prelude::*;

use crate::{
    layer,
    util::{self, Edge},
    Fonts, GameState, Player,
};

pub struct DirectionIndicatorPlugin;
impl Plugin for DirectionIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update.after(crate::movement), decorate).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct DirectionIndicator {
    pub target: Entity,
    pub settings: DirectionIndicatorSettings,
}

#[derive(Component, Clone)]
pub struct DirectionIndicatorSettings {
    pub color: Color,
    pub label: Option<String>,
}

#[derive(Component)]
struct DirectionIndicatorLabel;

#[derive(Component)]
struct DirectionIndicatorDistanceLabel;

#[derive(Component)]
struct DirectionIndicatorArrow;

fn update(
    mut query: Query<(
        &DirectionIndicator,
        &mut Transform,
        &mut Visibility,
        &Children,
    )>,
    transform_query: Query<
        &Transform,
        (
            Without<DirectionIndicator>,
            Without<DirectionIndicatorArrow>,
            Without<DirectionIndicatorLabel>,
            Without<DirectionIndicatorDistanceLabel>,
        ),
    >,
    player_query: Query<
        &Transform,
        (
            With<Player>,
            Without<DirectionIndicator>,
            Without<DirectionIndicatorArrow>,
            Without<DirectionIndicatorLabel>,
            Without<DirectionIndicatorDistanceLabel>,
        ),
    >,
    mut label_query: Query<
        &mut Transform,
        (
            With<DirectionIndicatorLabel>,
            Without<DirectionIndicator>,
            Without<DirectionIndicatorArrow>,
            Without<DirectionIndicatorDistanceLabel>,
        ),
    >,
    mut distance_query: Query<
        (&mut Text, &mut Transform),
        (
            With<DirectionIndicatorDistanceLabel>,
            Without<DirectionIndicator>,
            Without<DirectionIndicatorLabel>,
            Without<DirectionIndicatorArrow>,
        ),
    >,
    mut arrow_query: Query<
        &mut Transform,
        (
            With<DirectionIndicatorArrow>,
            Without<DirectionIndicator>,
            Without<DirectionIndicatorLabel>,
            Without<DirectionIndicatorDistanceLabel>,
        ),
    >,
) {
    let player = player_query.single();

    let indicator_rect = Vec2::new(625., 345.);
    let on_screen_rect = Vec2::new(640., 360.);

    for (indicator, mut transform, mut visibility, children) in query.iter_mut() {
        let Ok(target_transform) = transform_query.get(indicator.target) else {
            continue;
        };

        let diff = target_transform.translation.truncate() - player.translation.truncate();

        // TODO this should be a proper collision with the object geometry and the screen.
        if util::point_in_rect(diff, -on_screen_rect, on_screen_rect) {
            *visibility = Visibility::Hidden;
            continue;
        }

        let projection =
            util::project_onto_bounding_rectangle(diff, -indicator_rect, indicator_rect).unwrap();

        let pos = projection.0 + player.translation.truncate();

        transform.translation.x = pos.x;
        transform.translation.y = pos.y;

        let theta = diff.y.atan2(diff.x);
        let theta_label = theta + std::f32::consts::PI;
        let sin_cos = theta_label.sin_cos();

        let offset = match (projection.1, indicator.settings.label.as_ref()) {
            (Edge::Top, Some(_)) => Vec2::new(0., -6.),
            (_, Some(_)) => Vec2::new(0., 6.),
            _ => Vec2::ZERO,
        };

        for child in children {
            if let Ok(mut label_transform) = label_query.get_mut(*child) {
                let pos = Vec2::new(sin_cos.1, sin_cos.0) * 30. + offset;

                label_transform.translation.x = pos.x;
                label_transform.translation.y = pos.y;

                continue;
            }

            if let Ok((mut label, mut label_transform)) = distance_query.get_mut(*child) {
                let pos = Vec2::new(sin_cos.1, sin_cos.0) * 30. + -offset;

                label_transform.translation.x = pos.x;
                label_transform.translation.y = pos.y;

                label.sections[0].value = format!("{:.1}Mm", diff.length() / 1000.);
            }

            if let Ok(mut arrow) = arrow_query.get_mut(*child) {
                arrow.rotation = Quat::from_rotation_z(theta + std::f32::consts::FRAC_PI_2);
                continue;
            }
        }

        *visibility = Visibility::Visible;
    }
}

fn decorate(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &DirectionIndicator), Added<DirectionIndicator>>,
    fonts: Res<Fonts>,
) {
    for (entity, indicator) in query.iter() {
        let arrow = commands
            .spawn((
                ColorMesh2dBundle {
                    mesh: meshes.add(util::chevron(20., 20., 8.)).into(),
                    material: materials.add(indicator.settings.color.into()),
                    ..default()
                },
                DirectionIndicatorArrow,
            ))
            .id();

        let style = TextStyle {
            font: fonts.main.clone(),
            font_size: 16.,
            color: indicator.settings.color,
        };

        let label = indicator
            .settings
            .label
            .as_ref()
            .map_or_else(|| "".to_string(), |l| l.clone());

        let text = commands
            .spawn((
                Text2dBundle {
                    text: Text::from_section(label, style.clone())
                        .with_alignment(TextAlignment::Center),
                    ..default()
                },
                DirectionIndicatorLabel,
            ))
            .id();

        let distance_text = commands
            .spawn((
                Text2dBundle {
                    text: Text::from_section("", style.clone())
                        .with_alignment(TextAlignment::Center),
                    ..default()
                },
                DirectionIndicatorDistanceLabel,
            ))
            .id();

        commands
            .entity(entity)
            .insert(SpatialBundle {
                transform: Transform::from_xyz(0., 0., layer::UI),
                visibility: Visibility::Hidden,
                ..default()
            })
            .push_children(&[arrow, text, distance_text]);
    }
}

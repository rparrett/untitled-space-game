use bevy::{prelude::*, window::PrimaryWindow};
use interpolation::Ease;
use itertools::izip;
use rand::{distributions::Uniform, thread_rng, Rng};

use crate::{
    commodity::CommodityPrices, direction_indicator::DirectionIndicatorSettings, layer,
    scanner::Scanner, util, DespawnOnRestart, FuelTank, GameState, MovementSet, Player,
};

pub struct WarpNodePlugin;
impl Plugin for WarpNodePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WarpAnimation::default());

        app.add_system(spawn_nodes.in_schedule(OnEnter(GameState::Playing)));
        app.add_systems(
            (start_warp, end_warp, move_fade_sprite.after(MovementSet))
                .in_set(OnUpdate(GameState::Playing)),
        );
        app.add_systems(
            (warp, move_fade_sprite.after(MovementSet)).in_set(OnUpdate(GameState::Warping)),
        );
    }
}

#[derive(Component)]
pub struct WarpNode;
#[derive(Component)]
pub struct WarpFadeSprite;

#[derive(Resource)]
pub struct WarpAnimation {
    pub starfield_timer: Timer,
    pub fade_out_timer: Timer,
    pub fade_dwell_timer: Timer,
    pub fade_in_timer: Timer,
}

impl WarpAnimation {
    fn reset(&mut self) {
        self.fade_out_timer.pause();
        self.fade_out_timer.reset();
        self.fade_in_timer.pause();
        self.fade_in_timer.reset();
        self.fade_dwell_timer.pause();
        self.fade_dwell_timer.reset();
        self.starfield_timer.reset();
    }
}

impl Default for WarpAnimation {
    fn default() -> Self {
        let mut fade_out_timer = Timer::from_seconds(3., TimerMode::Once);
        fade_out_timer.pause();

        let mut fade_in_timer = Timer::from_seconds(3., TimerMode::Once);
        fade_in_timer.pause();

        let mut fade_dwell_timer = Timer::from_seconds(1., TimerMode::Once);
        fade_dwell_timer.pause();

        Self {
            starfield_timer: Timer::from_seconds(3., TimerMode::Once),
            fade_out_timer,
            fade_dwell_timer,
            fade_in_timer,
        }
    }
}

#[derive(Resource)]
pub struct WarpedTo(pub CommodityPrices);

fn spawn_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut scanner: ResMut<Scanner>,
) {
    let rng = thread_rng();

    let num = 3;

    let dist_range = Uniform::from(2600.0..3000.0);
    //let dist_range = Uniform::from(600.0..800.0);

    let labels = ('A'..).take(num).map(|c| c.to_string());
    let prices = (0..num).map(|_| CommodityPrices::new_random());
    let distances = rng.sample_iter(&dist_range).take(num);
    let angles = util::random_circular_f32_distribution(num as u32, 80., 360.);

    for (angle, distance, label, price) in izip!(angles, distances, labels, prices) {
        let angle = angle.to_radians();
        let (y, x) = angle.sin_cos();
        let pos = Vec3::new(x * distance, y * distance, layer::OBJECT);

        let entity = commands
            .spawn((
                ColorMesh2dBundle {
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
                },
                WarpNode,
                price,
                DirectionIndicatorSettings {
                    color: Color::ORANGE,
                    label: Some(label),
                },
                DespawnOnRestart,
            ))
            .id();

        scanner.warp_nodes.push(entity);
    }
}

fn start_warp(
    mut commands: Commands,
    query: Query<(&Transform, &CommodityPrices), With<WarpNode>>,
    query_player: Query<(&Transform, &FuelTank), With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut animation: ResMut<WarpAnimation>,
) {
    let (player_transform, fuel_tank) = query_player.single();

    if fuel_tank.current != fuel_tank.max {
        return;
    }

    for (node, prices) in query.iter() {
        let dist = player_transform
            .translation
            .truncate()
            .distance(node.translation.truncate());
        if dist < 80. {
            animation.starfield_timer.reset();

            next_state.set(GameState::Warping);

            commands.insert_resource(WarpedTo(prices.clone()));

            return;
        }
    }
}

fn warp(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut animation: ResMut<WarpAnimation>,
    mut fade_sprite_query: Query<&mut Sprite, With<WarpFadeSprite>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    animation.starfield_timer.tick(time.delta());
    if animation.starfield_timer.just_finished() {
        animation.fade_out_timer.unpause();

        let window = windows.single();

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::NONE,
                    custom_size: Some(Vec2::new(window.width(), window.height())),
                    ..default()
                },
                transform: Transform::from_xyz(0., 0., layer::FADE),
                ..default()
            },
            WarpFadeSprite,
        ));
    }

    animation.fade_out_timer.tick(time.delta());
    if animation.fade_out_timer.just_finished() {
        animation.fade_dwell_timer.unpause();
    } else if !animation.fade_out_timer.paused() && !animation.fade_out_timer.finished() {
        for mut sprite in fade_sprite_query.iter_mut() {
            let val = Ease::cubic_out(animation.fade_out_timer.percent());
            sprite.color = Color::rgba(0., 0., 0., val);
        }
    }

    animation.fade_dwell_timer.tick(time.delta());
    if animation.fade_dwell_timer.just_finished() {
        animation.fade_in_timer.unpause();

        next_state.set(GameState::Playing); // XXX shopping
    }
}

fn end_warp(
    mut commands: Commands,
    time: Res<Time>,
    mut animation: ResMut<WarpAnimation>,
    mut fade_sprite_query: Query<(Entity, &mut Sprite), With<WarpFadeSprite>>,
) {
    animation.fade_in_timer.tick(time.delta());
    if animation.fade_in_timer.just_finished() {
        for (entity, _) in fade_sprite_query.iter_mut() {
            commands.entity(entity).despawn();
        }

        animation.reset();
    } else if !animation.fade_in_timer.paused() && !animation.fade_in_timer.finished() {
        for (_, mut sprite) in fade_sprite_query.iter_mut() {
            let val = Ease::cubic_out(animation.fade_in_timer.percent_left());
            sprite.color = Color::rgba(0., 0., 0., val);
        }
    }
}

fn move_fade_sprite(
    camera_query: Query<&Transform, (With<Camera>, Without<WarpFadeSprite>)>,
    mut fade_sprite_query: Query<&mut Transform, With<WarpFadeSprite>>,
) {
    let camera = camera_query.single();

    for mut transform in fade_sprite_query.iter_mut() {
        transform.translation.x = camera.translation.x;
        transform.translation.y = camera.translation.y;
    }
}

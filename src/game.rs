use self::{
    items::{random_item, shoe},
    physics::{handle_collisions, Hooks, StuckItems},
    throw::{
        generate_item, handle_disabling, handle_stored_items, handle_throwable_removals,
        handle_throwing, Player, ThrowIndicator, Throwable,
    },
};
use crate::{utils::despawn_screen, Music};
use std::{collections::VecDeque, f32::consts::TAU, sync::Mutex, time::Duration};

use super::GameState;
use bevy::{
    audio::AudioSink,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        texture::ImageSampler,
    },
    sprite::{Material2d, MaterialMesh2dBundle},
    utils::{HashMap, HashSet},
};
use bevy_rapier2d::prelude::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use union_find::QuickFindUf;
use wgpu::{AddressMode, SamplerBorderColor};

mod collision_test;
mod items;
pub mod physics;
mod throw;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Game)
                .with_system(setup_graphics)
                .with_system(setup_physics)
                .with_system(setup_game),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Game)
                .with_system(handle_collisions)
                .with_system(handle_stored_items)
                .with_system(handle_throwing.after(handle_stored_items))
                .with_system(handle_item_dropping)
                .with_system(handle_death_timer)
                .with_system(handle_score_display)
                .with_system(handle_scoring_effect)
                .with_system(handle_lives_display)
                .with_system(handle_death)
                .with_system(handle_disabling)
                .with_system(handle_stickiness_effect)
                .with_system(customizing_sampler),
        )
        .add_system_to_stage(CoreStage::PostUpdate, handle_throwable_removals)
        .add_system_set(SystemSet::on_exit(GameState::Game).with_system(despawn_screen::<OnGame>));
    }
}

fn handle_death(
    players: Query<&Player>,
    audio_sinks: Res<Assets<AudioSink>>,
    mut music: ResMut<Music>,
    mut game_state: ResMut<State<GameState>>,
) {
    for player in players.iter() {
        if player.lives == 0 {
            let handle = music.0.take().unwrap();
            audio_sinks.get(&handle).unwrap().stop();
            game_state.set(GameState::Splash).unwrap();
        }
    }
}

#[derive(Component)]
pub struct OnGame;

fn setup_graphics(mut commands: Commands, asset_server: Res<AssetServer>) {
    let style = TextStyle {
        font: asset_server.load("fonts/MajorMonoDisplay-Regular.ttf"),
        font_size: 50.0,
        color: Color::WHITE,
    };
    commands.spawn((
        TextBundle::from_sections([
            TextSection {
                value: "Score:\n".to_owned(),
                style: style.clone(),
            },
            TextSection {
                value: "0".to_owned(),
                style: style.clone(),
            },
        ])
        .with_text_alignment(TextAlignment::TOP_LEFT)
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: UiRect {
                left: Val::Px(25.),
                top: Val::Percent(0.),
                ..default()
            },
            ..default()
        }),
        ScoreText,
        OnGame,
    ));

    commands.spawn((
        TextBundle::from_sections([
            TextSection {
                value: "Lives:\n".to_owned(),
                style: style.clone(),
            },
            TextSection {
                value: "".to_owned(),
                style: TextStyle {
                    font: asset_server.load("fonts/NotoEmoji-VariableFont_wght.ttf"),
                    font_size: 30.0,
                    color: Color::PINK,
                },
            },
        ])
        .with_text_alignment(TextAlignment::TOP_LEFT)
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: UiRect {
                left: Val::Px(25.),
                top: Val::Percent(10.),
                ..default()
            },
            ..default()
        }),
        LivesText,
        OnGame,
    ));
}

#[derive(Component)]
struct ScoreText;
fn handle_score_display(
    players: Query<&Player>,
    mut score_text: Query<&mut Text, With<ScoreText>>,
) {
    let player = players.single();
    let mut score = score_text.single_mut();
    score.sections[1].value = format!("{}", player.score);
}

#[derive(Component)]
struct LivesText;
fn handle_lives_display(
    players: Query<&Player>,
    mut lives_text: Query<&mut Text, With<LivesText>>,
) {
    let player = players.single();
    let mut lives_text = lives_text.single_mut();
    let mut lives = player.lives;
    let mut s = "".to_owned();
    while lives > 9 {
        s.push_str(&"♥".repeat(9));
        s.push('\n');
        lives -= 9;
    }
    s.push_str(&"♥".repeat(lives));
    lives_text.sections[1].value = s;
}

#[derive(Component)]
pub struct Disabler;

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_sinks: Res<Assets<AudioSink>>,
    mut music: ResMut<Music>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    audio: Res<Audio>,
) {
    // if let Some(handle) = music.0.take() {
    //     audio_sinks.get(&handle).map(|sink| sink.stop());
    // }
    // let music_asset = asset_server.load("music/StickyThrows.ogg");
    // music.0 = Some({
    //     let mut sink =
    //         audio.play_with_settings(music_asset, PlaybackSettings::LOOP.with_volume(0.4));
    //     sink.make_strong(&audio_sinks);
    //     sink
    // });

    commands
        .spawn((
            Player {
                lives: 9,
                score: 0,
                hold_timer: Timer::new(Duration::from_secs_f32(1.), TimerMode::Once),
                cooldown_timer: Timer::new(Duration::from_millis(200), TimerMode::Once),
                power_interval: Timer::new(Duration::from_millis(10), TimerMode::Repeating),
                disables: HashSet::new(),
                prev_mouse: None,
            },
            TransformBundle::from(Transform::from_xyz(SOURCE.x, SOURCE.y, 0.)),
            OnGame,
        ))
        .with_children(|child_builder| {
            child_builder.spawn((
                RigidBody::Fixed,
                Sensor,
                Collider::ball(200.),
                Disabler,
                TransformBundle::from(Transform::from_xyz(0., 0., 0.)),
            ));
        });

    commands.spawn((
        StuckItems {
            union_find: Mutex::new(QuickFindUf::from_iter(None)),
            map: HashMap::new(),
        },
        OnGame,
    ));

    let mut cur = Current {
        current: None,
        next: VecDeque::default(),
        rng: SmallRng::from_entropy(),
    };
    for _ in 0..3 {
        generate_item(
            &mut commands,
            &asset_server,
            &mut meshes,
            &mut custom_materials,
            &mut cur,
        );
    }
    commands.insert_resource(cur);
    commands.insert_resource(ThrowIndicator {
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
    });
    commands.insert_resource(ItemDropTimer {
        timer: Timer::from_seconds(2., TimerMode::Repeating),
        rng: SmallRng::from_entropy(),
    });
}

#[derive(Clone, Debug, Component)]
pub struct Destroyer;

#[derive(Component)]
pub struct Wall;

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(PhysicsHooksWithQueryResource(Box::new(Hooks)));

    // collision_test::test_collisions(commands);

    commands.spawn((
        Collider::cuboid(1000.0, 25.0),
        Destroyer,
        TransformBundle::from(Transform::from_xyz(0.0, -800.0, 0.0)),
        OnGame,
    ));

    // TODO: Make this repeat
    let texture_handle = asset_server.load("bricks.png");
    let mesh = Mesh::from(shape::Quad::new(2. * Vec2::new(20.0, 575.0)));

    commands.spawn((
        Collider::cuboid(20.0, 700.0),
        Restitution::coefficient(4.),
        Wall,
        OnGame,
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh.clone()).into(),
            material: materials.add(ColorMaterial::from(texture_handle.clone())),
            transform: Transform::from_xyz(-750.0, 0.0, 0.0),
            // .with_rotation(Quat::from_rotation_z(-TAU * 0.55)),
            ..default()
        },
    ));

    commands.spawn((
        Collider::cuboid(20.0, 700.0),
        Restitution::coefficient(4.),
        Wall,
        OnGame,
        MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material: materials.add(ColorMaterial::from(texture_handle)),
            transform: Transform::from_xyz(750.0, 0.0, 0.0),
            // .with_rotation(Quat::from_rotation_z(TAU * 0.55)),
            ..default()
        },
    ));

    // TODO: Create AI that throws items
    shoe(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut custom_materials,
        50.,
    )
    .insert((
        TransformBundle::from(Transform::from_xyz(ENEMY_SOURCE.x, ENEMY_SOURCE.y, 0.0)),
        Velocity {
            linvel: Vec2::new(-100.0, 150.0),
            angvel: 0.,
        },
        Throwable::new(None, true),
        OnGame,
    ));
}

#[derive(Resource)]
pub struct Current {
    pub current: Option<Entity>,
    pub next: VecDeque<Entity>,
    pub rng: SmallRng,
}

pub const STORAGE: Vec2 = Vec2::new(-900.0, -400.0);
pub const SOURCE: Vec2 = Vec2::new(-600.0, -375.0);
pub const ENEMY_SOURCE: Vec2 = Vec2::new(600.0, -375.0);

#[derive(Component)]
pub struct DeathTimer(pub Timer);
fn handle_death_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: Query<(&mut DeathTimer, Entity)>,
) {
    for (mut timer, cur) in timers.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            commands.entity(cur).despawn_recursive();
        }
    }
}

#[derive(Resource)]
pub struct ItemDropTimer {
    timer: Timer,
    rng: SmallRng,
}
fn handle_item_dropping(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    time: Res<Time>,
    mut timer: ResMut<ItemDropTimer>,
) {
    let commands = &mut commands;
    if timer.timer.tick(time.delta()).just_finished() {
        let x = timer.rng.gen_range(-350..=350);
        let y = timer.rng.gen_range(0..=100);
        let transform = Transform::from_xyz(x as f32, 550. + y as f32, 5.);
        let angle = TAU / 8.;
        random_item(
            &mut timer.rng,
            commands,
            &asset_server,
            &mut meshes,
            &mut custom_materials,
        )
        .insert((
            TransformBundle::from(transform),
            GravityScale(0.8),
            Throwable::new(None, true),
            ExternalImpulse {
                impulse: Vec2::ZERO,
                torque_impulse: timer.rng.gen_range(-angle..=angle),
            },
            OnGame,
        ));
    }
}

#[derive(Component)]
pub struct ScoringEffect {
    pub multiplier: usize,
    pub points: usize,
}
fn handle_scoring_effect(mut scoring_text: Query<(&mut Text, &ScoringEffect, &DeathTimer)>) {
    for (mut text, effect, timer) in scoring_text.iter_mut() {
        let mut style = &mut text.sections[0].style;
        style.font_size = effect.points as f32 + timer.0.percent() * 30.;
        if effect.multiplier > 1 {
            style.color = Color::PURPLE;
        }
    }
}

#[derive(Component)]
struct StickyEffect;
fn handle_stickiness_effect(
    throwables: Query<(&Throwable, &Handle<CustomMaterial>)>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
) {
    for (throwable, material) in throwables.iter() {
        if let Some(material) = custom_materials.get_mut(material) {
            if throwable.sticky {
                material.sticky = 1;
            } else {
                material.sticky = 0;
            }
        }
    }
}

impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sticky.wgsl".into()
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    sticky: i32,
    #[texture(2)]
    #[sampler(3)]
    color_texture: Handle<Image>,
}

// TODO: There was default sampler, but didn't work
fn customizing_sampler(
    mut events: EventReader<AssetEvent<Image>>,
    mut assets: ResMut<Assets<Image>>,
) {
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(texture) = assets.get_mut(handle) {
                    let mut linear = ImageSampler::linear();
                    if let ImageSampler::Descriptor(linear) = &mut linear {
                        linear.address_mode_u = AddressMode::ClampToBorder;
                        linear.address_mode_v = AddressMode::ClampToBorder;
                        linear.border_color = Some(SamplerBorderColor::TransparentBlack);
                    }

                    texture.sampler_descriptor = linear;
                }
            }
            AssetEvent::Modified { handle: _ } => {}
            AssetEvent::Removed { handle: _ } => {}
        }
    }
}

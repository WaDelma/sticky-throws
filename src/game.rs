use self::{
    items::{random_item, shoe},
    physics::{handle_collisions, Hooks, StuckItems},
    throw::{
        generate_item, handle_disabling, handle_stored_items, handle_throwable_removals,
        handle_throwing, Player, ThrowIndicator, Throwable,
    },
};
use crate::utils::despawn_screen;
use std::{collections::VecDeque, f32::consts::TAU, sync::Mutex, time::Duration};

use super::GameState;
use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    utils::{HashMap, HashSet},
};
use bevy_rapier2d::prelude::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use union_find::QuickFindUf;

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
                .with_system(handle_throwable_removals)
                .with_system(handle_stickiness_effect),
        )
        .add_system_set(SystemSet::on_exit(GameState::Game).with_system(despawn_screen::<OnGame>));
    }
}

fn handle_death(players: Query<&Player>, mut game_state: ResMut<State<GameState>>) {
    for player in players.iter() {
        if player.lives == 0 {
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
    commands
        .spawn_bundle(
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
                    ..default()
                },
                ..default()
            }),
        )
        .insert(ScoreText)
        .insert(OnGame);

    commands
        .spawn_bundle(
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
        )
        .insert(LivesText)
        .insert(OnGame);
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

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn()
        .insert(Player {
            lives: 9,
            score: 0,
            hold_timer: Timer::new(Duration::from_secs_f32(1.), false),
            cooldown_timer: Timer::new(Duration::from_millis(200), false),
            power_interval: Timer::new(Duration::from_millis(10), true),
            disables: HashSet::new(),
            prev_mouse: None,
        })
        .with_children(|child_builder| {
            child_builder
                .spawn()
                .insert(RigidBody::Fixed)
                .insert(Sensor)
                .insert(Collider::ball(200.))
                .insert(Disabler)
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0., 0., 0.)));
        })
        .insert_bundle(TransformBundle::from(Transform::from_xyz(
            SOURCE.x, SOURCE.y, 0.,
        )))
        .insert(OnGame);

    commands
        .spawn()
        .insert(StuckItems {
            union_find: Mutex::new(QuickFindUf::from_iter(None)),
            map: HashMap::new(),
        })
        .insert(OnGame);

    let mut cur = Current {
        current: None,
        next: VecDeque::default(),
        rng: SmallRng::from_entropy(),
    };
    for _ in 0..3 {
        generate_item(&mut commands, &asset_server, &mut cur);
    }
    commands.insert_resource(cur);
    commands.insert_resource(ThrowIndicator {
        timer: Timer::from_seconds(0.1, true),
    });
    commands.insert_resource(ItemDropTimer {
        timer: Timer::from_seconds(2., true),
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
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(PhysicsHooksWithQueryResource(Box::new(Hooks)));

    // collision_test::test_collisions(commands);

    commands
        .spawn()
        .insert(Collider::cuboid(1000.0, 25.0))
        .insert(Destroyer)
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -600.0, 0.0)))
        .insert(OnGame);

    // TODO: Make this repeat
    let texture_handle = asset_server.load("bricks.png");
    let mesh = Mesh::from(shape::Quad::new(2. * Vec2::new(20.0, 575.0)));

    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 575.0))
        .insert(Restitution::coefficient(4.))
        .insert(Wall)
        .insert_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(mesh.clone()).into(),
            material: materials.add(ColorMaterial::from(texture_handle.clone())),
            transform: Transform::from_xyz(-675.0, 0.0, 0.0)
                .with_rotation(Quat::from_rotation_z(-TAU * 0.55)),
            ..default()
        })
        .insert(OnGame);

    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 575.0))
        .insert(Restitution::coefficient(4.))
        .insert(Wall)
        .insert_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material: materials.add(ColorMaterial::from(texture_handle)),
            transform: Transform::from_xyz(675.0, 0.0, 0.0)
                .with_rotation(Quat::from_rotation_z(TAU * 0.55)),
            ..default()
        })
        .insert(OnGame);

    // TODO: Create AI that throws items
    shoe(&mut commands, &asset_server, 50.)
        .insert_bundle(TransformBundle::from(Transform::from_xyz(
            ENEMY_SOURCE.x,
            ENEMY_SOURCE.y,
            0.0,
        )))
        .insert(Velocity {
            linvel: Vec2::new(-100.0, 150.0),
            angvel: 0.,
        })
        .insert(Throwable::new(None, true))
        .insert(OnGame);
}

pub struct Current {
    pub current: Option<Entity>,
    pub next: VecDeque<Entity>,
    pub rng: SmallRng,
}

pub const STORAGE: Vec2 = Vec2::new(-900.0, -400.0);
pub const SOURCE: Vec2 = Vec2::new(-625.0, -375.0);
pub const ENEMY_SOURCE: Vec2 = Vec2::new(625.0, -375.0);

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

pub struct ItemDropTimer {
    timer: Timer,
    rng: SmallRng,
}
fn handle_item_dropping(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut timer: ResMut<ItemDropTimer>,
) {
    let commands = &mut commands;
    if timer.timer.tick(time.delta()).just_finished() {
        let x = timer.rng.gen_range(-350..=350);
        let y = timer.rng.gen_range(0..=100);
        let transform = Transform::from_xyz(x as f32, 550. + y as f32, 5.);
        let angle = TAU / 8.;
        random_item(&mut timer.rng, &asset_server, commands)
            .insert_bundle(TransformBundle::from(transform))
            .insert(Throwable::new(None, true))
            .insert(ExternalImpulse {
                impulse: Vec2::ZERO,
                torque_impulse: timer.rng.gen_range(-angle..=angle),
            })
            .insert(OnGame);
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
    mut commands: Commands,
    sticky_effects: Query<&StickyEffect>,
    throwables: Query<(&Throwable, Entity, Option<&Children>)>,
) {
    for (throwable, entity, children) in throwables.iter() {
        if throwable.sticky {
            if children
                .and_then(|children| {
                    children
                        .iter()
                        .find(|entity| sticky_effects.contains(**entity))
                })
                .is_none()
            {
                commands.add(move |world: &mut World| {
                    if let (Some(mut sprite), Some(texture)) = (
                        world.get::<Sprite>(entity).cloned(),
                        world.get::<Handle<Image>>(entity).cloned(),
                    ) {
                        sprite.color = Color::YELLOW_GREEN;
                        sprite.custom_size = sprite.custom_size.map(|s| s * 1.1);
                        world.entity_mut(entity).with_children(|child_builder| {
                            child_builder.spawn().insert(StickyEffect).insert_bundle(
                                SpriteBundle {
                                    sprite,
                                    texture,
                                    transform: Transform::from_xyz(0., 0., 0.),
                                    ..default()
                                },
                            );
                        });
                    }
                });
            }
        } else if let Some(children) = children {
            for child in children {
                if sticky_effects.contains(*child) {
                    commands.entity(*child).despawn();
                }
            }
        }
    }
}

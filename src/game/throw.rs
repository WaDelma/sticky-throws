use std::time::Duration;

use crate::{
    game::{Current, DeathTimer, OnGame, SOURCE, STORAGE},
    utils::{screen_to_world, EntityCommandsExt},
    MainCamera,
};
use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::{camera::RenderTarget, render_resource::Texture},
    utils::HashSet,
};
use bevy_rapier2d::prelude::*;

use super::items::random_item;

#[derive(Clone, Debug, Component)]
pub struct Throwable {
    pub player: Option<Entity>,
    pub multiplier: usize,
    pub stuck: bool,
    pub sticky: bool,
}

impl Throwable {
    pub fn new(player: Option<Entity>, sticky: bool) -> Self {
        Self {
            player,
            multiplier: 1,
            stuck: false,
            sticky,
        }
    }
}

#[derive(Component)]
pub struct Player {
    pub disables: HashSet<Entity>,
    pub lives: usize,
    pub score: usize,
    pub hold_timer: Timer,
    pub cooldown_timer: Timer,
    pub power_interval: Timer,
    pub prev_mouse: Option<Vec2>,
}

pub struct ThrowIndicator {
    pub timer: Timer,
}

#[derive(Component)]
pub struct Ghost(pub Entity);

pub fn handle_throwing(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    buttons: Res<Input<MouseButton>>,
    mut current: ResMut<Current>,
    mut indicator: ResMut<ThrowIndicator>,
    windows: Res<Windows>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    restitutions: Query<&Restitution>,
    collider_mass_props: Query<&ColliderMassProperties>,
    colliders: Query<&Collider>,
    transforms: Query<&Transform>,
    global_transforms: Query<&GlobalTransform>,
    childrens: Query<&Children>,
    mut players: Query<(&mut Player, &Transform, Entity)>,
) {
    let (camera, camera_transform) = cameras.single();

    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    let target = window
        .cursor_position()
        .map(|screen_pos| screen_to_world(window, camera, camera_transform, screen_pos))
        .unwrap_or(Vec2::ZERO);

    let dir = (target - SOURCE).normalize_or_zero();

    for (mut player, pos, player_entity) in players.iter_mut() {
        if player.cooldown_timer.tick(time.delta()).finished() {
            if buttons.just_pressed(MouseButton::Left) && current.current.is_none() {
                generate_item(&mut commands, &asset_server, &mut current);
                select_first_item(&mut commands, &mut current);
                player.hold_timer.reset();
            }

            let impulse = dir * 140. + dir * player.hold_timer.percent() * 200.;

            if buttons.pressed(MouseButton::Left) {
                if let Some(cur) = current.current {
                    player.hold_timer.tick(time.delta());

                    if player.power_interval.tick(time.delta()).just_finished() {
                        let prev_target = player.prev_mouse.unwrap_or(target);
                        let from = (prev_target - pos.translation.xy()).normalize_or_zero();
                        let to = (target - pos.translation.xy()).normalize_or_zero();
                        let torque_impulse = from.angle_between(to) * 0.3;
                        commands.entity(cur).insert(ExternalImpulse {
                            impulse: Vec2::ZERO,
                            torque_impulse,
                        });

                        player.prev_mouse = Some(target);
                    }

                    if indicator.timer.tick(time.delta()).just_finished() {
                        let indicator_size = (0.1 + 0.9 * player.hold_timer.percent()).powf(2.);
                        spawn_indicator(
                            &mut commands,
                            indicator_size,
                            &asset_server,
                            impulse,
                            cur,
                            dir,
                            &childrens,
                            &transforms,
                            &global_transforms,
                            &restitutions,
                            &colliders,
                            &collider_mass_props,
                        );
                    }
                }
            }

            if buttons.just_released(MouseButton::Left) && player.disables.is_empty() {
                if let Some(cur) = current.current.take() {
                    player.disables.insert(cur);
                    player.cooldown_timer.reset();
                    player.prev_mouse = None;
                    commands
                        .entity(cur)
                        .remove::<IgnoreCollisions>()
                        .insert(GravityScale(1.))
                        .insert(Throwable::new(Some(player_entity), false))
                        .insert(LockedAxes::empty())
                        .insert(ExternalImpulse {
                            impulse,
                            torque_impulse: 0.,
                        });
                }
            }
        }
    }
}

fn spawn_indicator(
    commands: &mut Commands,
    indicator_size: f32,
    asset_server: &AssetServer,
    impulse: Vec2,
    cur: Entity,
    dir: Vec2,
    childrens: &Query<&Children>,
    transforms: &Query<&Transform>,
    global_transforms: &Query<&GlobalTransform>,
    restitutions: &Query<&Restitution>,
    colliders: &Query<&Collider>,
    collider_mass_props: &Query<&ColliderMassProperties>,
) {
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(150., 150.) * indicator_size),
                ..default()
            },
            texture: asset_server.load("indicator.png"),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Ghost(cur))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
        .insert(DeathTimer(Timer::new(Duration::from_secs_f32(0.5), false)))
        .insert(ExternalImpulse {
            impulse,
            torque_impulse: 0.,
        })
        .with_children(|children| {
            if let Ok(cur_children) = childrens.get(cur) {
                for &child in cur_children {
                    children
                        .spawn()
                        .insert(ActiveEvents::COLLISION_EVENTS)
                        .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
                        .maybe_insert(transforms.get(child).ok().cloned())
                        .maybe_insert(global_transforms.get(child).ok().cloned())
                        .maybe_insert(restitutions.get(child).ok().cloned())
                        .maybe_insert(colliders.get(child).ok().cloned())
                        .maybe_insert(collider_mass_props.get(child).ok().cloned());
                }
            }
        })
        .maybe_insert(transforms.get(cur).ok().cloned().map(|t| Transform {
            translation: Vec3::new(t.translation.x, t.translation.y, 10.)
                + Vec3::new(dir.x, dir.y, 0.) * 50.,
            ..t
        }))
        .maybe_insert(global_transforms.get(cur).ok().cloned())
        .maybe_insert(restitutions.get(cur).ok().cloned())
        .maybe_insert(colliders.get(cur).ok().cloned())
        .maybe_insert(collider_mass_props.get(cur).ok().cloned())
        .insert(OnGame);
}

pub fn handle_stored_items(mut commands: Commands, current: ResMut<Current>) {
    for (i, &e) in current.next.iter().enumerate() {
        commands.add(move |world: &mut World| {
            let pos = STORAGE + Vec2::new(0., 100.) * i as f32;
            world
                .entity_mut(e)
                .insert(Transform::from_xyz(pos.x, pos.y, 0.).with_scale(Vec3::ONE * 0.5))
                .insert(Velocity {
                    linvel: Vec2::ZERO,
                    angvel: 0.,
                });
        });
    }
}

#[derive(Component)]
pub struct IgnoreCollisions;
pub fn generate_item(commands: &mut Commands, asset_server: &AssetServer, current: &mut Current) {
    let pos = STORAGE + Vec2::new(0., 75.) * current.next.len() as f32;
    let transform = Transform::from_xyz(pos.x, pos.y, 5.).with_scale(Vec3::ONE * 0.5);
    let entity = random_item(&mut current.rng, asset_server, commands)
        .insert(GravityScale(0.))
        .insert_bundle(TransformBundle::from(transform))
        .insert(OnGame)
        .insert(IgnoreCollisions)
        .id();
    current.next.push_back(entity);
}

fn select_first_item(commands: &mut Commands, cur: &mut Current) {
    if cur.current.is_some() {
        return;
    }
    if let Some(e) = cur.next.pop_front() {
        commands.add(move |world: &mut World| {
            *world.get_mut(e).unwrap() =
                Transform::from_xyz(SOURCE.x, SOURCE.y, 0.).with_scale(Vec3::ONE);
        });
        commands.entity(e).insert(LockedAxes::TRANSLATION_LOCKED);
        cur.current = Some(e);
    }
}

pub fn handle_throwable_removals(
    removals: RemovedComponents<Throwable>,
    mut players: Query<&mut Player>,
) {
    let mut player = players.single_mut();
    for entity in removals.iter() {
        player.disables.remove(&entity);
    }
}

pub fn handle_disabling(
    player: Query<&Player>,
    cur: Res<Current>,
    mut sprites: Query<&mut Sprite>,
) {
    if let Some(cur) = cur.current {
        let color = &mut sprites.get_mut(cur).unwrap().color;
        if player.single().disables.is_empty() {
            if let Color::Rgba { alpha, .. } = color {
                *alpha = 1.;
            }
        } else {
            if let Color::Rgba { alpha, .. } = color {
                *alpha = 0.5;
            }
        }
    }
}

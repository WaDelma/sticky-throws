use std::time::Duration;

use crate::{
    random_item, screen_to_world, Current, DeathTimer, EntityCommandsExt, Ghost, MainCamera, Throw,
    ThrowIndicator, Throwable, SOURCE, STORAGE,
};
use bevy::{prelude::*, render::camera::RenderTarget};
use bevy_rapier2d::prelude::*;

pub fn handle_throwing(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    buttons: Res<Input<MouseButton>>,
    mut throw: ResMut<Throw>,
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

    if throw.cooldown_timer.tick(time.delta()).finished() {
        if buttons.just_pressed(MouseButton::Left) {
            generate_item(&mut commands, &asset_server, &mut current);
            select_first_item(&mut commands, &mut current);
            throw.hold_timer.reset();
        }

        let impulse = dir * 100. + dir * throw.hold_timer.percent() * 220.;

        if buttons.pressed(MouseButton::Left) {
            if let Some(cur) = current.current {
                throw.hold_timer.tick(time.delta());

                if throw.power_interval.tick(time.delta()).just_finished() {
                    let prev_target = throw.prev_mouse.unwrap_or(target);
                    let from = (prev_target - SOURCE).normalize_or_zero();
                    let to = (target - SOURCE).normalize_or_zero();
                    let torque_impulse = from.angle_between(to) * 0.3;
                    commands.entity(cur).insert(ExternalImpulse {
                        impulse: Vec2::ZERO,
                        torque_impulse,
                    });

                    throw.prev_mouse = Some(target);
                }

                if indicator.timer.tick(time.delta()).just_finished() {
                    let indicator_size = (0.1 + 0.9 * throw.hold_timer.percent()).powf(2.);
                    spawn_indicator(
                        &mut commands,
                        indicator_size,
                        asset_server,
                        impulse,
                        cur,
                        dir,
                        childrens,
                        transforms,
                        global_transforms,
                        restitutions,
                        colliders,
                        collider_mass_props,
                    );
                }
            }
        }

        // TODO: Prevent throwing when there is item too close and ignore collisions before throwing
        if buttons.just_released(MouseButton::Left) {
            if let Some(cur) = current.current.take() {
                throw.cooldown_timer.reset();
                throw.prev_mouse = None;
                commands
                    .entity(cur)
                    .insert(GravityScale(1.))
                    .insert(Throwable)
                    .insert(LockedAxes::empty())
                    .insert(ExternalImpulse {
                        impulse,
                        torque_impulse: 0.,
                    });
            }
        }
    }
}

fn spawn_indicator(
    commands: &mut Commands,
    indicator_size: f32,
    asset_server: Res<AssetServer>,
    impulse: Vec2,
    cur: Entity,
    dir: Vec2,
    childrens: Query<&Children>,
    transforms: Query<&Transform>,
    global_transforms: Query<&GlobalTransform>,
    restitutions: Query<&Restitution>,
    colliders: Query<&Collider>,
    collider_mass_props: Query<&ColliderMassProperties>,
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
        .maybe_insert(collider_mass_props.get(cur).ok().cloned());
}

pub fn handle_stored_items(mut commands: Commands, current: ResMut<Current>) {
    for (i, &e) in current.next.iter().enumerate() {
        commands.add(move |world: &mut World| {
            let pos = STORAGE + Vec2::new(0., 100.) * i as f32;
            *world.get_mut(e).unwrap() =
                Transform::from_xyz(pos.x, pos.y, 0.).with_scale(Vec3::ONE * 0.5);
        });
    }
}

pub fn generate_item(commands: &mut Commands, asset_server: &AssetServer, current: &mut Current) {
    let pos = STORAGE + Vec2::new(0., 75.) * current.next.len() as f32;
    let transform = Transform::from_xyz(pos.x, pos.y, 0.).with_scale(Vec3::ONE * 0.5);

    current.next.push_back(
        random_item(&mut current.rng, asset_server, commands)
            .insert(GravityScale(0.))
            .insert_bundle(TransformBundle::from(transform))
            .id(),
    );
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

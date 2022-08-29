use bevy::{ecs::system::EntityCommands, prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::*;
use rand::Rng;

const DAMPING: Damping = Damping {
    linear_damping: 0.2,
    angular_damping: 0.2,
};

pub fn orange<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    asset_server: &AssetServer,
    radius: f32,
) -> EntityCommands<'w, 's, 'a> {
    let mut cmds = commands.spawn();
    cmds.insert(RigidBody::Dynamic)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
        .insert(Collider::ball(radius))
        .insert(Restitution::coefficient(0.8))
        .insert(ColliderMassProperties::Density(1.05))
        .insert(Ccd::enabled())
        .insert(DAMPING)
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(2., 3.) * radius),
                anchor: Anchor::Custom(Vec2::new(0., -0.175)),
                ..default()
            },
            texture: asset_server.load("orange.png"),
            ..default()
        });
    cmds
}

pub fn cereal_box<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    asset_server: &AssetServer,
    radius: f32,
) -> EntityCommands<'w, 's, 'a> {
    let mut cmds = commands.spawn();
    cmds.insert(RigidBody::Dynamic)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
        .insert(Collider::cuboid(0.75 * radius, radius))
        .insert(Restitution::coefficient(0.6))
        .insert(ColliderMassProperties::Density(0.45))
        .insert(Ccd::enabled())
        .insert(DAMPING)
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(1.5, 2.) * radius),
                ..default()
            },
            texture: asset_server.load("cereal.png"),
            ..default()
        });
    cmds
}

pub fn hammer<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    asset_server: &AssetServer,
    radius: f32,
) -> EntityCommands<'w, 's, 'a> {
    let mut cmds = commands.spawn();
    let handle_thickness = radius * 0.2;
    let head_thickness = radius * 0.25;
    let head_length = radius * 0.75;
    cmds.insert(RigidBody::Dynamic)
        .with_children(|children| {
            children
                .spawn()
                .insert(Restitution::coefficient(0.2))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
                .insert(Collider::cuboid(head_length, head_thickness))
                .insert(ColliderMassProperties::Density(3.5))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0., radius, 0.)));
            children
                .spawn()
                .insert(Restitution::coefficient(0.5))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
                .insert(Collider::cuboid(
                    handle_thickness,
                    radius - 0.5 * head_thickness,
                ))
                .insert(ColliderMassProperties::Density(0.8))
                .insert_bundle(TransformBundle::from(Transform::from_xyz(
                    0.0,
                    -0.5 * head_thickness,
                    0.,
                )));
        })
        .insert(Ccd::enabled())
        .insert(DAMPING)
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(2., 3.) * radius),
                anchor: Anchor::Custom(Vec2::new(0., -0.175)),
                ..default()
            },
            texture: asset_server.load("hammer.png"),
            ..default()
        });
    cmds
}

pub fn shoe<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    asset_server: &AssetServer,
    radius: f32,
) -> EntityCommands<'w, 's, 'a> {
    let mid = Vec2::new(radius, radius);
    let mut cmds = commands.spawn();

    cmds.insert(RigidBody::Dynamic)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
        .insert(Collider::convex_decomposition(
            &[
                Vec2::new(0., 0.) - mid,
                Vec2::new(0., 2. * radius) - mid,
                Vec2::new(radius, 2. * radius) - mid,
                Vec2::new(radius, 0.75 * radius) - mid,
                Vec2::new(2. * radius, 0.75 * radius) - mid,
                Vec2::new(2. * radius, 0.) - mid,
            ],
            &[[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 0]],
        ))
        .insert(Restitution::coefficient(1.))
        .insert(ColliderMassProperties::Density(1.15))
        .insert(Ccd::enabled())
        .insert(DAMPING)
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(2., 2.) * radius),
                ..default()
            },
            texture: asset_server.load("boot.png"),
            ..default()
        });
    cmds
}

pub fn random_item<'w, 's, 'a, R>(
    rng: &mut R,
    asset_server: &'a AssetServer,
    commands: &'a mut Commands<'w, 's>,
) -> EntityCommands<'w, 's, 'a>
where
    R: Rng,
{
    match rng.gen_range(0..=3) {
        0 => shoe(commands, asset_server, 50.),
        1 => orange(commands, asset_server, 50.),
        2 => cereal_box(commands, asset_server, 75.),
        3 => hammer(commands, asset_server, 50.),
        _ => unreachable!(),
    }
}
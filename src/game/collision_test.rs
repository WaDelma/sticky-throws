use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::throw::Throwable;

pub fn test_collisions(mut commands: Commands) {
    // TODO: Test collisions
    let radius = 50.;
    let angle = 0. * 0.0625;
    let damp = Damping {
        linear_damping: 0.1,
        angular_damping: 1.,
    };
    commands
        .spawn_empty()
        .insert(RigidBody::Dynamic)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
        .insert(Collider::cuboid(0.75 * radius, radius))
        .insert(Restitution::coefficient(0.6))
        .insert(ColliderMassProperties::Density(0.45))
        .insert(GravityScale(0.))
        .insert(Ccd::enabled())
        .insert(damp)
        // .insert(SpriteBundle {
        //     sprite: Sprite {
        //         custom_size: Some(Vec2::new(1.5, 2.) * radius),
        //         ..default()
        //     },
        //     texture: asset_server.load("cereal.png"),
        //     ..default()
        // })
        .insert(TransformBundle::from(Transform {
            translation: Vec3::new(-200., 0., 0.),
            rotation: Quat::from_rotation_z(angle * TAU),
            ..default()
        }))
        .insert(Throwable::new(None, true))
        .insert(ExternalImpulse {
            impulse: Vec2::new(20., 0.),
            torque_impulse: 0.,
        });

    let angle = 1. * 0.0625;
    let radius = 50.;
    let handle_thickness = radius * 0.2;
    let head_thickness = radius * 0.25;
    let head_length = radius * 0.75;
    commands
        .spawn_empty()
        .insert(RigidBody::Dynamic)
        .with_children(|children| {
            children
                .spawn_empty()
                .insert(Restitution::coefficient(0.2))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
                .insert(Collider::cuboid(head_length, head_thickness))
                .insert(ColliderMassProperties::Density(3.5))
                .insert(TransformBundle::from(Transform::from_xyz(0.0, radius, 0.)));
            children
                .spawn_empty()
                .insert(Restitution::coefficient(0.5))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
                .insert(Collider::cuboid(
                    handle_thickness,
                    radius - 0.5 * head_thickness,
                ))
                .insert(ColliderMassProperties::Density(0.8))
                .insert(TransformBundle::from(Transform::from_xyz(
                    0.0,
                    -0.5 * head_thickness,
                    0.,
                )));
        })
        .insert(Throwable::new(None, true))
        .insert(TransformBundle::from(Transform {
            translation: Vec3::new(0., 0., 0.),
            rotation: Quat::from_rotation_z(angle * TAU),
            ..default()
        }))
        .insert(GravityScale(0.))
        .insert(Ccd::enabled())
        .insert(damp);
    // .insert(SpriteBundle {
    //     sprite: Sprite {
    //         custom_size: Some(Vec2::new(2., 3.) * radius),
    //         anchor: Anchor::Custom(Vec2::new(0., -0.175)),
    //         ..default()
    //     },
    //     texture: asset_server.load("hammer.png"),
    //     ..default()
    // });
    // let radius = 50.;
    // let angle = 7. * 0.0625;
    // let mid = Vec2::new(radius, radius);
    // commands
    //     .spawn()
    //     .insert(RigidBody::Dynamic)
    //     .insert(ActiveEvents::COLLISION_EVENTS)
    //     .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
    //     .insert(Collider::convex_decomposition(
    //         &[
    //             Vec2::new(0., 0.) - mid,
    //             Vec2::new(0., 2. * radius) - mid,
    //             Vec2::new(radius, 2. * radius) - mid,
    //             Vec2::new(radius, 0.75 * radius) - mid,
    //             Vec2::new(2. * radius, 0.75 * radius) - mid,
    //             Vec2::new(2. * radius, 0.) - mid,
    //         ],
    //         &[[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 0]],
    //     ))
    //     .insert(GravityScale(0.))
    //     .insert(Restitution::coefficient(1.))
    //     .insert(ColliderMassProperties::Density(1.15))
    //     .insert(Ccd::enabled())
    //     .insert(damp)
    //     // .insert(SpriteBundle {
    //     //     sprite: Sprite {
    //     //         custom_size: Some(Vec2::new(2., 2.) * radius),
    //     //         ..default()
    //     //     },
    //     //     texture: asset_server.load("boot.png"),
    //     //     ..default()
    //     // })
    //     .insert(TransformBundle::from(Transform {
    //         translation: Vec3::new(0., 0., 0.),
    //         rotation: Quat::from_rotation_z(angle * TAU),
    //         ..default()
    //     }))
    //     .insert(Throwable)
    //     .insert(ExternalImpulse {
    //         impulse: Vec2::ZERO,
    //         torque_impulse: 0.,
    //     });
}

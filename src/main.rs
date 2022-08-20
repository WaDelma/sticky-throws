use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use physics::{handle_collisions, Hooks};

mod physics;

// TODO: Implement item throwing feature
// TODO: Implement AI that throws items
// TODO: Create bunch of different items
// TODO: Polish gameplay
// TODO: Implement rendering
// TODO: Implement sounds

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<&ImpulseJoint>::pixels_per_meter(
            100.0,
        ))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_system(handle_collisions)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

#[derive(Clone, Debug, Component)]
struct Throwable;

fn setup_physics(mut commands: Commands) {
    commands.insert_resource(PhysicsHooksWithQueryResource(Box::new(Hooks)));

    commands
        .spawn()
        .insert(Collider::cuboid(500.0, 50.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)));

    square(
        &mut commands,
        50.,
        Transform::from_xyz(-300.0, 200.0, 0.0),
        Velocity {
            linvel: Vec2::new(100.0, 150.0),
            angvel: 0.25,
        },
    );
    shoe(
        &mut commands,
        50.,
        Transform::from_xyz(300.0, 200.0, 0.0),
        Velocity {
            linvel: Vec2::new(-100.0, 150.0),
            angvel: 0.,
        },
    );
}

fn ball(commands: &mut Commands, radius: f32, transform: Transform, velocity: Velocity) {
    commands
        .spawn()
        .insert(Throwable)
        .insert(RigidBody::Dynamic)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
        .insert(Collider::ball(radius))
        .insert(Restitution::coefficient(0.7))
        .insert(velocity)
        .insert_bundle(TransformBundle::from(transform));
}

fn square(commands: &mut Commands, radius: f32, transform: Transform, velocity: Velocity) {
    commands
        .spawn()
        .insert(Throwable)
        .insert(RigidBody::Dynamic)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
        .insert(Collider::cuboid(radius, radius))
        .insert(Restitution::coefficient(0.7))
        .insert(velocity)
        .insert_bundle(TransformBundle::from(transform));
}

fn shoe(commands: &mut Commands, radius: f32, transform: Transform, velocity: Velocity) {
    let mid = Vec2::new(radius, radius);
    commands
        .spawn()
        .insert(Throwable)
        .insert(RigidBody::Dynamic)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
        .insert(Collider::convex_decomposition(
            &[
                Vec2::new(0., 0.) - mid,
                Vec2::new(0., 2. * radius) - mid,
                Vec2::new(radius, 2. * radius) - mid,
                Vec2::new(radius, radius) - mid,
                Vec2::new(1.75 * radius, radius) - mid,
                Vec2::new(1.75 * radius, 0.) - mid,
            ],
            &[[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 0]],
        ))
        .insert(Restitution::coefficient(0.7))
        .insert(velocity)
        .insert_bundle(TransformBundle::from(transform));
}

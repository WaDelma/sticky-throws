use std::collections::VecDeque;

use bevy::{
    ecs::system::EntityCommands, input::mouse::MouseMotion, prelude::*,
    render::camera::RenderTarget,
};
use bevy_rapier2d::prelude::*;
use physics::{handle_collisions, Hooks};
use rand::{rngs::SmallRng, Rng, SeedableRng};

mod physics;

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
        .add_startup_system(setup_game)
        .add_system(handle_collisions)
        .add_system(cursor_position)
        .add_system(handle_currents)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(MainCamera);
}

#[derive(Clone, Debug, Component)]
struct Throwable;

#[derive(Clone, Debug, Component)]
struct Destroyer;

fn setup_game(mut commands: Commands) {
    commands.insert_resource(Power(0.));

    let mut cur = Current(None, VecDeque::default(), SmallRng::from_entropy());
    for _ in 0..3 {
        gen_item(&mut commands, &mut cur);
    }
    commands.insert_resource(cur);
}

fn setup_physics(mut commands: Commands) {
    commands.insert_resource(PhysicsHooksWithQueryResource(Box::new(Hooks)));

    commands
        .spawn()
        .insert(Collider::cuboid(1000.0, 25.0))
        .insert(Destroyer)
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -350.0, 0.0)));

    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 1000.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(-500.0, 0.0, 0.0)));

    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 1000.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(500.0, 0.0, 0.0)));

    shoe(&mut commands, 50.)
        .insert_bundle(TransformBundle::from(Transform::from_xyz(
            300.0, 200.0, 0.0,
        )))
        .insert(Velocity {
            linvel: Vec2::new(-100.0, 150.0),
            angvel: 0.,
        })
        .insert(Throwable);
}

fn ball<'w, 's, 'a>(commands: &'a mut Commands<'w, 's>, radius: f32) -> EntityCommands<'w, 's, 'a> {
    let mut cmds = commands.spawn();
    cmds.insert(RigidBody::Dynamic)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
        .insert(Collider::ball(radius))
        .insert(Restitution::coefficient(0.8));
    cmds
}

fn cereal_box<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    radius: f32,
) -> EntityCommands<'w, 's, 'a> {
    let mut cmds = commands.spawn();
    cmds.insert(RigidBody::Dynamic)
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
        .insert(Collider::cuboid(0.75 * radius, radius))
        .insert(Restitution::coefficient(0.6));
    cmds
}

fn hammer<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    radius: f32,
) -> EntityCommands<'w, 's, 'a> {
    let mut cmds = commands.spawn();
    let handle_thickness = radius * 0.2;
    let head_thickness = radius * 0.25;
    let head_length = radius * 0.75;
    cmds.insert(RigidBody::Dynamic).with_children(|children| {
        children
            .spawn()
            .insert(Restitution::coefficient(0.2))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
            .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
            .insert(Collider::cuboid(head_length, head_thickness))
            .insert(ColliderMassProperties::Density(3.0))
            .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, radius, 0.)));
        children
            .spawn()
            .insert(Restitution::coefficient(0.5))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
            .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
            .insert(Collider::cuboid(
                handle_thickness,
                radius - 0.5 * head_thickness,
            ))
            .insert(ColliderMassProperties::Density(1.))
            .insert_bundle(TransformBundle::from(Transform::from_xyz(
                0.0,
                -0.5 * head_thickness,
                0.,
            )));
    });
    cmds
}

fn shoe<'w, 's, 'a>(commands: &'a mut Commands<'w, 's>, radius: f32) -> EntityCommands<'w, 's, 'a> {
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
        .insert(Restitution::coefficient(1.));
    cmds
}

#[derive(Default)]
struct Power(f32);

struct Current(Option<Entity>, VecDeque<Entity>, SmallRng);

#[derive(Component)]
struct MainCamera;

fn cursor_position(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    mut power: ResMut<Power>,
    mut current: ResMut<Current>,
    windows: Res<Windows>,
    mut motion_evr: EventReader<MouseMotion>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = cameras.single();

    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    let target = if let Some(screen_pos) = window.cursor_position() {
        screen_to_world(window, camera, camera_transform, screen_pos)
    } else {
        Vec2::ZERO
    };
    let source = Vec2::new(-350.0, -200.0);

    let dir = (target - source).normalize_or_zero();

    if buttons.just_pressed(MouseButton::Left) {
        gen_item(&mut commands, &mut current);
        select_item(&mut commands, &mut current);
    }

    if buttons.pressed(MouseButton::Left) {
        if let Some(cur) = current.0 {
            let delta = motion_evr.iter().fold(Vec2::ZERO, |a, b| a + b.delta);
            if delta != Vec2::ZERO {
                commands.entity(cur).insert(ExternalImpulse {
                    impulse: Vec2::ZERO,
                    torque_impulse: delta.angle_between(Vec2::X) / 100.,
                });
            };
        }
        power.0 += 5.;
        power.0 = power.0.min(300.);
    }

    if buttons.just_released(MouseButton::Left) {
        if let Some(cur) = current.0.take() {
            commands
                .entity(cur)
                .insert(GravityScale(1.))
                .insert(Throwable)
                .insert(LockedAxes::empty())
                .insert(ExternalImpulse {
                    impulse: dir * power.0,
                    torque_impulse: 0.,
                });
        }
        power.0 = 0.;
    }
}

fn handle_currents(mut commands: Commands, current: ResMut<Current>) {
    let storage = Vec2::new(-575.0, -250.0);
    for (i, &e) in current.1.iter().enumerate() {
        let pos = storage + Vec2::new(0., 100.) * i as f32;
        let transform = Transform::from_xyz(pos.x, pos.y, 0.).with_scale(Vec3::ONE * 0.5);

        commands.add(move |world: &mut World| {
            *world.get_mut::<Transform>(e).unwrap() = transform;
        });
    }
}

fn gen_item(commands: &mut Commands, current: &mut Current) {
    let storage = Vec2::new(-575.0, -250.0);
    let pos = storage + Vec2::new(0., 75.) * current.1.len() as f32;
    let transform = Transform::from_xyz(pos.x, pos.y, 0.).with_scale(Vec3::ONE * 0.5);

    let item = match current.2.gen_range(0..=3) {
        0 => shoe(commands, 50.),
        1 => ball(commands, 50.),
        2 => cereal_box(commands, 50.),
        3 => hammer(commands, 50.),
        _ => unreachable!(),
    }
    .insert(GravityScale(0.))
    .insert_bundle(TransformBundle::from(transform))
    .id();

    current.1.push_back(item);
}

fn select_item(commands: &mut Commands, cur: &mut Current) {
    let source = Vec2::new(-350.0, -200.0);
    if cur.0.is_none() {
        if let Some(e) = cur.1.pop_front() {
            commands.add(move |world: &mut World| {
                let mut transform = world.get_mut::<Transform>(e).unwrap();
                transform.translation = Vec3::new(source.x, source.y, 0.);
                transform.scale = Vec3::ONE;
            });
            commands.entity(e).insert(LockedAxes::TRANSLATION_LOCKED);
            cur.0 = Some(e);
        }
    }
}

fn screen_to_world(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    screen_pos: Vec2,
) -> Vec2 {
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);
    let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    world_pos.truncate()
}

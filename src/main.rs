use std::{collections::VecDeque, f32::consts::TAU, time::Duration};

use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    sprite::{Anchor, MaterialMesh2dBundle},
    window::PresentMode,
};
use bevy_rapier2d::prelude::*;
use physics::{handle_collisions, Hooks, PhysicsData};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use throw::{generate_item, handle_stored_items, handle_throwing};

mod physics;
mod throw;

// TODO: Polish gameplay
// TODO: Implement rendering
// TODO: Implement sounds

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Sticky throves".to_owned(),
            width: 1920.,
            height: 1080.,
            resizable: false,
            // TODO: Figure out how to scale game if resolution changes
            // mode: WindowMode::BorderlessFullscreen,
            present_mode: PresentMode::Immediate,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<PhysicsData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_startup_system(setup_game)
        .add_system(handle_collisions)
        .add_system(handle_throwing)
        .add_system(handle_stored_items)
        .add_system(handle_item_dropping)
        .add_system(handle_death_timer)
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

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Throw {
        hold_timer: Timer::new(Duration::from_secs_f32(1.), false),
        cooldown_timer: Timer::new(Duration::from_millis(200), false),
        power_interval: Timer::new(Duration::from_millis(10), true),
        prev_mouse: None,
    });

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

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(PhysicsHooksWithQueryResource(Box::new(Hooks)));

    // TODO: Test collisions
    // let radius = 50.;
    // let angle = 5. * 0.0625;
    // let damp = Damping {
    //     linear_damping: 0.1,
    //     angular_damping: 1.,
    // };
    // commands
    //     .spawn()
    //     .insert(RigidBody::Dynamic)
    //     .insert(ActiveEvents::COLLISION_EVENTS)
    //     .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
    //     .insert(Collider::cuboid(0.75 * radius, radius))
    //     .insert(Restitution::coefficient(0.6))
    //     .insert(ColliderMassProperties::Density(0.45))
    //     .insert(GravityScale(0.))
    //     .insert(Ccd::enabled())
    //     .insert(damp)
    //     // .insert_bundle(SpriteBundle {
    //     //     sprite: Sprite {
    //     //         custom_size: Some(Vec2::new(1.5, 2.) * radius),
    //     //         ..default()
    //     //     },
    //     //     texture: asset_server.load("cereal.png"),
    //     //     ..default()
    //     // })
    //     .insert_bundle(TransformBundle::from(Transform {
    //         translation: Vec3::new(-200., 0., 0.),
    //         rotation: Quat::from_rotation_z(angle * TAU),
    //         ..default()
    //     }))
    //     .insert(Throwable)
    //     .insert(ExternalImpulse {
    //         impulse: Vec2::new(100., 0.),
    //         torque_impulse: 0.,
    //     });
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
    //     // .insert_bundle(SpriteBundle {
    //     //     sprite: Sprite {
    //     //         custom_size: Some(Vec2::new(2., 2.) * radius),
    //     //         ..default()
    //     //     },
    //     //     texture: asset_server.load("boot.png"),
    //     //     ..default()
    //     // })
    //     .insert_bundle(TransformBundle::from(Transform {
    //         translation: Vec3::new(0., 0., 0.),
    //         rotation: Quat::from_rotation_z(angle * TAU),
    //         ..default()
    //     }))
    //     .insert(Throwable)
    //     .insert(ExternalImpulse {
    //         impulse: Vec2::ZERO,
    //         torque_impulse: 0.,
    //     });

    commands
        .spawn()
        .insert(Collider::cuboid(1000.0, 25.0))
        .insert(Destroyer)
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -600.0, 0.0)));

    // TODO: Make this repeat
    let texture_handle = asset_server.load("bricks.png");
    let mesh = Mesh::from(shape::Quad::new(2. * Vec2::new(20.0, 575.0)));

    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 575.0))
        .insert(Restitution::coefficient(4.))
        .insert_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(mesh.clone()).into(),
            material: materials.add(ColorMaterial::from(texture_handle.clone())),
            transform: Transform::from_xyz(-675.0, 0.0, 0.0)
                .with_rotation(Quat::from_rotation_z(-TAU * 0.55)),
            ..default()
        });

    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 575.0))
        .insert(Restitution::coefficient(4.))
        .insert_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material: materials.add(ColorMaterial::from(texture_handle)),
            transform: Transform::from_xyz(675.0, 0.0, 0.0)
                .with_rotation(Quat::from_rotation_z(TAU * 0.55)),
            ..default()
        });

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
        .insert(Throwable);
}

const DAMPING: Damping = Damping {
    linear_damping: 0.2,
    angular_damping: 0.2,
};

fn orange<'w, 's, 'a>(
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

fn cereal_box<'w, 's, 'a>(
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

fn hammer<'w, 's, 'a>(
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
                .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, radius, 0.)));
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

fn shoe<'w, 's, 'a>(
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

#[derive(Default)]
pub struct Throw {
    hold_timer: Timer,
    cooldown_timer: Timer,
    power_interval: Timer,
    prev_mouse: Option<Vec2>,
}

pub struct Current {
    current: Option<Entity>,
    next: VecDeque<Entity>,
    rng: SmallRng,
}

#[derive(Component)]
pub struct MainCamera;

const STORAGE: Vec2 = Vec2::new(-900.0, -400.0);
const SOURCE: Vec2 = Vec2::new(-625.0, -375.0);
const ENEMY_SOURCE: Vec2 = Vec2::new(625.0, -375.0);

pub struct ThrowIndicator {
    timer: Timer,
}

#[derive(Component)]
pub struct Ghost(Entity);

#[derive(Component)]
pub struct DeathTimer(Timer);

#[derive(Component)]
pub struct Stuck;

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

trait EntityCommandsExt {
    fn maybe_insert<C>(&mut self, c: Option<C>) -> &mut Self
    where
        C: Component;
}

impl<'w, 's, 'a> EntityCommandsExt for EntityCommands<'w, 's, 'a> {
    fn maybe_insert<C>(&mut self, c: Option<C>) -> &mut Self
    where
        C: Component,
    {
        if let Some(c) = c {
            self.insert(c);
        }
        self
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
        let transform = Transform::from_xyz(x as f32, 600. + y as f32, 0.);
        let angle = TAU / 8.;
        random_item(&mut timer.rng, &asset_server, commands)
            .insert_bundle(TransformBundle::from(transform))
            .insert(Throwable)
            .insert(ExternalImpulse {
                impulse: Vec2::ZERO,
                torque_impulse: timer.rng.gen_range(-angle..=angle),
            });
    }
}

fn random_item<'w, 's, 'a, R>(
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

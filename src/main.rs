use std::{collections::VecDeque, f32::consts::TAU, time::Duration};

use bevy::{
    ecs::system::EntityCommands, input::mouse::MouseMotion, prelude::*,
    render::camera::RenderTarget, sprite::Anchor, window::PresentMode,
};
use bevy_rapier2d::prelude::*;
use physics::{handle_collisions, Hooks, PhysicsData};
use rand::{rngs::SmallRng, Rng, SeedableRng};

mod physics;

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
    commands.insert_resource(Power(0.));

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
        timer: Timer::from_seconds(5.0, false),
        rng: SmallRng::from_entropy(),
    });
}

fn setup_physics(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PhysicsHooksWithQueryResource(Box::new(Hooks)));

    commands
        .spawn()
        .insert(Collider::cuboid(1000.0, 25.0))
        .insert(Destroyer)
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -600.0, 0.0)));

    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 550.0))
        .insert(Restitution::coefficient(1.))
        .insert_bundle(TransformBundle::from(
            Transform::from_xyz(-650.0, 0.0, 0.0).with_rotation(Quat::from_rotation_z(-TAU * 0.55)),
        ));

    commands
        .spawn()
        .insert(Collider::cuboid(20.0, 550.0))
        .insert(Restitution::coefficient(1.))
        .insert_bundle(TransformBundle::from(
            Transform::from_xyz(650.0, 0.0, 0.0).with_rotation(Quat::from_rotation_z(TAU * 0.55)),
        ));

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
        .insert(ColliderMassProperties::Density(0.4))
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
struct Power(f32);

struct Current {
    current: Option<Entity>,
    next: VecDeque<Entity>,
    rng: SmallRng,
}

#[derive(Component)]
struct MainCamera;

const STORAGE: Vec2 = Vec2::new(-900.0, -400.0);
const SOURCE: Vec2 = Vec2::new(-550.0, -300.0);
const ENEMY_SOURCE: Vec2 = Vec2::new(550.0, -300.0);

struct ThrowIndicator {
    timer: Timer,
}

#[derive(Component)]
pub struct Ghost(Entity);

#[derive(Component)]
pub struct DeathTimer(Timer);

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

fn handle_throwing(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    buttons: Res<Input<MouseButton>>,
    mut power: ResMut<Power>,
    mut current: ResMut<Current>,
    mut indicator: ResMut<ThrowIndicator>,
    windows: Res<Windows>,
    mut mouse_motions: EventReader<MouseMotion>,
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

    if buttons.just_pressed(MouseButton::Left) {
        generate_item(&mut commands, &asset_server, &mut current);
        select_first_item(&mut commands, &mut current);
    }

    if buttons.pressed(MouseButton::Left) {
        if let Some(cur) = current.current {
            // Make moving mouse before throwing moving rotate the item
            // TODO: After making window large this became crazy
            let delta = mouse_motions.iter().fold(Vec2::ZERO, |a, b| a + b.delta);
            if delta != Vec2::ZERO {
                // TODO: Cap rotation velocity
                commands.entity(cur).insert(ExternalImpulse {
                    impulse: Vec2::ZERO,
                    torque_impulse: delta.angle_between(Vec2::X) / 1000.,
                });
            };
            // TODO: Scale by elapsed time?
            power.0 += 0.2;
            let max = 300.;
            power.0 = power.0.min(max);
            let percentage_power = power.0 / max;

            if indicator.timer.tick(time.delta()).just_finished() {
                // TODO: This doesn't really work
                let sim_scale = 1.;
                let indicator_size = percentage_power.powf(2.);
                commands
                    .spawn()
                    .insert_bundle(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(200., 200.) * indicator_size),
                            ..default()
                        },
                        texture: asset_server.load("indicator.png"),
                        ..default()
                    })
                    .insert(RigidBody::Dynamic)
                    .insert(Ghost(cur))
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(ActiveHooks::FILTER_CONTACT_PAIRS)
                    .insert(GravityScale(sim_scale))
                    .insert(DeathTimer(Timer::new(Duration::from_secs_f32(0.5), false)))
                    .insert(ExternalImpulse {
                        impulse: dir * power.0 * sim_scale,
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
                    .insert(
                        collider_mass_props
                            .get(cur)
                            .ok()
                            .cloned()
                            .map(|m| match m {
                                ColliderMassProperties::Density(d) => {
                                    ColliderMassProperties::Density(d * sim_scale)
                                }
                                ColliderMassProperties::Mass(m) => {
                                    ColliderMassProperties::Mass(m * sim_scale)
                                }
                                ColliderMassProperties::MassProperties(mp) => {
                                    ColliderMassProperties::MassProperties(MassProperties {
                                        local_center_of_mass: mp.local_center_of_mass,
                                        mass: mp.mass * sim_scale,
                                        principal_inertia: mp.principal_inertia * sim_scale,
                                    })
                                }
                            })
                            .unwrap_or_else(|| ColliderMassProperties::Density(sim_scale)),
                    );
            }
        }
    }

    if buttons.just_released(MouseButton::Left) {
        if let Some(cur) = current.current.take() {
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

fn handle_stored_items(mut commands: Commands, current: ResMut<Current>) {
    for (i, &e) in current.next.iter().enumerate() {
        commands.add(move |world: &mut World| {
            let pos = STORAGE + Vec2::new(0., 100.) * i as f32;
            *world.get_mut(e).unwrap() =
                Transform::from_xyz(pos.x, pos.y, 0.).with_scale(Vec3::ONE * 0.5);
        });
    }
}

fn generate_item(commands: &mut Commands, asset_server: &AssetServer, current: &mut Current) {
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
        let x = timer.rng.gen_range(-400..=400);
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
        timer.timer.set_duration(Duration::from_secs_f32(5.));
        timer.timer.reset();
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

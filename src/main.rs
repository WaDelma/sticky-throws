use std::{collections::VecDeque, f32::consts::TAU, time::Duration};

use bevy::{
    ecs::system::EntityCommands, input::mouse::MouseMotion, prelude::*,
    render::camera::RenderTarget,
};
use bevy_rapier2d::prelude::*;
use physics::{handle_collisions, Hooks, PhysicsData};
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

fn setup_game(mut commands: Commands) {
    commands.insert_resource(Power(0.));

    let mut cur = Current {
        current: None,
        next: VecDeque::default(),
        rng: SmallRng::from_entropy(),
    };
    for _ in 0..3 {
        generate_item(&mut commands, &mut cur);
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
            .insert(Collider::cuboid(head_length, head_thickness))
            .insert(ColliderMassProperties::Density(3.0))
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

struct Current {
    current: Option<Entity>,
    next: VecDeque<Entity>,
    rng: SmallRng,
}

#[derive(Component)]
struct MainCamera;

const STORAGE: Vec2 = Vec2::new(-575.0, -250.0);
const SOURCE: Vec2 = Vec2::new(-350.0, -200.0);

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
        generate_item(&mut commands, &mut current);
        select_first_item(&mut commands, &mut current);
    }

    if buttons.pressed(MouseButton::Left) {
        if let Some(cur) = current.current {
            // Make moving mouse before throwing moving rotate the item
            let delta = mouse_motions.iter().fold(Vec2::ZERO, |a, b| a + b.delta);
            if delta != Vec2::ZERO {
                // TODO: Cap rotation velocity
                commands.entity(cur).insert(ExternalImpulse {
                    impulse: Vec2::ZERO,
                    torque_impulse: delta.angle_between(Vec2::X) / 100.,
                });
            };
            power.0 += 6.;
            power.0 = power.0.min(250.);

            if indicator.timer.tick(time.delta()).just_finished() {
                // TODO: This doesn't really work
                let sim_scale = 1.;
                commands
                    .spawn()
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
                    .maybe_insert(transforms.get(cur).ok().cloned())
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

fn generate_item(commands: &mut Commands, current: &mut Current) {
    let pos = STORAGE + Vec2::new(0., 75.) * current.next.len() as f32;
    let transform = Transform::from_xyz(pos.x, pos.y, 0.).with_scale(Vec3::ONE * 0.5);

    current.next.push_back(
        random_item(&mut current.rng, commands)
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

fn handle_item_dropping(mut commands: Commands, time: Res<Time>, mut timer: ResMut<ItemDropTimer>) {
    let commands = &mut commands;
    if timer.timer.tick(time.delta()).just_finished() {
        let x = timer.rng.gen_range(-400..=400);
        let y = timer.rng.gen_range(0..=100);
        let transform = Transform::from_xyz(x as f32, 500. + y as f32, 0.);
        let angle = TAU / 8.;
        random_item(&mut timer.rng, commands)
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
    commands: &'a mut Commands<'w, 's>,
) -> EntityCommands<'w, 's, 'a>
where
    R: Rng,
{
    match rng.gen_range(0..=3) {
        0 => shoe(commands, 50.),
        1 => ball(commands, 50.),
        2 => cereal_box(commands, 50.),
        3 => hammer(commands, 50.),
        _ => unreachable!(),
    }
}

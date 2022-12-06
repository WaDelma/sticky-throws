use bevy::{ecs::system::EntityCommands, prelude::*, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;
use rand::Rng;

use super::CustomMaterial;

const DAMPING: Damping = Damping {
    linear_damping: 0.2,
    angular_damping: 0.2,
};

pub fn orange<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    asset_server: &AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
    custom_materials: &mut ResMut<Assets<CustomMaterial>>,
    radius: f32,
) -> EntityCommands<'w, 's, 'a> {
    let mut cmds = commands.spawn_empty();
    cmds.with_children(|f| {
        f.spawn((
            ActiveEvents::COLLISION_EVENTS,
            ActiveHooks::FILTER_CONTACT_PAIRS,
            Collider::ball(radius),
            Restitution::coefficient(0.8),
            ColliderMassProperties::Density(1.05),
            TransformBundle::from(Transform::from_xyz(0., -radius * 0.3, 0.)),
        ));
    })
    .insert((
        RigidBody::Dynamic,
        Ccd::enabled(),
        DAMPING,
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad::new(Vec2::new(2., 3.) * radius)))
                .into(),
            material: custom_materials.add(CustomMaterial {
                color: Color::LIME_GREEN,
                color_texture: asset_server.load("orange.png"),
                sticky: 0,
            }),
            ..default()
        },
    ));
    cmds
}

pub fn cereal_box<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    asset_server: &AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
    custom_materials: &mut ResMut<Assets<CustomMaterial>>,
    radius: f32,
) -> EntityCommands<'w, 's, 'a> {
    let mut cmds = commands.spawn_empty();
    cmds.insert((
        RigidBody::Dynamic,
        ActiveEvents::COLLISION_EVENTS,
        ActiveHooks::FILTER_CONTACT_PAIRS,
        Collider::cuboid(0.75 * radius, radius),
        Restitution::coefficient(0.6),
        ColliderMassProperties::Density(0.45),
        Ccd::enabled(),
        DAMPING,
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad::new(
                    Vec2::new(0.75 * 2., 2.) * radius,
                )))
                .into(),
            material: custom_materials.add(CustomMaterial {
                color: Color::LIME_GREEN,
                color_texture: asset_server.load("cereal.png"),
                sticky: 0,
            }),
            ..default()
        },
    ));
    cmds
}

pub fn hammer<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    asset_server: &AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
    custom_materials: &mut ResMut<Assets<CustomMaterial>>,
    radius: f32,
) -> EntityCommands<'w, 's, 'a> {
    let mut cmds = commands.spawn_empty();
    let handle_thickness = radius * 0.2;
    let head_thickness = radius * 0.25;
    let head_length = radius * 0.75;
    cmds.with_children(|children| {
        children.spawn((
            Restitution::coefficient(0.2),
            ActiveEvents::COLLISION_EVENTS,
            ActiveHooks::FILTER_CONTACT_PAIRS,
            Collider::cuboid(head_length, head_thickness),
            ColliderMassProperties::Density(3.5),
            TransformBundle::from(Transform::from_xyz(0., radius - radius * 0.3, 0.)),
        ));
        children.spawn((
            Restitution::coefficient(0.5),
            ActiveEvents::COLLISION_EVENTS,
            ActiveHooks::FILTER_CONTACT_PAIRS,
            Collider::cuboid(handle_thickness, radius - 0.5 * head_thickness),
            ColliderMassProperties::Density(0.8),
            TransformBundle::from(Transform::from_xyz(
                0.0,
                -0.5 * head_thickness - radius * 0.3,
                0.,
            )),
        ));
    })
    .insert((
        RigidBody::Dynamic,
        Ccd::enabled(),
        DAMPING,
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad::new(Vec2::new(2., 3.) * radius)))
                .into(),
            material: custom_materials.add(CustomMaterial {
                color: Color::LIME_GREEN,
                color_texture: asset_server.load("hammer.png"),
                sticky: 0,
            }),
            ..default()
        },
    ));
    cmds
}

pub fn shoe<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    asset_server: &AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
    custom_materials: &mut ResMut<Assets<CustomMaterial>>,
    radius: f32,
) -> EntityCommands<'w, 's, 'a> {
    let mid = Vec2::new(radius, radius);
    let mut cmds = commands.spawn_empty();

    cmds.insert((
        RigidBody::Dynamic,
        ActiveEvents::COLLISION_EVENTS,
        ActiveHooks::FILTER_CONTACT_PAIRS,
        Collider::convex_decomposition(
            &[
                Vec2::new(0., 0.) - mid,
                Vec2::new(0., 2. * radius) - mid,
                Vec2::new(radius, 2. * radius) - mid,
                Vec2::new(radius, 0.75 * radius) - mid,
                Vec2::new(2. * radius, 0.75 * radius) - mid,
                Vec2::new(2. * radius, 0.) - mid,
            ],
            &[[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 0]],
        ),
        Restitution::coefficient(1.),
        ColliderMassProperties::Density(1.15),
        Ccd::enabled(),
        DAMPING,
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad::new(Vec2::new(2., 2.) * radius)))
                .into(),
            material: custom_materials.add(CustomMaterial {
                color: Color::LIME_GREEN,
                color_texture: asset_server.load("boot.png"),
                sticky: 0,
            }),
            ..default()
        },
    ));
    cmds
}

pub fn random_item<'w, 's, 'a, R>(
    rng: &mut R,
    commands: &'a mut Commands<'w, 's>,
    asset_server: &'a AssetServer,
    meshes: &mut ResMut<Assets<Mesh>>,
    custom_materials: &mut ResMut<Assets<CustomMaterial>>,
) -> EntityCommands<'w, 's, 'a>
where
    R: Rng,
{
    match rng.gen_range(0..=3) {
        0 => shoe(commands, asset_server, meshes, custom_materials, 50.),
        1 => orange(commands, asset_server, meshes, custom_materials, 50.),
        2 => cereal_box(commands, asset_server, meshes, custom_materials, 75.),
        3 => hammer(commands, asset_server, meshes, custom_materials, 50.),
        _ => unreachable!(),
    }
}

use std::convert::identity;
use std::sync::Mutex;

use bevy::ecs::query::WorldQuery;
use bevy::math::Vec3Swizzles;
use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::prelude::CollisionEventFlags;
use union_find::Union;
use union_find::UnionFind;
use union_find::{QuickFindUf, UnionBySizeRank};

use crate::game::{DeathTimer, Destroyer, ScoringEffect, Wall};

use super::throw::{Ghost, IgnoreCollisions, Player, Throwable};
use super::Disabler;

// TODO: When entities are deleted clear this data structure
#[derive(Component)]
pub struct StuckItems {
    // TODO: Had to wrap this to mutex because `find` borrows mutably (because QuickUnionUf modifies the data structure)
    // and `PhysicsHooksWithQuery` allows only immutable queries :/
    pub union_find: Mutex<QuickFindUf<EntityWrapper>>,
    pub map: HashMap<Entity, usize>,
}

// TODO: `UnionBySizeRank` chosen for no particular reason. `UnionBySizeRank` is broken https://github.com/gifnksm/union-find-rs/issues/12
pub struct EntityWrapper(Entity, UnionBySizeRank);

impl Union for EntityWrapper {
    fn union(lval: Self, rval: Self) -> union_find::UnionResult<Self> {
        use union_find::UnionResult::*;
        match UnionBySizeRank::union(lval.1, rval.1) {
            Left(l) => Left(EntityWrapper(lval.0, l)),
            Right(r) => Right(EntityWrapper(rval.0, r)),
        }
    }
}

pub struct Hooks;

#[derive(WorldQuery)]
#[world_query(derive(Default))]
pub struct PhysicsData<'a> {
    joint: Option<&'a ImpulseJoint>,
    ghost: Option<&'a Ghost>,
    parent: Option<&'a Parent>,
    items: Option<&'a StuckItems>,
    ignore_collisions: Option<&'a IgnoreCollisions>,
}

fn hook_get_rec_ghost<'a, 'b>(
    query: &'b Query<PhysicsData<'a>>,
    e: Entity,
) -> Option<(&'b Ghost, Entity)> {
    let physics_data = query
        .get(e)
        .ok()
        .map_or(PhysicsDataItem::default(), identity);
    physics_data.ghost.map(|g| (g, e)).or_else(|| {
        physics_data
            .parent
            .and_then(|parent| hook_get_rec_ghost(query, parent.get()))
    })
}

fn hook_find_parent<'a, 'b, F>(
    query: &'b Query<PhysicsData<'a>>,
    e: Entity,
    mut f: F,
) -> Option<Entity>
where
    F: FnMut(Entity) -> bool,
{
    if f(e) {
        return Some(e);
    }
    let physics_data = query
        .get(e)
        .ok()
        .map_or(PhysicsDataItem::default(), identity);

    physics_data
        .parent
        .and_then(|p| hook_find_parent(query, p.get(), f))
}

impl<'a> PhysicsHooksWithQuery<PhysicsData<'a>> for Hooks {
    fn filter_contact_pair(
        &self,
        context: PairFilterContextView,
        query: &Query<PhysicsData<'a>>,
    ) -> Option<SolverFlags> {
        let (a, b) = (context.collider1(), context.collider2());
        let ghost_a = hook_get_rec_ghost(query, a);
        if let Some((&Ghost(g), _)) = ghost_a {
            if hook_find_parent(query, b, |e| g == e).is_some() {
                return None;
            }
        }
        let ghost_b = hook_get_rec_ghost(query, b);
        if let Some((&Ghost(g), _)) = ghost_b {
            if hook_find_parent(query, a, |e| g == e).is_some() {
                return None;
            }
        }
        if ghost_a.is_some() && ghost_b.is_some() {
            return None;
        }

        let get_parent = |e| query.get(e).ok().and_then(|j| j.parent);
        let get = |e| query.get(e).ok().and_then(|j| j.ignore_collisions);

        if get_recursively(get_parent, get, a).is_some() {
            return None;
        }
        if get_recursively(get_parent, get, b).is_some() {
            return None;
        }

        let stuck_items = query.iter().flat_map(|j| j.items).next().unwrap();

        let p1 = find_most_parent(get_parent, a);
        let p2 = find_most_parent(get_parent, b);

        if let (Some(ia), Some(ib)) = (stuck_items.map.get(&p1), stuck_items.map.get(&p2)) {
            let pa = stuck_items.union_find.lock().unwrap().find(*ia);
            let pb = stuck_items.union_find.lock().unwrap().find(*ib);
            if pa == pb {
                return None;
            }
        }

        return Some(SolverFlags::COMPUTE_IMPULSES);
    }
}

fn get_recursively<'a, FP, FT, T>(
    mut get_parent: FP,
    mut get: FT,
    e: Entity,
) -> Option<(&'a T, Entity)>
where
    FP: FnMut(Entity) -> Option<&'a Parent>,
    FT: FnMut(Entity) -> Option<&'a T>,
    T: Component,
{
    get(e)
        .map(|t| (t, e))
        .or_else(|| get_parent(e).and_then(|parent| get_recursively(get_parent, get, parent.get())))
}

fn find_most_parent<'a, F>(mut get_parent: F, e: Entity) -> Entity
where
    F: FnMut(Entity) -> Option<&'a Parent>,
{
    get_parent(e)
        .map(|p| find_most_parent(get_parent, p.get()))
        .unwrap_or(e)
}

pub fn handle_collisions(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut collision_events: EventReader<CollisionEvent>,
    parents: Query<&Parent>,
    mut players: Query<&mut Player>,
    ghosts: Query<&Ghost>,
    mut throwables: Query<&mut Throwable>,
    transforms: Query<&Transform>,
    impulse_joints: Query<&ImpulseJoint>,
    destroyers: Query<&Destroyer>,
    disablers: Query<&Disabler>,
    walls: Query<&Wall>,
    mut stuck_items: Query<&mut StuckItems>,
    asset_server: Res<AssetServer>,
) {
    let stuck_items = &mut *stuck_items.single_mut();

    for collision_event in collision_events.iter() {
        let get_parent = |e| parents.get(e).ok();
        let get_throwable = |e| throwables.get(e).ok();
        let get_ghost = |e| ghosts.get(e).ok();
        match collision_event {
            CollisionEvent::Started(a, b, flags) if flags.contains(CollisionEventFlags::SENSOR) => {
                if disablers.get(*a).is_ok() {
                    if let Some((_, e)) = get_recursively(get_parent, get_throwable, *b) {
                        players.single_mut().disables.insert(e);
                    }
                }
                if disablers.get(*b).is_ok() {
                    if let Some((_, e)) = get_recursively(get_parent, get_throwable, *a) {
                        players.single_mut().disables.insert(e);
                    }
                }
            }
            // TODO: Sometimes delete is not registered
            CollisionEvent::Stopped(a, b, flags) if flags.contains(CollisionEventFlags::SENSOR) => {
                if disablers.get(*a).is_ok() {
                    if let Some((_, e)) = get_recursively(get_parent, get_throwable, *b) {
                        players.single_mut().disables.remove(&e);
                    }
                }
                if disablers.get(*b).is_ok() {
                    if let Some((_, e)) = get_recursively(get_parent, get_throwable, *a) {
                        players.single_mut().disables.remove(&e);
                    }
                }
            }
            CollisionEvent::Started(a, b, _) => {
                let (a, b) = (*a, *b);
                // `filter_contact_pair` should ensure that both are not ghosts or what ghost was made of
                if let Some((_, e)) = get_recursively(get_parent, get_ghost, a) {
                    commands.entity(e).despawn_recursive();
                } else if let Some((_, e)) = get_recursively(get_parent, get_ghost, b) {
                    commands.entity(e).despawn_recursive();
                }
                let ta = get_recursively(get_parent, get_throwable, a);
                let tb = get_recursively(get_parent, get_throwable, b);
                if let (Some((t1, e1)), Some((t2, e2))) = (ta, tb) {
                    if !t1.sticky && !t2.sticky {
                        continue;
                    }
                    if connected_by_impulse_joint(
                        (e1, impulse_joints.get(e1).ok()),
                        (e2, impulse_joints.get(e2).ok()),
                    ) {
                        // Objects are already glued together
                        continue;
                    }

                    let get_rotation = |e| {
                        transforms
                            .get(e)
                            .map(|t| t.rotation.to_euler(EulerRot::XYZ).2)
                            .unwrap_or_default()
                    };

                    // Determine where they collided from the contact graph
                    if let Some(contact_pair) = rapier_context.contact_pair(a, b) {
                        if let Some((_, contact_point)) = contact_pair.find_deepest_contact() {
                            let (e1, e2, a, b) = if a == contact_pair.collider1() {
                                (e1, e2, a, b)
                            } else {
                                (e2, e1, b, a)
                            };

                            let rot1 = get_rotation(e1);
                            let rot2 = get_rotation(e2);

                            let local_transform = |mut cur, parent| {
                                let mut transform = Vec2::ZERO;
                                while let Ok(p) = parents.get(cur) {
                                    if let Ok(t) = transforms.get(cur) {
                                        transform += t.translation.xy();
                                    }
                                    cur = p.get();
                                    if cur == parent {
                                        break;
                                    }
                                }
                                return transform;
                            };

                            // TODO: Get rid of hardcoded 100
                            let t1 = local_transform(a, e1) + contact_point.local_p1() * 100.;
                            let t2 = local_transform(b, e2) + contact_point.local_p2() * 100.;

                            let (la1, lb1, la2, lb2) = (t1, -rot1, t2, -rot2);
                            // info!("{}, {}, {}, {}", la1, lb1, la2, lb2);

                            // And then glue them together by creating fixed impulse joint between them
                            let joint = FixedJointBuilder::new()
                                .local_anchor1(la1)
                                .local_basis1(lb1)
                                .local_anchor2(la2)
                                .local_basis2(lb2);

                            commands.entity(e2).add_children(|builder| {
                                builder.spawn().insert(ImpulseJoint::new(e1, joint));
                            });

                            let i1 = *stuck_items.map.entry(e1).or_insert_with(|| {
                                stuck_items
                                    .union_find
                                    .lock()
                                    .unwrap()
                                    .insert(EntityWrapper(e1, default()))
                            });
                            let i2 = *stuck_items.map.entry(e2).or_insert_with(|| {
                                stuck_items
                                    .union_find
                                    .lock()
                                    .unwrap()
                                    .insert(EntityWrapper(e2, default()))
                            });
                            stuck_items.union_find.lock().unwrap().union(i1, i2);

                            // Add score
                            if let Ok([mut tr1, mut tr2]) = throwables.get_many_mut([e1, e2]) {
                                let mut handle = |throwable: &Throwable, e| {
                                    if throwable.stuck {
                                        return;
                                    }
                                    if let Some(mut player) =
                                        throwable.player.and_then(|p| players.get_mut(p).ok())
                                    {
                                        if let Some(&key) = stuck_items.map.get(&e) {
                                            let size = stuck_items
                                                .union_find
                                                .lock()
                                                .unwrap()
                                                .get(key)
                                                .1
                                                .size();
                                            let points = 10 * fibonacci(size);
                                            let pos =
                                                transforms.get(e1).unwrap().translation.xy() + t1;
                                            let total_points = throwable.multiplier * points;
                                            visualise_scoring(
                                                &asset_server,
                                                pos,
                                                &mut commands,
                                                points,
                                                throwable.multiplier,
                                                total_points,
                                            );
                                            player.score += total_points;
                                        }
                                    }
                                };
                                handle(&tr1, e1);
                                handle(&tr2, e2);
                                tr1.sticky = true;
                                tr2.sticky = true;
                                tr1.stuck = true;
                                tr2.stuck = true;
                            }
                        }
                    }
                } else {
                    if let Some((throwable, e)) = ta {
                        if destroyers.get(b).is_ok() {
                            commands.entity(e).despawn_recursive();
                            if !throwable.stuck {
                                if let Some(mut player) =
                                    throwable.player.and_then(|e| players.get_mut(e).ok())
                                {
                                    player.lives -= 1;
                                }
                            }
                        }
                        if walls.get(b).is_ok() {
                            commands.add(move |world: &mut World| {
                                world.get_mut::<Throwable>(e).unwrap().multiplier += 1;
                            });
                        }
                    }
                    if let Some((throwable, e)) = tb {
                        if destroyers.get(a).is_ok() {
                            commands.entity(e).despawn_recursive();
                            if !throwable.stuck {
                                if let Some(mut player) =
                                    throwable.player.and_then(|e| players.get_mut(e).ok())
                                {
                                    player.lives -= 1;
                                }
                            }
                        }
                        if walls.get(a).is_ok() {
                            commands.add(move |world: &mut World| {
                                world.get_mut::<Throwable>(e).unwrap().multiplier += 1;
                            });
                        }
                    }
                    continue;
                }
            }
            _ => {}
        }
    }
}

fn visualise_scoring(
    asset_server: &AssetServer,
    pos: Vec2,
    commands: &mut Commands,
    points: usize,
    multiplier: usize,
    total_points: usize,
) {
    let font = asset_server.load("fonts/MajorMonoDisplay-Regular.ttf");
    let text_style = TextStyle {
        font,
        font_size: 30.0,
        color: Color::ORANGE,
    };
    let text_alignment = TextAlignment::CENTER;
    let mult = if multiplier > 1 {
        format!("{multiplier}×")
    } else {
        "".to_owned()
    };
    let fifties = total_points / 50;
    let exclamation_marks = "!".repeat(fifties / 2);
    let question_mark = "?".repeat(fifties % 2);
    let emphasis = format!("{exclamation_marks}{question_mark}");
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_section(format!("{mult}{points}{emphasis}"), text_style.clone())
                .with_alignment(text_alignment),
            transform: Transform::from_xyz(pos.x, pos.y, 10.),
            ..default()
        })
        .insert(ScoringEffect { multiplier, points })
        .insert(DeathTimer(Timer::from_seconds(1., false)));
}

fn fibonacci(n: usize) -> usize {
    if n <= 1 {
        1
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

/// Checks if either entity has a impulse joint which parent the other entity is
fn connected_by_impulse_joint(
    (a, joint_a): (Entity, Option<&ImpulseJoint>),
    (b, joint_b): (Entity, Option<&ImpulseJoint>),
) -> bool {
    return joint_a.map(|joint| joint.parent == b).unwrap_or(false)
        || joint_b.map(|joint| joint.parent == a).unwrap_or(false);
}

use bevy::{ecs::system::EntityCommands, prelude::*};

pub trait EntityCommandsExt {
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

pub fn screen_to_world(
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

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

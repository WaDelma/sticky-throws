use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

use super::throw::Throwable;

#[derive(Component)]
struct StickyEffect;
pub fn handle_stickiness_effect(
    throwables: Query<(&Throwable, &Handle<StickyMaterial>)>,
    mut custom_materials: ResMut<Assets<StickyMaterial>>,
) {
    for (throwable, material) in throwables.iter() {
        if let Some(material) = custom_materials.get_mut(material) {
            if throwable.sticky {
                material.sticky = 1;
            } else {
                material.sticky = 0;
            }
        }
    }
}

impl Material2d for StickyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sticky.wgsl".into()
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct StickyMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(1)]
    pub sticky: i32,
    #[texture(2)]
    #[sampler(3)]
    pub color_texture: Handle<Image>,
}

impl Material2d for TilingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tiling.wgsl".into()
    }
}

impl TilingMaterial {
    pub fn new(texture_handle: Handle<Image>, dims: [f32; 4]) -> Self {
        Self {
            dims: Vec4::new(dims[0], dims[1], dims[2], dims[3]),
            color_texture: texture_handle,
        }
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "785246fd-1dfe-40e9-8c8b-1886d8d4d47e"]
pub struct TilingMaterial {
    #[uniform(0)]
    pub dims: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Handle<Image>,
}

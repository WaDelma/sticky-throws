use bevy::audio::AudioSink;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::render::texture::ImageSampler;
use bevy::{prelude::*, sprite::Material2dPlugin, window::PresentMode};
use bevy_rapier2d::prelude::*;
use game::physics::PhysicsData;
use game::shaders::{StickyMaterial, TilingMaterial};
use wgpu::{AddressMode, SamplerBorderColor, SamplerDescriptor};

mod game;
mod menu;
mod splash;
mod utils;

// TODO: Add one way dome around throwing position
// TODO: Polish gameplay
// TODO: Implement sounds

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Splash,
    Menu,
    Game,
}

fn main() {
    App::new()
        .add_state(GameState::Splash)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "Sticky throws".to_owned(),
                        width: 1920.,
                        height: 1080.,
                        resizable: false,
                        // TODO: Figure out how to scale game if resolution changes
                        // mode: WindowMode::BorderlessFullscreen,
                        // TODO: Does not work in webassembly
                        // present_mode: PresentMode::Immediate,
                        ..default()
                    },
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: SamplerDescriptor {
                        address_mode_u: AddressMode::ClampToBorder,
                        address_mode_v: AddressMode::ClampToBorder,
                        border_color: Some(SamplerBorderColor::TransparentBlack),
                        ..default()
                    },
                }),
        )
        .add_plugin(splash::SplashPlugin)
        .add_plugin(game::GamePlugin)
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(menu::MenuPlugin)
        .add_plugin(RapierPhysicsPlugin::<PhysicsData>::pixels_per_meter(100.0))
        .add_plugin(Material2dPlugin::<StickyMaterial>::default())
        .add_plugin(Material2dPlugin::<TilingMaterial>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .run();
}

#[derive(Default, Resource)]
pub struct Music(Option<Handle<AudioSink>>);

#[derive(Component)]
pub struct MainCamera;
fn setup(mut commands: Commands) {
    commands.init_resource::<Music>();
    commands.spawn(Camera2dBundle::default()).insert(MainCamera);
}

use bevy::audio::AudioSink;
use bevy::{prelude::*, sprite::Material2dPlugin, window::PresentMode};
use bevy_rapier2d::prelude::*;
use game::{physics::PhysicsData, CustomMaterial};

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
        .insert_resource(WindowDescriptor {
            title: "Sticky throws".to_owned(),
            width: 1920.,
            height: 1080.,
            resizable: false,
            // TODO: Figure out how to scale game if resolution changes
            // mode: WindowMode::BorderlessFullscreen,
            // TODO: Does not work in webassembly
            // present_mode: PresentMode::Immediate,
            ..default()
        })
        .add_state(GameState::Splash)
        .add_plugins(DefaultPlugins)
        .add_plugin(splash::SplashPlugin)
        .add_plugin(game::GamePlugin)
        // .add_plugin(menu::MenuPlugin)
        .add_plugin(RapierPhysicsPlugin::<PhysicsData>::pixels_per_meter(100.0))
        .add_plugin(Material2dPlugin::<CustomMaterial>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .run();
}

#[derive(Default)]
pub struct Music(Option<Handle<AudioSink>>);

#[derive(Component)]
pub struct MainCamera;
fn setup(mut commands: Commands) {
    commands.init_resource::<Music>();
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(MainCamera);
}

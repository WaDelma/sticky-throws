use bevy::prelude::*;

use crate::utils::despawn_screen;

use super::GameState;

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Splash).with_system(splash_setup))
            .add_system_set(SystemSet::on_update(GameState::Splash).with_system(countdown))
            .add_system_set(
                SystemSet::on_exit(GameState::Splash).with_system(despawn_screen::<OnSplashScreen>),
            );
    }
}

#[derive(Component)]
struct OnSplashScreen;

#[derive(Deref, DerefMut, Resource)]
struct SplashTimer(Timer);

fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let style = TextStyle {
        font: asset_server.load("fonts/MajorMonoDisplay-Regular.ttf"),
        font_size: 100.0,
        color: Color::WHITE,
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::DARK_GRAY.into(),
            ..default()
        })
        .insert(OnSplashScreen)
        .with_children(|parent| {
            // Display the game name
            parent.spawn(
                TextBundle::from_sections([
                    TextSection {
                        value: "🍊".to_owned(),
                        style: TextStyle {
                            font: asset_server.load("fonts/NotoEmoji-VariableFont_wght.ttf"),
                            font_size: 75.0,
                            color: Color::ORANGE,
                        },
                    },
                    TextSection {
                        value: "StIcKy".to_owned(),
                        style: style.clone(),
                    },
                    TextSection {
                        value: "👢\n".to_owned(),
                        style: TextStyle {
                            font: asset_server.load("fonts/NotoEmoji-VariableFont_wght.ttf"),
                            font_size: 75.0,
                            color: Color::PURPLE,
                        },
                    },
                    TextSection {
                        value: "🐒".to_owned(),
                        style: TextStyle {
                            font: asset_server.load("fonts/NotoEmoji-VariableFont_wght.ttf"),
                            font_size: 75.0,
                            color: Color::BEIGE,
                        },
                    },
                    TextSection {
                        value: "ThrOWs".to_owned(),
                        style: style.clone(),
                    },
                    TextSection {
                        value: "🐒\n".to_owned(),
                        style: TextStyle {
                            font: asset_server.load("fonts/NotoEmoji-VariableFont_wght.ttf"),
                            font_size: 75.0,
                            color: Color::BEIGE,
                        },
                    },
                ])
                .with_style(Style {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                }),
            );
        });
    commands.insert_resource(SplashTimer(Timer::from_seconds(2.0, TimerMode::Once)));
}

fn countdown(
    mut game_state: ResMut<State<GameState>>,
    time: Res<Time>,
    buttons: Res<Input<MouseButton>>,
    mut timer: ResMut<SplashTimer>,
) {
    if buttons.just_pressed(MouseButton::Left) || timer.tick(time.delta()).finished() {
        game_state.set(GameState::Game).unwrap();
    }
}

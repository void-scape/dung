use bevy::prelude::*;
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_frp::ReactPlugin;
use bevy_seedling::SeedlingPlugin;

mod arena;
mod enemy;
mod equipment;
mod input;
mod mapgen;
mod observer;
mod player;
mod tile;

pub const WIDTH: usize = 1024;
pub const HEIGHT: usize = 1024;
pub const TILE_SIZE: usize = 16;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "debug")]
    app.add_systems(Update, close_on_escape);

    // #[cfg(not(feature = "debug"))]
    // app.set_error_handler(bevy::ecs::error::warn);

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: [WIDTH as u32, HEIGHT as u32].into(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        bevy_rand::prelude::EntropyPlugin::<bevy_rand::prelude::WyRand>::with_seed(
            69u64.to_le_bytes(),
        ),
        SeedlingPlugin::default(),
        ReactPlugin,
        EnhancedInputPlugin,
    ))
    .add_plugins((
        tile::plugin,
        mapgen::plugin,
        input::plugin,
        enemy::plugin,
        arena::plugin,
        player::plugin,
        equipment::plugin,
    ))
    .init_state::<GameState>()
    .add_systems(Startup, camera);

    app.add_systems(Update, enter_exit_arena);

    app.run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Overworld,
    Arena,
}

fn enter_exit_arena(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut in_arena: Local<bool>,
) {
    if input.just_pressed(KeyCode::KeyP) {
        if *in_arena {
            commands.set_state(GameState::Overworld);
        } else {
            commands.set_state(GameState::Arena);
        }
        *in_arena = !*in_arena;
    }
}

#[cfg(not(feature = "debug"))]
pub fn name(_: impl Into<std::borrow::Cow<'static, str>>) -> () {}
#[cfg(feature = "debug")]
pub fn name(name: impl Into<std::borrow::Cow<'static, str>>) -> Name {
    Name::new(name)
}

#[cfg(feature = "debug")]
fn close_on_escape(input: Res<ButtonInput<KeyCode>>, mut writer: MessageWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        writer.write(AppExit::Success);
    }
}

fn camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

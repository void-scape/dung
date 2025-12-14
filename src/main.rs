use bevy::prelude::*;
use bevy_frp::ReactPlugin;
use bevy_seedling::SeedlingPlugin;

mod input;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "debug")]
    app.add_systems(Update, close_on_escape);

    // #[cfg(not(feature = "debug"))]
    // app.set_error_handler(bevy::ecs::error::warn);

    app.add_plugins((
        DefaultPlugins,
        bevy_rand::prelude::EntropyPlugin::<bevy_rand::prelude::WyRand>::with_seed(
            69u64.to_le_bytes(),
        ),
        SeedlingPlugin::default(),
        ReactPlugin,
        input::InputPlugin,
    ))
    .add_systems(Startup, (camera, spawn_tiles));

    app.run();
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

fn spawn_tiles(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = server.load("anno16.png");
    let atlas = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(16),
        16,
        16,
        None,
        None,
    ));

    commands.spawn((
        Sprite::from_atlas_image(
            image,
            TextureAtlas {
                layout: atlas,
                index: 1,
            },
        ),
        Transform::default(),
    ));
}

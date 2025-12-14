use crate::TILE_SIZE;
use bevy::{prelude::*, sprite::Anchor};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_atlas)
        .add_systems(Update, tile_sprite);
}

/// Marks a tile as obstructive to entity movement.
#[derive(Default, Component)]
pub struct Solid;

#[derive(Component)]
#[require(TileSprite::WALL, Solid)]
pub struct Wall;

#[derive(Component)]
#[require(TileSprite::FLOOR)]
pub struct Floor;

#[derive(Component)]
pub struct TileSprite {
    pub ascii: u8,
    pub fg: Color,
    pub bg: Color,
}

impl TileSprite {
    pub const WALL: Self = Self {
        ascii: b'x',
        fg: Color::WHITE,
        bg: Color::BLACK,
    };

    pub const FLOOR: Self = Self {
        ascii: b'.',
        fg: Color::WHITE,
        bg: Color::BLACK,
    };
}

fn tile_sprite(
    mut commands: Commands,
    atlas: Res<TileAtlas>,
    tiles: Query<(Entity, &TileSprite), Added<TileSprite>>,
) {
    for (entity, tile) in tiles.iter() {
        commands.entity(entity).insert((
            Sprite {
                image: atlas.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas.atlas.clone(),
                    index: tile.ascii as usize,
                }),
                color: tile.fg,
                ..Default::default()
            },
            Anchor::BOTTOM_LEFT,
        ));
        commands.spawn((
            Transform::from_xyz(0.0, 0.0, -1.0),
            Sprite::from_color(tile.bg, Vec2::splat(TILE_SIZE as f32)),
            Anchor::BOTTOM_LEFT,
            ChildOf(entity),
        ));
    }
}

#[derive(Resource)]
pub struct TileAtlas {
    pub image: Handle<Image>,
    pub atlas: Handle<TextureAtlasLayout>,
}

fn spawn_atlas(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.insert_resource(TileAtlas {
        image: server.load("anno16.png"),
        atlas: atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(TILE_SIZE as u32),
            16,
            16,
            None,
            None,
        )),
    });
}

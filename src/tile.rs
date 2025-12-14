#![allow(unused)]

use crate::TILE_SIZE;
use bevy::{
    color::palettes::tailwind::GREEN_300,
    ecs::{
        query::{QueryData, QueryEntityError, QueryFilter, ROQueryItem},
        system::SystemParam,
    },
    platform::collections::{HashMap, hash_map::Entry},
    prelude::*,
    sprite::Anchor,
};
use bevy_query_observer::{AddStartObserver, AddStopObserver, Start, Stop};

pub fn plugin(app: &mut App) {
    app.init_resource::<TileIndex>()
        .add_systems(PreStartup, spawn_atlas)
        .add_start_observer(tile_sprite)
        .add_start_observer(TilePosition::update_transform)
        .add_start_observer(TilePosition::observe_insert)
        .add_stop_observer(TilePosition::observe_replace)
        .add_start_observer(move_object);
}

#[derive(Component, Deref, DerefMut, Clone, Copy, PartialEq, Eq, Hash)]
#[component(immutable)]
pub struct MoveIntent(pub IVec2);

fn move_object(
    data: Start<(Entity, &TilePosition, &MoveIntent)>,
    solid: PositionQuery<(), With<Solid>>,
    mut commands: Commands,
) {
    let (entity, position, intent) = data.into_inner();

    let new_position = TilePosition(position.0 + intent.0);

    let mut entity = commands.entity(entity);
    entity.remove::<MoveIntent>();

    if solid.iter(&new_position).count() == 0 {
        entity.insert(new_position);
    }
}

#[derive(Resource, Deref, Default)]
pub struct TileIndex(HashMap<TilePosition, Vec<Entity>>);

#[derive(Component, Default, Deref, DerefMut, Clone, Copy, PartialEq, Eq, Hash)]
#[require(Transform, TileZ)]
#[component(immutable)]
pub struct TilePosition(pub IVec2);

#[derive(Component, Default, Deref, DerefMut, Clone, Copy, PartialEq, Eq)]
pub struct TileZ(pub i32);

impl TilePosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self(IVec2::new(x, y))
    }

    fn update_transform(data: Start<(Entity, &TilePosition, &TileZ)>, mut commands: Commands) {
        let (entity, &position, &z) = data.into_inner();

        commands
            .entity(entity)
            .entry::<Transform>()
            .or_default()
            .and_modify(move |mut transform| {
                transform.translation.x = position.x as f32 * TILE_SIZE as f32;
                transform.translation.y = position.y as f32 * TILE_SIZE as f32;
                transform.translation.z = z.0 as f32;
            });
    }

    fn observe_insert(data: Start<(Entity, &TilePosition)>, mut index: ResMut<TileIndex>) {
        let (entity, position) = data.into_inner();
        index.0.entry(*position).or_default().push(entity);
    }

    fn observe_replace(data: Stop<(Entity, &TilePosition)>, mut index: ResMut<TileIndex>) {
        let (entity, position) = data.into_inner();

        if let Entry::Occupied(mut e) = index.0.entry(*position) {
            e.get_mut().retain(|e| *e != entity);
        }
    }
}

#[derive(SystemParam)]
pub struct PositionQuery<'w, 's, D, F = ()>
where
    D: QueryData + 'static,
    F: QueryFilter + 'static,
{
    query: Query<'w, 's, D, F>,
    index: Res<'w, TileIndex>,
}

#[derive(Debug, Clone)]
pub enum PositionQueryError {
    NoMatchingEntity,
    MultipleMatches,
    Query(QueryEntityError),
}

impl core::fmt::Display for PositionQueryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoMatchingEntity => {
                write!(f, "no matching entity for tile query")
            }
            Self::MultipleMatches => {
                write!(f, "more than one entity matches tile query")
            }
            Self::Query(q) => q.fmt(f),
        }
    }
}

impl core::error::Error for PositionQueryError {}

impl<'w, 's, D, F> PositionQuery<'w, 's, D, F>
where
    D: QueryData + 'static,
    F: QueryFilter + 'static,
{
    pub fn get(
        &self,
        position: &TilePosition,
    ) -> Result<ROQueryItem<'_, 's, D>, PositionQueryError> {
        let entities = self
            .index
            .get(position)
            .ok_or(PositionQueryError::NoMatchingEntity)?;

        if entities.iter().filter(|e| self.query.contains(**e)).count() > 1 {
            return Err(PositionQueryError::MultipleMatches);
        }

        entities
            .iter()
            .find_map(|e| self.query.get(*e).ok())
            .ok_or(PositionQueryError::NoMatchingEntity)
    }

    pub fn get_mut(
        &mut self,
        position: &TilePosition,
    ) -> Result<D::Item<'_, 's>, PositionQueryError> {
        let entities = self
            .index
            .get(position)
            .ok_or(PositionQueryError::NoMatchingEntity)?;

        match entities.iter().filter(|e| self.query.contains(**e)).count() {
            0 => {
                return Err(PositionQueryError::MultipleMatches);
            }
            1 => {}
            _ => {
                return Err(PositionQueryError::MultipleMatches);
            }
        }
        let first_match = entities.iter().find(|e| self.query.contains(**e)).unwrap();

        self.query
            .get_mut(*first_match)
            .map_err(PositionQueryError::Query)
    }

    pub fn iter(&self, position: &TilePosition) -> impl Iterator<Item = ROQueryItem<'_, 's, D>> {
        self.index
            .get(position)
            .into_iter()
            .flat_map(|e| e.iter())
            .flat_map(|e| self.query.get(*e))
    }
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
        fg: Color::Srgba(GREEN_300),
        bg: Color::BLACK,
    };

    pub const FLOOR: Self = Self {
        ascii: b'.',
        fg: Color::Srgba(GREEN_300),
        bg: Color::BLACK,
    };
}

#[derive(Component)]
pub struct TileBg;

fn tile_sprite(tiles: Start<(Entity, &TileSprite)>, atlas: Res<TileAtlas>, mut commands: Commands) {
    let (entity, tile) = tiles.into_inner();

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
        TileBg,
        Transform::from_xyz(0.0, 0.0, -0.01),
        Sprite::from_color(tile.bg, Vec2::splat(TILE_SIZE as f32)),
        Anchor::BOTTOM_LEFT,
        ChildOf(entity),
    ));
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

#[derive(Clone, Copy)]
pub enum TextAnchor {
    TopLeft,
    Center,
}

pub fn text_tiles(
    str: &str,
    x: i32,
    y: i32,
    anchor: TextAnchor,
) -> impl Iterator<Item = (TileSprite, Vec2)> {
    let tx = x as f32;
    let ty = y as f32;

    let lines = str.lines().count();
    str.lines().enumerate().flat_map(move |(y, line)| {
        line.as_bytes().iter().enumerate().map(move |(x, byte)| {
            let position = match anchor {
                TextAnchor::Center => Vec2::new(
                    (x as f32 + tx - (line.len() / 2) as f32) * TILE_SIZE as f32,
                    (-(y as f32) + ty + (lines / 2) as f32) * TILE_SIZE as f32,
                ),
                TextAnchor::TopLeft => Vec2::new(
                    (x as f32 + tx) * TILE_SIZE as f32,
                    (-(y as f32) + ty) * TILE_SIZE as f32,
                ),
            };

            (
                TileSprite {
                    ascii: *byte,
                    fg: Color::WHITE,
                    bg: Color::BLACK,
                },
                position,
            )
        })
    })
}

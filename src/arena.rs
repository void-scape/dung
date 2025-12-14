use crate::{
    GameState,
    tile::{TextAnchor, text_tiles},
};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Arena), enter_arena)
        .add_systems(OnExit(GameState::Arena), exit_arena);
}

#[derive(Component)]
pub struct ArenaEntity;

const ARENA: &str = r#"
+-------------------------------+
|                               |
|                               |
|                               |
|                       e       |
|                               |
|                               |
|      p            e           |
|                               |
|                               |
|                         e     |
|                               |
|                               |
|                               |
+-------------------------------+
"#;

fn enter_arena(mut commands: Commands) {
    for (tile, position) in text_tiles(ARENA, 0, 0, TextAnchor::Center) {
        commands.spawn((
            tile,
            Transform::from_translation(position.extend(10.0)),
            ArenaEntity,
        ));
    }
}

fn exit_arena(mut commands: Commands, entities: Query<Entity, With<ArenaEntity>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}

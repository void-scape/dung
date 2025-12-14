use crate::{
    input::Move,
    tile::{MoveIntent, Solid, TilePosition, TileSprite, TileZ},
};
use bevy::{color::palettes::css::WHITE, prelude::*};
use bevy_enhanced_input::prelude::Fire;

#[derive(Component)]
#[require(TilePosition, TileSprite::PLAYER, TileZ(1), Solid)]
pub struct Player;

impl TileSprite {
    pub const PLAYER: Self = Self {
        ascii: b'p',
        fg: Color::Srgba(WHITE),
        bg: Color::BLACK,
    };
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((Player, TilePosition::new(0, 0)));
    })
    .add_observer(move_player);
}

fn move_player(
    trigger: On<Fire<Move>>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
) {
    let direction = trigger.value;

    let x = direction.x.round() as i32;
    let y = direction.y.round() as i32;

    commands
        .entity(*player)
        .insert(MoveIntent(IVec2::new(x, y)));
}

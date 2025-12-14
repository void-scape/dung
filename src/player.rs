use bevy::prelude::*;
use bevy_enhanced_input::prelude::Fire;

use crate::{input::Move, tile::PositionQuery};

#[derive(Component)]
pub struct Player;

pub fn plugin(app: &mut App) {
    // app.add_systems(Startp, |mut commands: Commands| {
    //     commands.spawn(Player);
    // })
}

// fn cool(tile_stuff: PositionQuery<Entity, With<Solid>>) {
//     if tile_stuff.iter(&some_position).count() > 0 {
//         // wow cool
//     }
// }

// fn move_player(trigger: On<Fire<Move>>, player: Single<>) {

// }

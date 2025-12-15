use crate::{
    arena::{Attack, Death},
    equipment::{EquipmentOf, Health, HealthUnit, ShieldUnit},
    input::Move,
    observer::ObserverSystem,
    tile::{MoveIntent, Solid, TilePosition, TileSprite, TileZ},
};
use bevy::{color::palettes::css::WHITE, prelude::*};
use bevy_enhanced_input::prelude::Fire;

#[derive(Component)]
#[require(
    TilePosition, TileSprite::PLAYER, TileZ(1), Solid,
    ObserverSystem::<Attack>::on(Self::observe_hit),
)]
pub struct Player;

impl Player {
    fn observe_hit(
        trigger: On<Attack>,
        mut health: Query<&mut Health>,
        mut commands: Commands,
    ) -> Result {
        let mut health = health.get_mut(trigger.entity)?;

        health.0 -= trigger.damage;
        info!("Player took {} damage", trigger.damage);
        if health.0 <= 0 {
            commands.trigger(Death {
                entity: trigger.entity,
            });
        }

        Ok(())
    }
}

impl TileSprite {
    pub const PLAYER: Self = Self {
        ascii: b'p',
        fg: Color::Srgba(WHITE),
        bg: Color::BLACK,
    };
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, |mut commands: Commands| {
        let player = commands.spawn((Player, TilePosition::new(0, 0))).id();
        commands.spawn((HealthUnit(10), EquipmentOf(player)));
        commands.spawn((ShieldUnit(10), EquipmentOf(player)));
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

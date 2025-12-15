use crate::{
    GameState,
    enemy::Enemy,
    equipment::Health,
    player::Player,
    tile::{CollisionEvent, TextAnchor, text_tiles},
};
use bevy::{input::keyboard::KeyboardInput, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_sub_state::<BattleState>()
        .add_systems(OnEnter(GameState::Arena), enter_arena)
        .add_systems(Update, player_stage.run_if(in_state(BattleState::Player)))
        .add_systems(OnExit(GameState::Arena), exit_arena)
        .add_systems(
            Update,
            (
                evaluate_win.run_if(in_state(BattleState::PlayerResult)),
                evaluate_loss.run_if(in_state(BattleState::EnemyResult)),
            ),
        )
        .add_systems(
            OnEnter(BattleState::Complete(BattleComplete::Win)),
            |mut commands: Commands| {
                info!("player won!");
                commands.set_state(GameState::Overworld);
            },
        )
        .add_systems(
            OnEnter(BattleState::Complete(BattleComplete::Loss)),
            |mut commands: Commands| {
                panic!("player lost!");
            },
        )
        .add_observer(enter_battle)
        .add_observer(observe_death);
}

fn evaluate_win(enemies: Query<(), With<BattleTarget>>, mut c: Commands) {
    if enemies.iter().len() == 0 {
        c.set_state(BattleState::Complete(BattleComplete::Win));
    } else {
        c.set_state(BattleState::EnemyResult);
    }
}

fn evaluate_loss(player: Single<&Health, With<Player>>, mut c: Commands) {
    if player.0 <= 0 {
        c.set_state(BattleState::Complete(BattleComplete::Loss));
    } else {
        c.set_state(BattleState::Player);
    }
}

fn enter_battle(
    collision: On<CollisionEvent>,
    player: Query<(), With<Player>>,
    enemies: Query<Entity, With<Enemy>>,
    mut commands: Commands,
) {
    if player.contains(collision.target) && enemies.contains(collision.collider)
        || player.contains(collision.collider) && enemies.contains(collision.target)
    {
        commands.set_state(GameState::Arena);

        for (i, enemy) in enemies.iter().enumerate() {
            commands.entity(enemy).insert(BattleTarget(i));
        }
    }
}

#[derive(Component)]
#[component(immutable)]
pub struct BattleTarget(usize);

#[derive(EntityEvent)]
pub struct Attack {
    pub entity: Entity,
    pub attacker: Entity,
    pub damage: i32,
}

#[derive(EntityEvent)]
pub struct Death {
    pub entity: Entity,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, SubStates)]
#[source(GameState = GameState::Arena)]
pub enum BattleState {
    #[default]
    Player,
    PlayerResult,
    EnemyResult,
    Complete(BattleComplete),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BattleComplete {
    Win,
    Loss,
}

#[derive(Component)]
pub struct ArenaEntity;

const ARENA: &str = r#"
+-------------------------------+
|                               |
|           F I G H T           |
|                               |
+-------------------------------+
|                               |
|                               |
|                               |
|                               |
|                               |
|                               |
|      p                 e      |
|                               |
|                               |
|                               |
|                               |
|                               |
|                               |
+-------------------------------+
|                               |
|  1. ATTACK        2. BLOCK    |
|                               |
|  3. ESCAPE                    |
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

fn player_stage(
    mut input: MessageReader<KeyboardInput>,
    player: Single<Entity, With<Player>>,
    targets: Query<(Entity, &BattleTarget)>,
    mut commands: Commands,
) {
    for input in input.read() {
        if !input.state.is_pressed() {
            continue;
        }
        match input.key_code {
            KeyCode::Digit1 => {
                commands.set_state(BattleState::PlayerResult);

                let (entity, target) = targets.iter().next().unwrap();
                commands.trigger(Attack {
                    entity,
                    attacker: *player,
                    damage: 1,
                });

                return;
            }
            KeyCode::Digit2 => {
                commands.set_state(BattleState::PlayerResult);
                return;
            }
            KeyCode::Digit3 => {
                commands.set_state(BattleState::PlayerResult);
                return;
            }
            _ => {}
        }
    }
}

fn observe_death(trigger: On<Death>, target: Query<&BattleTarget>, mut commands: Commands) {
    let Ok(battle_target) = target.get(trigger.entity) else {
        return;
    };

    commands.entity(trigger.entity).despawn();
}

fn exit_arena(
    mut commands: Commands,
    entities: Query<Entity, With<ArenaEntity>>,
    targets: Query<Entity, With<BattleTarget>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }

    for target in targets {
        commands.entity(target).remove::<BattleTarget>();
    }
}

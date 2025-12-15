use crate::{
    GameState,
    arena::{Attack, BattleState, Death},
    equipment::{Equipment, Health, HealthUnit},
    observer::ObserverSystem,
    player::Player,
    tile::{MoveIntent, Solid, TilePosition, TileSprite, TileZ},
};
use bevy::{color::palettes::tailwind::BLUE_300, prelude::*};
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::{Rng, seq::IteratorRandom};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_enemy)
        .add_systems(Update, walk.run_if(in_state(GameState::Overworld)))
        .add_systems(OnEnter(BattleState::EnemyResult), Droid::trigger_attack);
}

fn spawn_enemy(mut commands: Commands) {
    commands.spawn((
        Droid,
        TilePosition(IVec2::new(10, 10)),
        related!(Equipment[
            HealthUnit(5),
        ]),
    ));
}

#[derive(Default, Component)]
pub struct Enemy;

#[derive(Component)]
#[require(
    TileSprite::DROID,
    WalkTimer::from_secs_prob(0.2, 0.2),
    TileZ(1),
    Solid,
    Enemy,
    ObserverSystem::<Attack>::on(Self::observe_hit),
)]
pub struct Droid;

impl Droid {
    fn observe_hit(
        trigger: On<Attack>,
        mut health: Query<&mut Health>,
        mut commands: Commands,
    ) -> Result {
        let mut health = health.get_mut(trigger.entity)?;

        health.0 -= trigger.damage;
        info!("Droid took {} damage", trigger.damage);
        if health.0 <= 0 {
            commands.trigger(Death {
                entity: trigger.entity,
            });
        }

        Ok(())
    }

    fn trigger_attack(
        player: Single<Entity, With<Player>>,
        droids: Query<Entity, With<Self>>,
        mut rng: Single<&mut WyRand, With<GlobalRng>>,
        mut commands: Commands,
    ) {
        for droid in droids {
            if rng.random_bool(0.5) {
                commands.trigger(Attack {
                    entity: *player,
                    attacker: droid,
                    damage: 1,
                });
            }
        }
    }
}

impl TileSprite {
    pub const DROID: Self = Self {
        ascii: b'd',
        fg: Color::Srgba(BLUE_300),
        bg: Color::BLACK,
    };
}

#[derive(Component)]
#[require(LastWalk)]
struct WalkTimer {
    timer: Timer,
    prob: f32,
}

impl WalkTimer {
    pub fn from_secs_prob(secs: f32, prob: f32) -> Self {
        #[cfg(feature = "debug")]
        assert!((0.0..=1.0).contains(&prob));
        Self {
            timer: Timer::from_seconds(secs, TimerMode::Repeating),
            prob,
        }
    }
}

#[derive(Default, Component)]
struct LastWalk(IVec2);

fn walk(
    time: Res<Time>,
    mut timers: Query<(Entity, &mut WalkTimer, &mut LastWalk)>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    mut commands: Commands,
) {
    for (entity, mut timer, mut last_walk) in timers.iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() && rng.random_range(0.0..=1.0) <= timer.prob {
            let dir = [
                IVec2::new(-1, 0),
                IVec2::new(1, 0),
                IVec2::new(0, -1),
                IVec2::new(0, 1),
            ]
            .into_iter()
            .filter(|dir| *dir != last_walk.0)
            .choose(&mut rng)
            .unwrap();

            commands.entity(entity).insert(MoveIntent(dir));

            last_walk.0 = -dir;
        }
    }
}

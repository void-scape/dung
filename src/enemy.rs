use crate::{TILE_SIZE, tile::TileSprite};
use bevy::{color::palettes::tailwind::BLUE_300, prelude::*};
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::{Rng, seq::IteratorRandom};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_enemy)
        .add_systems(Update, walk);
}

fn spawn_enemy(mut commands: Commands) {
    commands.spawn((
        Droid,
        Transform::from_xyz(TILE_SIZE as f32 * 10.0, TILE_SIZE as f32 * 10.0, 2.0),
    ));
}

#[derive(Component)]
#[require(TileSprite::DROID, WalkTimer::from_secs_prob(0.2, 0.2))]
pub struct Droid;

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
struct LastWalk(Vec2);

fn walk(
    time: Res<Time>,
    mut timers: Query<(&mut WalkTimer, &mut LastWalk, &mut Transform)>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    for (mut timer, mut last_walk, mut transform) in timers.iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() && rng.random_range(0.0..=1.0) <= timer.prob {
            let dir = [
                Vec2::new(-1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, -1.0),
                Vec2::new(0.0, 1.0),
            ]
            .into_iter()
            .filter(|dir| *dir != last_walk.0)
            .choose(&mut rng)
            .unwrap();
            transform.translation += (dir * TILE_SIZE as f32).extend(0.0);
            last_walk.0 = -dir;
        }
    }
}

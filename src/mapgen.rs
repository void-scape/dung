use crate::tile::TileSprite;
use bevy::color::palettes::tailwind::GREEN_300;
pub use bevy::prelude::*;
use bevy_rand::{global::GlobalRng, prelude::WyRand};
use rand::Rng;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_bsp_level);
}

fn spawn_bsp_level(mut commands: Commands, mut rng: Single<&mut WyRand, With<GlobalRng>>) {
    let width = 64;
    let height = 64;
    let tiles = BSPDungeonGenerator::generate(&mut rng, width, height);

    for (y, tiles) in tiles.chunks(height).enumerate() {
        for (x, tile) in tiles.iter().enumerate() {
            commands.spawn((
                TileSprite {
                    ascii: match tile {
                        Tile::Wall => b'x',
                        Tile::Floor => b'.',
                    },
                    fg: GREEN_300.into(),
                    bg: Color::BLACK,
                },
                Transform::from_xyz(
                    (x as f32 - width as f32 / 2.0) * 16.0,
                    (y as f32 - height as f32 / 2.0) * 16.0,
                    0.0,
                ),
            ));
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Floor,
}

pub struct BSPDungeonGenerator {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    rooms: Vec<Rect>,
    rects: Vec<Rect>,
}

impl BSPDungeonGenerator {
    pub fn generate(rng: &mut impl Rng, width: usize, height: usize) -> Vec<Tile> {
        Self {
            width,
            height,
            tiles: vec![Tile::Wall; width * height],
            rooms: Vec::new(),
            rects: Vec::new(),
        }
        .build(rng)
    }

    pub fn build(mut self, rng: &mut impl Rng) -> Vec<Tile> {
        self.rects.push(Rect::new(
            2,
            2,
            self.width as i32 - 5,
            self.height as i32 - 5,
        )); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects(first_room); // Divide the first room

        // Up to 240 times, we get a random rectangle and divide it. If its possible to squeeze a
        // room in there, we place it and add it to the rooms list.
        let mut n_rooms = 0;
        while n_rooms < 240 {
            let rect = self.get_random_rect(rng);
            let candidate = self.get_random_sub_rect(rect, rng);

            if self.is_possible(candidate) {
                self.apply_room_to_map(&candidate);
                self.rooms.push(candidate);
                self.add_subrects(rect);
            }

            n_rooms += 1;
        }

        self.rooms.sort_by(|a, b| a.x1.cmp(&b.x1));

        for i in 0..self.rooms.len() - 1 {
            let room = self.rooms[i];
            let next_room = self.rooms[i + 1];
            let start_x = room.x1 + (rng.random_range(1..i32::abs(room.x1 - room.x2)) - 1);
            let start_y = room.y1 + (rng.random_range(1..i32::abs(room.y1 - room.y2)) - 1);
            let end_x =
                next_room.x1 + (rng.random_range(1..i32::abs(next_room.x1 - next_room.x2)) - 1);
            let end_y =
                next_room.y1 + (rng.random_range(1..i32::abs(next_room.y1 - next_room.y2)) - 1);
            self.draw_corridor(start_x, start_y, end_x, end_y);
        }

        self.tiles
    }

    fn add_subrects(&mut self, rect: Rect) {
        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects
            .push(Rect::new(rect.x1, rect.y1, half_width, half_height));
        self.rects.push(Rect::new(
            rect.x1,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1 + half_width,
            rect.y1,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1 + half_width,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
    }

    fn get_random_rect(&mut self, rng: &mut impl Rng) -> Rect {
        if self.rects.len() == 1 {
            return self.rects[0];
        }
        let idx = (rng.random_range(1..(self.rects.len() as i32)) - 1) as usize;
        self.rects[idx]
    }

    fn get_random_sub_rect(&self, rect: Rect, rng: &mut impl Rng) -> Rect {
        let mut result = rect;
        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);

        let w = i32::max(3, rng.random_range(1..i32::min(rect_width, 10)) - 1) + 1;
        let h = i32::max(3, rng.random_range(1..i32::min(rect_height, 10)) - 1) + 1;

        result.x1 += rng.random_range(1..6) - 1;
        result.y1 += rng.random_range(1..6) - 1;
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;

        result
    }

    fn is_possible(&self, rect: Rect) -> bool {
        let mut expanded = rect;
        expanded.x1 -= 2;
        expanded.x2 += 2;
        expanded.y1 -= 2;
        expanded.y2 += 2;

        let mut can_build = true;

        for y in expanded.y1..=expanded.y2 {
            for x in expanded.x1..=expanded.x2 {
                if x > self.width as i32 - 2 {
                    can_build = false;
                }
                if y > self.height as i32 - 2 {
                    can_build = false;
                }
                if x < 1 {
                    can_build = false;
                }
                if y < 1 {
                    can_build = false;
                }
                if can_build && self.tiles[y as usize * self.width + x as usize] != Tile::Wall {
                    can_build = false;
                }
            }
        }

        can_build
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                self.tiles[y as usize * self.width + x as usize] = Tile::Floor;
            }
        }
    }

    fn draw_corridor(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut x = x1;
        let mut y = y1;

        while x != x2 || y != y2 {
            if x < x2 {
                x += 1;
            } else if x > x2 {
                x -= 1;
            } else if y < y2 {
                y += 1;
            } else if y > y2 {
                y -= 1;
            }

            self.tiles[y as usize * self.width + x as usize] = Tile::Floor;
        }
    }
}

#[derive(Copy, Clone)]
pub struct Rect {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
}

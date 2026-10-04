// SPDX-License-Identifier: MPL-2.0

//! Pure movement/animation state of the pet; independent of any UI toolkit.

/// Number of sprite frames in the walk cycle.
pub const FRAME_COUNT: usize = 4;

/// Walking speed in pixels per second.
const WALK_SPEED: f32 = 30.0;
/// Running speed in pixels per second (used while the CPU is busy).
const RUN_SPEED: f32 = 90.0;
/// Ticks between sprite frame changes while walking.
const WALK_TICKS_PER_FRAME: u32 = 3;
/// Ticks between sprite frame changes while running.
const RUN_TICKS_PER_FRAME: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Towards the start of the track (the sprites face this way natively).
    Backward,
    Forward,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pet {
    /// Offset of the sprite from the start of the track, in pixels.
    pub position: f32,
    pub direction: Direction,
    /// Index into the walk cycle, `0..FRAME_COUNT`.
    pub frame: usize,
    ticks: u32,
}

impl Default for Pet {
    fn default() -> Self {
        Self {
            position: 0.0,
            direction: Direction::Forward,
            frame: 0,
            ticks: 0,
        }
    }
}

impl Pet {
    /// Advances the pet by `dt` seconds along a track of `track_len` pixels, with a sprite
    /// of `sprite_len` pixels. Bounces at both ends.
    pub fn tick(&mut self, dt: f32, running: bool, track_len: f32, sprite_len: f32) {
        let max = (track_len - sprite_len).max(0.0);
        let speed = if running { RUN_SPEED } else { WALK_SPEED };
        let delta = speed * dt;
        match self.direction {
            Direction::Forward => self.position += delta,
            Direction::Backward => self.position -= delta,
        }
        if self.position >= max {
            self.position = max;
            self.direction = Direction::Backward;
        } else if self.position <= 0.0 {
            self.position = 0.0;
            self.direction = Direction::Forward;
        }

        self.ticks += 1;
        let per_frame = if running {
            RUN_TICKS_PER_FRAME
        } else {
            WALK_TICKS_PER_FRAME
        };
        if self.ticks >= per_frame {
            self.ticks = 0;
            self.frame = (self.frame + 1) % FRAME_COUNT;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounces_back_and_forth() {
        let mut pet = Pet::default();
        for _ in 0..1000 {
            pet.tick(0.1, true, 100.0, 20.0);
            assert!((0.0..=80.0).contains(&pet.position));
        }
        let mut pet = Pet::default();
        pet.tick(1.0, false, 100.0, 20.0);
        assert_eq!(pet.direction, Direction::Forward);
        pet.tick(1.0, true, 100.0, 20.0);
        assert_eq!(pet.position, 80.0);
        assert_eq!(pet.direction, Direction::Backward);
        pet.tick(0.5, true, 100.0, 20.0);
        assert_eq!(pet.position, 35.0);
        pet.tick(1.0, true, 100.0, 20.0);
        assert_eq!(pet.position, 0.0);
        assert_eq!(pet.direction, Direction::Forward);
    }

    #[test]
    fn frames_cycle() {
        let mut pet = Pet::default();
        for _ in 0..(WALK_TICKS_PER_FRAME as usize * FRAME_COUNT) {
            pet.tick(0.05, false, 100.0, 20.0);
        }
        assert_eq!(pet.frame, 0);
    }

    #[test]
    fn tiny_track_does_not_move() {
        let mut pet = Pet::default();
        pet.tick(1.0, true, 10.0, 20.0);
        assert_eq!(pet.position, 0.0);
    }
}

use std::time::{Duration, Instant};

use rand::{thread_rng, Rng};
use smitten::Vec2;

use crate::util::Cooldown;

pub trait Weapon: core::fmt::Debug {
	fn can_fire(&self) -> bool {
		self.cooldown().is_ready() && !self.ammo().is_empty()
	}

	fn ammo(&self) -> &Ammunition;
	fn ammo_mut(&mut self) -> &mut Ammunition;

	fn damage(&self) -> f32;
	fn damage_mut(&mut self) -> &mut f32;

	fn cooldown(&self) -> &Cooldown;
	fn cooldown_mut(&mut self) -> &mut Cooldown;

	fn bullets(&self, direction: Vec2) -> Vec<Bullet>;

	fn name(&self) -> &'static str;
}

macro_rules! weapon_common_impl {
	($name:literal) => {
		fn ammo(&self) -> &Ammunition {
			&self.ammo
		}

		fn ammo_mut(&mut self) -> &mut Ammunition {
			&mut self.ammo
		}

		fn damage(&self) -> f32 {
			self.damage
		}

		fn damage_mut(&mut self) -> &mut f32 {
			&mut self.damage
		}

		fn cooldown(&self) -> &Cooldown {
			&self.cooldown
		}

		fn cooldown_mut(&mut self) -> &mut Cooldown {
			&mut self.cooldown
		}

		fn name(&self) -> &'static str {
			$name
		}
	};
}

#[derive(Debug)]
pub enum Ammunition {
	Infinite,
	Limited { capacity: u32, rounds: u32 },
}

impl Ammunition {
	#[allow(dead_code)]
	pub fn is_infinte(&self) -> bool {
		if let Self::Infinite = self {
			true
		} else {
			false
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Self::Infinite => false,
			Self::Limited { rounds, .. } => *rounds == 0,
		}
	}

	pub fn decrement(&mut self) {
		match self {
			Self::Limited { rounds, .. } if *rounds > 0 => *rounds -= 1,
			_ => (),
		}
	}

	pub fn reload(&mut self) {
		if let Self::Limited { capacity, rounds } = self {
			*rounds = *capacity;
		}
	}

	pub fn scale_magazine(&mut self, scalar: f32) {
		if let Self::Limited { capacity, .. } = self {
			*capacity = (*capacity as f32 * scalar).round() as u32;
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bullet {
	pub position: Vec2,
	pub velocity: Vec2,
	pub birth: Instant,
	pub damage: f32,
}

impl Bullet {
	pub fn new(position: Vec2, velocity: Vec2, damage: f32) -> Self {
		Self {
			position,
			velocity,
			birth: Instant::now(),
			damage,
		}
	}
}

#[derive(Debug)]
pub struct Pistol {
	cooldown: Cooldown,
	ammo: Ammunition,
	damage: f32,
}

impl Weapon for Pistol {
	weapon_common_impl!("Pistol");

	fn bullets(&self, direction: Vec2) -> Vec<Bullet> {
		let direction = direction.angle() + thread_rng().gen_range(-5.0..5.0);

		vec![Bullet::new(
			Vec2::ZERO,
			Vec2::from_degrees(direction) * crate::Game::BULLET_SPEED,
			self.damage,
		)]
	}
}

impl Default for Pistol {
	fn default() -> Self {
		Self {
			cooldown: Cooldown::ready(Duration::from_secs_f32(0.5)),
			ammo: Ammunition::Infinite,
			damage: 7.5,
		}
	}
}

#[derive(Debug)]
pub struct Uzi {
	cooldown: Cooldown,
	ammo: Ammunition,
	damage: f32,
}

impl Weapon for Uzi {
	weapon_common_impl!("Uzi");

	fn bullets(&self, direction: Vec2) -> Vec<Bullet> {
		let direction = direction.angle() + thread_rng().gen_range(-5.0..5.0);

		vec![Bullet::new(
			Vec2::ZERO,
			Vec2::from_degrees(direction) * crate::Game::BULLET_SPEED,
			self.damage,
		)]
	}
}

impl Default for Uzi {
	fn default() -> Self {
		Self {
			cooldown: Cooldown::ready(Duration::from_secs_f32(0.1)),
			ammo: Ammunition::Limited {
				capacity: 30,
				rounds: 0,
			},
			damage: 6.5,
		}
	}
}

#[derive(Debug)]
pub struct Shotgun {
	cooldown: Cooldown,
	ammo: Ammunition,
	damage: f32,
}

impl Weapon for Shotgun {
	weapon_common_impl!("Shotgun");

	fn bullets(&self, direction: Vec2) -> Vec<Bullet> {
		let mut inaccuracy = 1.5;
		std::iter::repeat_with(|| {
			let direction = direction.angle() + thread_rng().gen_range(-inaccuracy..inaccuracy);
			inaccuracy += 4.25;

			Bullet::new(
				Vec2::ZERO,
				Vec2::from_degrees(direction) * crate::Game::BULLET_SPEED,
				self.damage,
			)
		})
		.take(3)
		.collect()
	}
}

impl Default for Shotgun {
	fn default() -> Self {
		Self {
			cooldown: Cooldown::ready(Duration::from_secs_f32(1.0)),
			ammo: Ammunition::Limited {
				capacity: 10,
				rounds: 0,
			},
			damage: 15.0,
		}
	}
}

#[derive(Debug)]
pub struct Wall {
	cooldown: Cooldown,
	ammo: Ammunition,
	damage: f32,
}

impl Weapon for Wall {
	weapon_common_impl!("Walls");

	fn bullets(&self, _direction: Vec2) -> Vec<Bullet> {
		unreachable!("Called bullets on wall")
	}
}

impl Default for Wall {
	fn default() -> Self {
		Self {
			cooldown: Cooldown::ready(Duration::from_secs_f32(0.25)),
			ammo: Ammunition::Limited {
				capacity: 5,
				rounds: 0,
			},
			damage: 0.0,
		}
	}
}

#[derive(Debug)]
pub struct Barrel {
	cooldown: Cooldown,
	ammo: Ammunition,
	damage: f32,
}

impl Weapon for Barrel {
	weapon_common_impl!("Barrels");

	fn bullets(&self, _direction: Vec2) -> Vec<Bullet> {
		unreachable!("Called bullets on barrel")
	}
}

impl Default for Barrel {
	fn default() -> Self {
		Self {
			cooldown: Cooldown::ready(Duration::from_secs_f32(0.25)),
			ammo: Ammunition::Limited {
				capacity: 5,
				rounds: 0,
			},
			damage: 0.0,
		}
	}
}

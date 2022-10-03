use std::{num::NonZeroUsize, time::Duration};

use rand::{thread_rng, Rng};
use smitten::Vec2;

use crate::{things::Bullet, util::Cooldown};

pub trait Weapon: core::fmt::Debug {
	fn can_fire(&self) -> bool {
		self.cooldown().is_ready() && !self.ammo().is_empty()
	}

	fn ammo(&self) -> &Ammunition;
	fn ammo_mut(&mut self) -> &mut Ammunition;

	fn cooldown(&self) -> &Cooldown;
	fn cooldown_mut(&mut self) -> &mut Cooldown;

	fn bullets(&self, direction: Vec2) -> Vec<Bullet>;

	fn name(&self) -> &'static str;
}

#[derive(Debug)]
pub enum Ammunition {
	Infinite,
	Limited { capacity: u32, rounds: u32 },
}

impl Ammunition {
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
}

#[derive(Debug)]
pub struct Pistol {
	cooldown: Cooldown,
	ammo: Ammunition,
}

impl Weapon for Pistol {
	fn ammo(&self) -> &Ammunition {
		&self.ammo
	}

	fn ammo_mut(&mut self) -> &mut Ammunition {
		&mut self.ammo
	}

	fn cooldown(&self) -> &Cooldown {
		&self.cooldown
	}

	fn cooldown_mut(&mut self) -> &mut Cooldown {
		&mut self.cooldown
	}

	fn bullets(&self, direction: Vec2) -> Vec<Bullet> {
		let direction = direction.angle() + thread_rng().gen_range(-5.0..5.0);

		vec![Bullet::new(
			Vec2::ZERO,
			Vec2::from_degrees(direction) * crate::Game::BULLET_SPEED,
			10.0,
		)]
	}

	fn name(&self) -> &'static str {
		"Pistol"
	}
}

impl Default for Pistol {
	fn default() -> Self {
		Self {
			cooldown: Cooldown::ready(Duration::from_secs_f32(0.35)),
			ammo: Ammunition::Infinite,
		}
	}
}

#[derive(Debug)]
pub struct Uzi {
	cooldown: Cooldown,
	ammo: Ammunition,
}

impl Weapon for Uzi {
	fn ammo(&self) -> &Ammunition {
		&self.ammo
	}

	fn ammo_mut(&mut self) -> &mut Ammunition {
		&mut self.ammo
	}

	fn cooldown(&self) -> &Cooldown {
		&self.cooldown
	}

	fn cooldown_mut(&mut self) -> &mut Cooldown {
		&mut self.cooldown
	}

	fn bullets(&self, direction: Vec2) -> Vec<Bullet> {
		let direction = direction.angle() + thread_rng().gen_range(-5.0..5.0);

		vec![Bullet::new(
			Vec2::ZERO,
			Vec2::from_degrees(direction) * crate::Game::BULLET_SPEED,
			7.5,
		)]
	}

	fn name(&self) -> &'static str {
		"Uzi"
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
		}
	}
}
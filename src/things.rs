use std::time::Instant;

use smitten::{Color, Vec2};

use crate::{
	traits::{Colideable, Destructible, Hittable},
	util::Cooldown,
	BoundingCircle, Game,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Bullet {
	pub position: Vec2,
	pub velocity: Vec2,
	pub birth: Instant,
}

impl Bullet {
	pub fn new(position: Vec2, velocity: Vec2) -> Self {
		Self {
			position,
			velocity,
			birth: Instant::now(),
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Enemy {
	pub position: Vec2,
	pub color: Color,
	pub health: f32,
	pub speed: f32,
	pub cooldown: Cooldown,
	pub should_move_next_frame: bool,
}

impl Colideable for Enemy {
	fn bounds(&self) -> BoundingCircle {
		BoundingCircle {
			position: self.position,
			radius: Game::PLAYER_LENGTH,
		}
	}

	fn position_mut(&mut self) -> &mut Vec2 {
		&mut self.position
	}
}

impl Hittable for Enemy {
	fn hit(&mut self) {
		self.health -= 1.0;
	}
}

impl Destructible for Enemy {
	fn health(&self) -> f32 {
		self.health
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Barrel {
	pub position: Vec2,
	pub health: f32,
}

impl Barrel {
	pub const BARREL_HEALTH: f32 = 10.0;

	pub fn damage_color(&self) -> Color {
		crate::color_lerp(
			Color::GREEN,
			Color::RED,
			self.health / Barrel::BARREL_HEALTH,
		)
	}
}

impl Colideable for Barrel {
	fn bounds(&self) -> BoundingCircle {
		BoundingCircle {
			position: self.position,
			radius: 1.0,
		}
	}

	fn position_mut(&mut self) -> &mut Vec2 {
		&mut self.position
	}
}

impl Hittable for Barrel {
	fn hit(&mut self) {
		self.health -= 1.0;
	}
}

impl Destructible for Barrel {
	fn health(&self) -> f32 {
		self.health
	}
}

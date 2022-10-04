use std::time::Instant;

use smitten::{Color, Vec2};

use crate::{
	traits::{Colideable, Destructible, Explosive, ExplosiveDetails, Hittable},
	util::Cooldown,
	weapon::Bullet,
	BoundingCircle, Game,
};

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
	fn hit(&mut self, bullet: &Bullet) {
		self.health -= bullet.damage;
	}
}

impl Destructible for Enemy {
	fn health(&self) -> f32 {
		self.health
	}

	fn health_mut(&mut self) -> &mut f32 {
		&mut self.health
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wall {
	pub position: Vec2,
	pub health: f32,
}

impl Wall {
	pub const WALL_HEALTH: f32 = 100.0;

	pub fn damage_color(&self) -> Color {
		crate::color_lerp(Color::WHITE, Color::BLACK, self.health / Wall::WALL_HEALTH)
	}
}

impl Colideable for Wall {
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

impl Hittable for Wall {
	fn hit(&mut self, bullet: &Bullet) {
		self.health -= bullet.damage;
	}
}

impl Destructible for Wall {
	fn health(&self) -> f32 {
		self.health
	}

	fn health_mut(&mut self) -> &mut f32 {
		&mut self.health
	}
}

pub struct Pickup {
	pub position: Vec2,
}

impl Colideable for Pickup {
	fn bounds(&self) -> BoundingCircle {
		BoundingCircle {
			position: self.position,
			radius: Game::PLAYER_LENGTH / 2.0,
		}
	}

	fn position_mut(&mut self) -> &mut Vec2 {
		&mut self.position
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Barrel {
	pub position: Vec2,
	pub health: f32,
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
	fn hit(&mut self, _bullet: &Bullet) {
		self.health = 0.0;
	}
}

impl Destructible for Barrel {
	fn health(&self) -> f32 {
		self.health
	}

	fn health_mut(&mut self) -> &mut f32 {
		&mut self.health
	}
}

impl Explosive for Barrel {
	fn details(&self) -> crate::traits::ExplosiveDetails {
		ExplosiveDetails::new(25.0, self.position, 1.5)
	}
}

use smitten::Vec2;

use crate::{weapon::Bullet, BoundingCircle};

pub trait Colideable {
	fn bounds(&self) -> BoundingCircle;
	fn position_mut(&mut self) -> &mut Vec2;

	fn colides_with<C: Colideable>(&self, other: &C) -> bool {
		self.bounds()
			.position
			.distance_with(other.bounds().position)
			< self.bounds().radius
	}
}

pub trait Hittable: Colideable {
	fn hit(&mut self, bullet: &Bullet);

	fn was_hit(&self, bullet: &Bullet) -> bool {
		let bounds = self.bounds();
		bounds.position.distance_with(bullet.position) < (bounds.radius / 2.0)
	}
}

pub trait Destructible {
	fn health(&self) -> f32;
	fn health_mut(&mut self) -> &mut f32;
}

pub trait Explosive {
	fn details(&self) -> ExplosiveDetails;

	fn explode_on<T>(&self, thing: &mut T, knockback: bool)
	where
		T: Colideable + Destructible,
	{
		let direction = thing.bounds().position - self.details().position;
		let magnitude = 1.0 / direction.length();
		*thing.health_mut() -= self.details().damage * magnitude;

		if knockback {
			*thing.position_mut() += direction * magnitude;
		}
	}
}

pub struct ExplosiveDetails {
	pub damage: f32,
	pub position: Vec2,
	pub radius: f32,
}

impl ExplosiveDetails {
	pub fn new(damage: f32, position: Vec2, radius: f32) -> Self {
		Self {
			damage,
			position,
			radius,
		}
	}
}

impl Colideable for ExplosiveDetails {
	fn bounds(&self) -> BoundingCircle {
		BoundingCircle {
			position: self.position,
			radius: self.radius,
		}
	}

	fn position_mut(&mut self) -> &mut Vec2 {
		&mut self.position
	}
}

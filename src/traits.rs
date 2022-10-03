use smitten::Vec2;

use crate::{things::Bullet, BoundingCircle};

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
}

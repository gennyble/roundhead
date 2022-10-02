use smitten::Vec2;

use crate::{things::Bullet, BoundingCircle};

pub trait Colideable {
	fn bounds(&self) -> BoundingCircle;
	fn position_mut(&mut self) -> &mut Vec2;
}

pub trait Hittable: Colideable {
	fn hit(&mut self);

	fn was_hit(&self, bullet: &Bullet) -> bool {
		let bounds = self.bounds();
		bounds.position.distance_with(bullet.position) < bounds.radius
	}
}

pub trait Destructible {
	fn health(&self) -> f32;
}

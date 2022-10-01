use crate::{things::Bullet, BoundingCircle};

pub trait Hittable {
	fn bounding_circle(&self) -> BoundingCircle;

	fn hit(&mut self);

	fn was_hit(&self, bullet: &Bullet) -> bool {
		let bounds = self.bounding_circle();
		bounds.position.distance_with(bullet.position) < bounds.radius
	}
}

pub trait Destructible {
	fn health(&self) -> f32;
}

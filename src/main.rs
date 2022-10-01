#![feature(drain_filter)]

mod things;
mod traits;
mod util;

use things::{Barrel, Bullet, Enemy};
use traits::{Destructible, Hittable};
use util::Cooldown;

use std::{
	ops::{Add, Mul, Sub},
	time::{Duration, Instant},
};

use smitten::{
	Color, Draw, HorizontalAnchor, Key, SignedDistance, Smitten, SmittenEvent, Vec2, VerticalAnchor,
};

const TURQUOISE: Color = Color::rgb(
	0x33 as f32 / 256.0,
	0xaa as f32 / 256.0,
	0x88 as f32 / 256.0,
);
const PURPLE: Color = Color::rgb(0.9, 0.8, 0.85);
const MUR: u32 = 48;
const DIM: (u32, u32) = (1280, 960);

fn main() {
	let smitty = Smitten::new(DIM, "Roundhead", MUR);

	let cooldown = Cooldown::ready(Duration::from_secs(1));

	let mut game = Game {
		smitten: smitty,
		camera: Vec2::ZERO,
		bullets: vec![],
		enemies: vec![
			Enemy {
				position: Vec2::new(5.0, 5.0),
				color: PURPLE,
				health: 1.0,
				speed: 1.25,
				cooldown,
			},
			Enemy {
				position: Vec2::new(5.0, 3.0),
				color: Color::BLUE,
				health: 1.0,
				speed: 2.0,
				cooldown,
			},
			Enemy {
				position: Vec2::new(5.0, 3.0),
				color: Color::BLUE,
				health: 1.0,
				speed: 2.2,
				cooldown,
			},
			Enemy {
				position: Vec2::new(5.0, 3.0),
				color: Color::BLUE,
				health: 1.0,
				speed: 1.0,
				cooldown,
			},
		],
		last_render: Instant::now(),
		score: 0.0,
		health: 10.0,
		barrels: vec![],
		barrel_count: 5,
	};

	loop {
		let events = game.smitten.events();

		events.iter().for_each(|e| match e {
			SmittenEvent::MouseDown { button } => {
				let pos = game.smitten.mouse_position_absolute();
				let angle = pos.normalize_correct().angle();
				println!("{pos} - angle {angle}");
				game.shoot();
			}
			SmittenEvent::Keydown { key, .. } => {
				if let Some(Key::E) = key {
					game.place_barrel();
				}
			}
			_ => (),
		});

		if game.smitten.is_key_down(Key::Escape) {
			break;
		}

		let mut movec = Vec2::ZERO;
		if game.smitten.is_key_down(Key::W) {
			movec += Vec2::new(0.0, 1.0);
		} else if game.smitten.is_key_down(Key::S) {
			movec -= Vec2::new(0.0, 1.0);
		}

		if game.smitten.is_key_down(Key::A) {
			movec -= Vec2::new(1.0, 0.0);
		} else if game.smitten.is_key_down(Key::D) {
			movec += Vec2::new(1.0, 0.0);
		}

		movec = movec.normalize_correct() * (1.5 / 32.0);
		game.camera += movec;

		game.tick();

		// Draw
		game.smitten.clear();
		game.draw();
		game.smitten.swap();
	}
}

// A higher level struct so I can keep a camera et al.
struct Game {
	smitten: Smitten,
	camera: Vec2,
	bullets: Vec<Bullet>,
	enemies: Vec<Enemy>,
	last_render: Instant,
	score: f32,
	health: f32,
	barrels: Vec<Barrel>,
	barrel_count: usize,
}

impl Game {
	const BULLET_LIFESPAN: Duration = Duration::from_secs(1);
	const BULLET_SPEED: f32 = 20.0;
	const BULLET_WIDTH: u32 = 4;
	const BULLET_WIDTH_MUR: f32 = Game::BULLET_WIDTH as f32 / MUR as f32;
	const PLAYER_LENGTH: f32 = 0.75;
	const PLAYER_DIM: Vec2 = Vec2::new(Game::PLAYER_LENGTH, Game::PLAYER_LENGTH);
	const PLAYER_HEALTH_MAX: f32 = 10.0;

	pub fn rect<P: Into<Vec2>, D: Into<Vec2>, R: Into<Draw>>(&self, pos: P, dim: D, draw: R) {
		self.smitten.rect(pos.into() - self.camera, dim, draw)
	}

	pub fn draw(&self) {
		self.draw_grid();

		for bullet in &self.bullets {
			self.smitten.sdf(SignedDistance::Circle {
				center: bullet.position - self.camera,
				radius: 4,
				color: Color::rgb(1.0, 0.0, 0.0),
			})
		}

		for barrel in &self.barrels {
			self.smitten.sdf(SignedDistance::Circle {
				center: barrel.position - self.camera,
				radius: MUR / 2,
				color: barrel.damage_color(),
			})
		}

		for enemy in &self.enemies {
			self.rect(enemy.position, Game::PLAYER_DIM, enemy.color)
		}

		// Draw us. We're not affected by camera movement
		self.smitten.rect((0f32, 0f32), Game::PLAYER_DIM, TURQUOISE);

		self.smitten.anchored_rect(
			(HorizontalAnchor::Left, VerticalAnchor::Bottom),
			(10.0, 0.75),
			Color::rgb(0.2, 0.2, 0.2),
		);

		self.smitten.anchored_rect(
			(HorizontalAnchor::Left, VerticalAnchor::Bottom),
			(10.0 * (self.health / Self::PLAYER_HEALTH_MAX), 0.75),
			Color::rgb(0.75, 0.0, 0.0),
		)
	}

	pub fn tick(&mut self) {
		let now = Instant::now();
		let delta = now.duration_since(self.last_render);
		self.last_render = now;
		let dsec = delta.as_secs_f64();

		self.bullets = self
			.bullets
			.drain_filter(|bul| now.duration_since(bul.birth) < Game::BULLET_LIFESPAN)
			.collect();

		self.bullets
			.iter_mut()
			.for_each(|bul| bul.position += bul.velocity * dsec as f32);

		let hits = Self::do_bullet_hits(&mut self.enemies, &mut self.bullets);
		Self::burry_dead(&mut self.enemies);
		self.tick_enemies(delta);

		let barrel_hits = Self::do_bullet_hits(&mut self.barrels, &mut self.bullets);
		Self::burry_dead(&mut self.barrels);
	}

	pub fn shoot(&mut self) {
		let direction = self.smitten.mouse_position().normalize_correct();
		let bullet = Bullet::new(self.camera, direction * Game::BULLET_SPEED);
		self.bullets.push(bullet);
	}

	pub fn place_barrel(&mut self) {
		if self.barrel_count == 0 {
			return;
		}

		let direction = self.smitten.mouse_position_absolute().normalize_correct();

		let place_direction = if direction.x.abs() > direction.y.abs() {
			// It's horizontal
			if direction.x > 0.0 {
				Vec2::new(1.0, 0.0)
			} else {
				Vec2::new(-1.0, 0.0)
			}
		} else {
			// Vertical
			if direction.y > 0.0 {
				Vec2::new(0.0, 1.0)
			} else {
				Vec2::new(0.0, -1.0)
			}
		};

		let position = (self.camera + place_direction).operation(f32::round);

		if self.has_barrel_at(position) {
			println!("Barrel already at {position}, not placing another!");
			return;
		}
		self.barrel_count -= 1;

		self.barrels.push(Barrel {
			position,
			health: Barrel::BARREL_HEALTH,
		});
	}

	fn has_barrel_at(&self, pos: Vec2) -> bool {
		self.barrels
			.iter()
			.find(|bar| bar.position == pos)
			.is_some()
	}

	fn do_bullet_hits<H: Hittable>(hittables: &mut Vec<H>, bullets: &mut Vec<Bullet>) {
		'enemy: for enemy in hittables.iter_mut() {
			let mut unhit_bullets = vec![];

			loop {
				match bullets.pop() {
					None => {
						bullets.extend(unhit_bullets);
						break;
					}
					Some(bullet) => {
						if enemy.was_hit(&bullet) {
							enemy.hit();
							bullets.extend(unhit_bullets);
							continue 'enemy;
						} else {
							unhit_bullets.push(bullet);
						}
					}
				}
			}
		}
	}

	// Why did you choose this name lol
	fn burry_dead<D: Destructible>(things: &mut Vec<D>) -> Vec<D> {
		let (alive, dead) = things.drain(..).partition(|d| d.health() > 0.0);
		things.extend(alive);
		dead
	}

	fn draw_grid(&self) {
		let mur_width = (DIM.0 / MUR) + 3;
		let mur_height = (DIM.1 / MUR) + 3;

		for x in 0..mur_width {
			for y in 0..mur_height {
				let x = x as f32 - mur_width as f32 / 2.0;
				let y = y as f32 - mur_height as f32 / 2.0;

				let camera = self.camera.operation(f32::fract);

				let pos = Vec2::new(x.floor(), y.floor()) - camera;
				self.smitten.sdf(SignedDistance::Circle {
					center: pos,
					radius: 4,
					color: Color::grey(0.5),
				});
			}
		}
	}

	fn tick_enemies(&mut self, delta: Duration) {
		for enemy in self.enemies.iter_mut() {
			enemy.cooldown.subtract(delta);

			if enemy.cooldown.is_ready() {
				let dist = (enemy.position - self.camera).length();

				if dist < Self::PLAYER_LENGTH {
					enemy.cooldown.reset();
					self.health -= 1.0;
				}
			}
		}

		//Movement
		let mut moved = vec![];

		let fix = |enemy: &mut Enemy, others: &mut [Enemy]| -> bool {
			let mut moved = false;
			others.iter_mut().for_each(|other| {
				let dist = enemy.position.distance_with(other.position);

				if dist < enemy.bounding_circle().radius {
					let dir = enemy.position - other.position;
					//desired sepration
					let wanted =
						dir.normalize_correct() * (enemy.bounding_circle().radius - dir.length());

					let collective_speed = enemy.speed + other.speed;

					enemy.position +=
						wanted * (collective_speed - (enemy.speed / collective_speed));
					other.position -=
						wanted * (collective_speed - (other.speed / collective_speed));
					moved = true;
				}
			});
			moved
		};

		loop {
			match self.enemies.pop() {
				None => break,
				Some(mut enemy) => {
					let direction = (self.camera - enemy.position).normalize_correct();
					let movement = direction * enemy.speed;
					enemy.position += movement * delta.as_secs_f32();

					fix(&mut enemy, &mut self.enemies);
					fix(&mut enemy, &mut moved);

					moved.push(enemy);
				}
			}
		}
		self.enemies.extend(moved.drain(..));
	}
}

struct BoundingCircle {
	position: Vec2,
	radius: f32,
}

fn lerp<T>(a: T, b: T, c: f32) -> T
where
	T: Clone + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T>,
{
	let diff = b - a.clone();
	a + diff * c
}

fn color_lerp(a: Color, b: Color, c: f32) -> Color {
	let c = 1.0 - c;
	let r = a.r + ((b.r - a.r) * c);
	let g = a.g + ((b.g - a.g) * c);
	let bl = a.b + ((b.b - a.b) * c);
	let a = a.a + ((b.a - a.a) * c);

	Color { r, g, b: bl, a }
}

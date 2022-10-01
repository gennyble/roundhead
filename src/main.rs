#![feature(drain_filter)]

use std::time::{Duration, Instant};

use smitten::{Color, Draw, Key, SignedDistance, Smitten, SmittenEvent, Vec2};

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

	let mut game = Game {
		smitten: smitty,
		camera: Vec2::ZERO,
		bullets: vec![],
		enemies: vec![Enemy {
			position: Vec2::new(5.0, 5.0),
			color: PURPLE,
		}],
		last_render: Instant::now(),
		score: 0.0,
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
				color: Color::rgb(0.0, 1.0, 0.0),
			})
		}

		for enemy in &self.enemies {
			self.rect(enemy.position, Game::PLAYER_DIM, enemy.color)
		}

		// Draw us. We're not affected by camera movement
		self.smitten.rect((0f32, 0f32), Game::PLAYER_DIM, TURQUOISE);
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

		let hits = self.do_hits();
		for (enemy, _) in hits {
			self.score += 1.0;
		}
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

		let position = (self.camera + place_direction).operation(f32::floor);

		if self.has_barrel_at(position) {
			println!("Barrel already at {position}, not placing another!");
			return;
		}
		self.barrel_count -= 1;

		self.barrels.push(Barrel {
			position,
			health: 1.0,
		});
	}

	fn has_barrel_at(&self, pos: Vec2) -> bool {
		self.barrels
			.iter()
			.find(|bar| bar.position == pos)
			.is_some()
	}

	fn do_hits(&mut self) -> Vec<(Enemy, Bullet)> {
		let mut hit = vec![];

		let mut enemies_alive = vec![];

		'enemy: for enemy in self.enemies.drain(..) {
			let mut unhit_bullets = vec![];

			loop {
				match self.bullets.pop() {
					None => {
						self.bullets.extend(unhit_bullets);
						break;
					}
					Some(bullet) => {
						if Self::enemy_hit(&enemy, &bullet) {
							hit.push((enemy, bullet));
							self.bullets.extend(unhit_bullets);
							continue 'enemy;
						} else {
							unhit_bullets.push(bullet);
						}
					}
				}
			}

			enemies_alive.push(enemy);
		}

		self.enemies = enemies_alive;

		hit
	}

	// Lazy collisions; everything is a circle
	fn enemy_hit(enemy: &Enemy, bullet: &Bullet) -> bool {
		enemy.position.distance_with(bullet.position) < Game::PLAYER_LENGTH
	}

	fn draw_grid(&self) {
		let mur_width = (DIM.0 / MUR) + 3;
		let mur_height = (DIM.1 / MUR) + 3;

		for x in 0..mur_width {
			for y in 0..mur_height {
				let x = x as f32 - mur_width as f32 / 2.0;
				let y = y as f32 - mur_height as f32 / 2.0;

				let camera = Vec2::new(self.camera.x.fract(), self.camera.y.fract());

				let pos = Vec2::new(x.floor(), y.floor()) - camera;
				self.smitten.sdf(SignedDistance::Circle {
					center: pos,
					radius: 4,
					color: Color::grey(0.5),
				});
			}
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
struct Bullet {
	position: Vec2,
	velocity: Vec2,
	birth: Instant,
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
struct Enemy {
	position: Vec2,
	color: Color,
}

#[derive(Clone, Debug, PartialEq)]
struct Barrel {
	position: Vec2,
	health: f32,
}

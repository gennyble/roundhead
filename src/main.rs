#![feature(drain_filter)]

use std::time::{Duration, Instant};

use smitten::{Color, Draw, Key, SignedDistance, Smitten, SmittenEvent, Vec2};

const TURQUOISE: Color = Color::rgb(
	0x33 as f32 / 256.0,
	0xaa as f32 / 256.0,
	0x88 as f32 / 256.0,
);
const PURPLE: Color = Color::rgb(0.9, 0.8, 0.85);
const MUR: u32 = 64;

fn main() {
	let smitty = Smitten::new((1280, 960), "Roundhead", MUR);

	let mut game = Game {
		smitten: smitty,
		camera: Vec2::ZERO,
		bullets: vec![],
		enemies: vec![Enemy {
			position: Vec2::new(5.0, 5.0),
			color: PURPLE,
		}],
		last_render: Instant::now(),
	};

	loop {
		let events = game.smitten.events();

		events.iter().for_each(|e| match e {
			SmittenEvent::MouseDown { button } => {
				let pos = game.smitten.mouse_position();
				println!("{}", pos);
				game.shoot();
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
}

impl Game {
	const BULLET_LIFESPAN: Duration = Duration::from_secs(1);
	const BULLET_SPEED: f32 = 20.0;
	const BULLET_WIDTH: u32 = 4;
	const BULLET_WIDTH_MUR: f32 = Game::BULLET_WIDTH as f32 / MUR as f32;
	const PLAYER_LENGTH: f32 = 0.5;
	const PLAYER_DIM: Vec2 = Vec2::new(Game::PLAYER_LENGTH, Game::PLAYER_LENGTH);

	pub fn rect<P: Into<Vec2>, D: Into<Vec2>, R: Into<Draw>>(&self, pos: P, dim: D, draw: R) {
		self.smitten.rect(pos.into() - self.camera, dim, draw)
	}

	pub fn draw(&self) {
		for bullet in &self.bullets {
			self.smitten.sdf(SignedDistance::Circle {
				center: bullet.position,
				radius: 4,
				color: Color::rgb(1.0, 0.0, 0.0),
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

		self.do_hits();
	}

	pub fn shoot(&mut self) {
		let direction = self.smitten.mouse_position().normalize_correct();
		let bullet = Bullet::new(self.camera, direction * Game::BULLET_SPEED);
		self.bullets.push(bullet);
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

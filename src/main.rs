#![feature(drain_filter)]

use std::time::{Duration, Instant};

use smitten::{Color, Draw, Key, SignedDistance, Smitten, SmittenEvent, Vec2};

const TURQUOISE: Color = Color::rgb(
	0x33 as f32 / 256.0,
	0xaa as f32 / 256.0,
	0x88 as f32 / 256.0,
);

fn main() {
	let smitty = Smitten::new((640, 480), "Roundhead", 32);

	let mut game = Game {
		smitten: smitty,
		camera: Vec2::ZERO,
		bullets: vec![],
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

		movec = movec.normalize_correct() * (3.0 / 32.0);
		game.camera += movec;

		game.tick();

		// Draw
		game.smitten.clear();
		game.draw();
		game.rect((0f32, 0f32), (1f32, 1f32), TURQUOISE);
		game.smitten.swap();
	}
}

// A higher level struct so I can keep a camera et al.
struct Game {
	smitten: Smitten,
	camera: Vec2,
	bullets: Vec<Bullet>,
	last_render: Instant,
}

impl Game {
	const BULLET_LIFESPAN: Duration = Duration::from_secs(1);
	const BULLET_SPEED: f32 = 20.0;

	pub fn rect<P: Into<Vec2>, D: Into<Vec2>, R: Into<Draw>>(&self, pos: P, dim: D, draw: R) {
		self.smitten.rect(self.camera + pos.into(), dim, draw)
	}

	pub fn draw(&self) {
		for bullet in &self.bullets {
			self.smitten.sdf(SignedDistance::Circle {
				center: bullet.position,
				radius: 4,
				color: Color::rgb(1.0, 0.0, 0.0),
			})
		}
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
	}

	pub fn shoot(&mut self) {
		let direction = self.smitten.mouse_position().normalize_correct();
		let bullet = Bullet::new(self.camera, direction * Game::BULLET_SPEED);
		self.bullets.push(bullet);
	}
}

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

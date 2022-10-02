#![feature(drain_filter)]

mod things;
mod traits;
mod util;

use rand::{thread_rng, Rng};
use things::{Barrel, Bullet, Enemy};
use traits::{Colideable, Destructible, Hittable};
use util::Cooldown;

use std::{
	ops::{Add, Mul, Sub},
	time::{Duration, Instant},
};

use smitten::{
	Color, Draw, FontId, HorizontalAnchor, Key, SignedDistance, Smitten, SmittenEvent, Vec2,
	VerticalAnchor,
};

const TURQUOISE: Color = Color::rgb8(0x33, 0xaa, 0x88);
const PURPLE: Color = Color::rgb(0.9, 0.8, 0.85);
const MUR: u32 = 48;
const DIM: (u32, u32) = (1280, 960);

fn main() {
	let mut smitty = Smitten::new(DIM, "Roundhead", MUR);

	let cooldown = Cooldown::ready(Duration::from_secs(1));
	let font = smitty.make_font("Cabin-Regular.ttf");
	smitty.clear_color(Color::grey(0.5));

	let mut game = Game {
		smitten: smitty,
		player: Player::default(),
		bullets: vec![],
		enemies: vec![],
		last_render: Instant::now(),
		score: 0.0,
		barrels: vec![],
		barrel_count: 100,
		wave_timer: Cooldown::ready(Duration::from_secs_f32(10.0)),
		font,
	};

	loop {
		let events = game.smitten.events();

		events.iter().for_each(|e| match e {
			SmittenEvent::MouseDown { button } => {
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

		if game.smitten.is_key_down(Key::Space) {
			game.shoot();
		}

		if game.smitten.is_key_down(Key::P) {
			game.score += 1.0;
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

		movec = movec.normalize_correct() * (1.25 / 32.0);
		game.player.position += movec;
		if movec != Vec2::ZERO {
			game.player.facing = movec.normalize_correct();
		}

		game.tick();

		// Draw
		game.smitten.clear();
		game.draw();
		game.smitten.swap();
	}
}

// A higher level struct so I can keep a player.position et al.
struct Game {
	smitten: Smitten,
	player: Player,
	bullets: Vec<Bullet>,
	enemies: Vec<Enemy>,
	last_render: Instant,
	score: f32,
	barrels: Vec<Barrel>,
	barrel_count: usize,
	wave_timer: Cooldown,
	font: FontId,
}

impl Game {
	const BULLET_LIFESPAN: Duration = Duration::from_secs(1);
	const BULLET_SPEED: f32 = 40.0;
	const PLAYER_LENGTH: f32 = 0.75;
	const PLAYER_DIM: Vec2 = Vec2::new(Game::PLAYER_LENGTH, Game::PLAYER_LENGTH);
	const PLAYER_HEALTH_MAX: f32 = 30.0;

	pub fn rect<P: Into<Vec2>, D: Into<Vec2>, R: Into<Draw>>(&self, pos: P, dim: D, draw: R) {
		self.smitten
			.rect(pos.into() - self.player.position, dim, draw)
	}

	pub fn draw(&self) {
		self.draw_grid();

		for bullet in &self.bullets {
			self.smitten.sdf(SignedDistance::Circle {
				center: bullet.position - self.player.position,
				radius: 2,
				color: Color::rgb(1.0, 0.0, 0.0),
			})
		}

		for barrel in &self.barrels {
			self.smitten.sdf(SignedDistance::Circle {
				center: barrel.position - self.player.position,
				radius: MUR / 2,
				color: barrel.damage_color(),
			})
		}

		for enemy in &self.enemies {
			//self.rect(enemy.position, Game::PLAYER_DIM, enemy.color)
			self.smitten.sdf(SignedDistance::Circle {
				center: enemy.position - self.player.position,
				radius: (Game::PLAYER_LENGTH * MUR as f32 / 2.0).floor() as u32,
				color: enemy.color,
			})
		}

		// Draw us. We're not affected by player.position movement
		self.smitten.sdf(SignedDistance::LineSegment {
			start: Vec2::new(0.0, 0.0),
			end: self.player.facing * 0.5,
			thickness: 2,
			color: Color::BLACK,
		});
		self.smitten.sdf(SignedDistance::Circle {
			center: Vec2::new(0.0, 0.0),
			radius: (Game::PLAYER_LENGTH * MUR as f32 / 2.0).floor() as u32,
			color: TURQUOISE,
		});

		self.draw_walls();
		self.draw_ui();
	}

	fn draw_ui(&self) {
		self.smitten.write(
			self.font,
			&format!("{}", self.score),
			(HorizontalAnchor::Center(0.0), VerticalAnchor::Top(-0.75)),
			Color::BLACK,
			1.0,
		);

		self.smitten.anchored_rect(
			(HorizontalAnchor::Left(0.0), VerticalAnchor::Top(0.0)),
			(DIM.0 as f32 / MUR as f32, 0.5),
			Color::rgba(0.0, 0.0, 0.0, 0.2),
		);

		self.smitten.anchored_rect(
			(HorizontalAnchor::Left(0.0), VerticalAnchor::Top(0.0)),
			(
				(DIM.0 as f32 / MUR as f32) * (1.0 - self.wave_timer.percent()),
				0.5,
			),
			Color::BLUE,
		);

		self.smitten.write(
			self.font,
			"PISTOL",
			(0.0, VerticalAnchor::Bottom(1.0)),
			Color::BLACK,
			0.5,
		);

		self.smitten
			.anchored_rect((0.0, 1.0), (2.0, 0.4), Color::rgba(0.0, 0.0, 0.0, 0.5));

		self.smitten.anchored_rect(
			(
				-(Self::PLAYER_HEALTH_MAX - self.player.health) / Self::PLAYER_HEALTH_MAX,
				1.0,
			),
			(1.8 * (self.player.health / Self::PLAYER_HEALTH_MAX), 0.2),
			Color::rgb(0.0, 0.75, 0.0),
		)
	}

	pub fn tick(&mut self) {
		let now = Instant::now();
		let delta = now.duration_since(self.last_render);
		self.last_render = now;
		let dsec = delta.as_secs_f64();

		self.wave_things(delta);

		self.bullets = self
			.bullets
			.drain_filter(|bul| now.duration_since(bul.birth) < Game::BULLET_LIFESPAN)
			.collect();

		self.bullets
			.iter_mut()
			.for_each(|bul| bul.position += bul.velocity * dsec as f32);

		Self::collide_walls(&mut self.player);
		self.barrels.iter().for_each(|barrel| {
			colide_and_move(barrel, &mut self.player);
		});
		self.player.weapon.cooldown_mut().subtract(delta);

		let hits = Self::do_bullet_hits(&mut self.enemies, &mut self.bullets);
		Self::burry_dead(&mut self.enemies)
			.iter()
			.for_each(|e| self.score += 123.0);
		self.tick_enemies(delta);

		let barrel_hits = Self::do_bullet_hits(&mut self.barrels, &mut self.bullets);
		Self::burry_dead(&mut self.barrels);
	}

	pub fn shoot(&mut self) {
		if !self.player.weapon.is_ready() {
			return;
		}
		self.player.weapon.cooldown_mut().reset();

		for mut bull in self.player.weapon.bullets(self.player.facing) {
			bull.position = self.player.position;

			self.bullets.push(bull);
		}
	}

	pub fn place_barrel(&mut self) {
		if self.barrel_count == 0 {
			return;
		}

		let direction = self.player.facing;

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

		let position = (self.player.position + place_direction).operation(f32::round);

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
							enemy.hit(&bullet);
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

	const ROOM_WIDTH: f32 = 40f32;
	const ROOM_HEIGHT: f32 = 40f32;
	const WALL_WIDTH: f32 = 1.0;

	fn draw_walls(&self) {
		let room_width = Self::ROOM_WIDTH;
		let room_height = Self::ROOM_HEIGHT;

		let hrw = room_width / 2.0;
		let hrh = room_height / 2.0;

		let walls = [
			(
				(-hrw, 0.0),
				(Self::WALL_WIDTH, room_height + Self::WALL_WIDTH),
			),
			(
				(hrw, 0.0),
				(Self::WALL_WIDTH, room_height + Self::WALL_WIDTH),
			),
			(
				(0.0, hrh),
				(room_width + Self::WALL_WIDTH, Self::WALL_WIDTH),
			),
			(
				(0.0, -hrh),
				(room_width + Self::WALL_WIDTH, Self::WALL_WIDTH),
			),
		];

		for (pos, dim) in walls {
			self.rect(pos, dim, Color::BLACK)
		}
	}

	fn collide_walls<C: Colideable>(thing: &mut C) {
		let bounds = thing.bounds();
		let p = bounds.position;
		let r = bounds.radius;

		let top = p.y + r;
		let btm = p.y - r;
		let lft = p.x - r;
		let rht = p.x + r;

		let hrw = Game::ROOM_WIDTH / 2.0;
		let hrh = Game::ROOM_HEIGHT / 2.0;

		let mpos = thing.position_mut();
		if top > hrh {
			mpos.y = hrh - r;
		} else if btm < -hrh {
			mpos.y = -hrh + r;
		}

		if lft < -hrw {
			mpos.x = -hrw + r;
		} else if rht > hrw {
			mpos.x = hrw - r;
		}
	}

	fn draw_grid(&self) {
		let mur_width = (DIM.0 / MUR) + 4;
		let mur_height = (DIM.1 / MUR) + 4;

		for x in 0..mur_width {
			for y in 0..mur_height {
				let x = x as f32 - mur_width as f32 / 2.0;
				let y = y as f32 - mur_height as f32 / 2.0;

				let camera = self.player.position.operation(f32::fract);

				let pos = Vec2::new(x.floor(), y.floor()) - camera;
				/*self.smitten.sdf(SignedDistance::Circle {
					center: pos,
					radius: 4,
					color: Color::grey(0.5),
				});*/
				let pixel_gap = (MUR as f32 - 1.0) / MUR as f32;
				self.smitten.rect(
					pos,
					Vec2::new(pixel_gap, pixel_gap),
					Color::rgb(0.95, 0.95, 0.85),
				)
			}
		}
	}

	fn tick_enemies(&mut self, delta: Duration) {
		for enemy in self.enemies.iter_mut() {
			enemy.cooldown.subtract(delta);

			if colide_and_move(&self.player, enemy) {
				enemy.should_move_next_frame = false;
				if enemy.cooldown.is_ready() {
					enemy.cooldown.reset();
					self.player.health -= 6.66;
				}
			}

			for barrel in self.barrels.iter_mut() {
				enemy.should_move_next_frame = false;
				if colide_and_move(barrel, enemy) {
					if enemy.cooldown.is_ready() {
						enemy.cooldown.reset();
						barrel.health -= 6.66;
					}
				}
			}
		}

		//Movement
		let mut moved = vec![];

		let fix = |enemy: &mut Enemy, others: &mut [Enemy]| -> bool {
			let mut moved = false;
			others.iter_mut().for_each(|other| {
				let dist = enemy.position.distance_with(other.position);

				if dist < enemy.bounds().radius {
					let dir = enemy.position - other.position;
					//desired sepration
					let wanted = dir.normalize_correct() * (enemy.bounds().radius - dir.length());

					let collective_speed = enemy.speed + other.speed;

					enemy.position += wanted; //* (collective_speed - (enemy.speed / collective_speed));
						  /*other.position -=
						  wanted * (collective_speed - (other.speed / collective_speed));*/

					//enemy.should_move_next_frame = false;
					//other.should_move_next_frame = false;
					moved = true;
				}
			});
			moved
		};

		loop {
			match self.enemies.pop() {
				None => break,
				Some(mut enemy) => {
					Self::collide_walls(&mut enemy);

					let direction = (self.player.position - enemy.position).normalize_correct();
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

	const WAVE_SPAWN_AREA: f32 = 5.0;

	fn wave_things(&mut self, delta: Duration) {
		self.wave_timer.subtract(delta);
		if self.wave_timer.is_ready() {
			self.wave_timer.reset();

			let room_dim = (Vec2::new(Game::ROOM_WIDTH, Game::ROOM_HEIGHT) / 2.0)
				- (Vec2::new(Game::WAVE_SPAWN_AREA, Game::WAVE_SPAWN_AREA) / 2.0);

			// Top-Right, Bottom-Right, Bottom-Left, Top-Left
			let mut corners = vec![
				room_dim,
				room_dim.invert(false, true),
				room_dim.invert(true, true),
				room_dim.invert(true, false),
			];

			// What corner we don't want to spawn in.
			corners.swap_remove(
				match (self.player.position.x > 0.0, self.player.position.y > 0.0) {
					(true, true) => 0,
					(true, false) => 1,
					(false, false) => 2,
					(false, true) => 3,
				},
			);

			let wave_spawn = corners[thread_rng().gen_range(0..corners.len())];

			let randoms: Vec<Enemy> = std::iter::from_fn(move || {
				Some((
					thread_rng().gen_range(0.0..Game::WAVE_SPAWN_AREA),
					thread_rng().gen_range(0.0..Game::WAVE_SPAWN_AREA),
				))
			})
			.take(3)
			.map(|position| Enemy {
				position: Vec2::from(position) + wave_spawn,
				color: Color::YELLOW,
				health: 25.0,
				speed: 0.5,
				cooldown: Cooldown::ready(Duration::from_secs(2)),
				should_move_next_frame: true,
			})
			.collect();

			self.enemies.extend(randoms);
		}
	}
}

#[derive(Debug)]
struct Player {
	position: Vec2,
	facing: Vec2,
	health: f32,
	weapon: Box<dyn Weapon>,
}

impl Colideable for Player {
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

impl Default for Player {
	fn default() -> Self {
		Self {
			position: Default::default(),
			facing: Vec2::new(0.0, 1.0),
			health: Game::PLAYER_HEALTH_MAX,
			weapon: Box::new(Pistol::default()),
		}
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

fn colide_and_move<A: Colideable, B: Colideable>(a: &A, b: &mut B) -> bool {
	let abound = a.bounds();
	let bbound = b.bounds();

	let dist = abound.position.distance_with(bbound.position);

	if dist < abound.radius {
		let dir = abound.position - bbound.position;
		//desired sepration
		let wanted = dir.normalize_correct() * (abound.radius - dir.length());

		*b.position_mut() -= wanted;
		true
	} else {
		false
	}
}

#[derive(Debug)]
struct Pistol {
	cooldown: Cooldown,
}

impl Weapon for Pistol {
	fn cooldown(&self) -> &Cooldown {
		&self.cooldown
	}

	fn cooldown_mut(&mut self) -> &mut Cooldown {
		&mut self.cooldown
	}

	fn bullets(&self, direction: Vec2) -> Vec<Bullet> {
		let direction = direction.angle() + thread_rng().gen_range(-5.0..5.0);
		println!("{direction}");

		vec![Bullet::new(
			Vec2::ZERO,
			Vec2::from_degrees(direction) * Game::BULLET_SPEED,
			10.0,
		)]
	}

	fn name(&self) -> &'static str {
		"Pistol"
	}
}

impl Default for Pistol {
	fn default() -> Self {
		Self {
			cooldown: Cooldown::ready(Duration::from_secs_f32(0.35)),
		}
	}
}

trait Weapon: core::fmt::Debug {
	fn is_ready(&self) -> bool {
		self.cooldown().is_ready()
	}

	fn cooldown(&self) -> &Cooldown;
	fn cooldown_mut(&mut self) -> &mut Cooldown;

	fn bullets(&self, direction: Vec2) -> Vec<Bullet>;

	fn name(&self) -> &'static str;
}

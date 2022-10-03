#![feature(drain_filter)]

mod things;
mod traits;
mod util;
mod weapon;

use rand::{thread_rng, Rng};
use things::{Barrel, Bullet, Enemy, Pickup};
use traits::{Colideable, Destructible, Hittable};
use util::Cooldown;
use weapon::{Ammunition, Pistol, Shotgun, Uzi, Wall, Weapon};

use std::{
	collections::VecDeque,
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
	let font = smitty.make_font("Hack-Regular.ttf");
	smitty.clear_color(Color::grey(0.5));

	let mut game = Game {
		smitten: smitty,
		player: Player::default(),
		bullets: vec![],
		enemies: vec![],
		last_render: Instant::now(),
		score: 0.0,
		score_multiplier: Multiplier::default(),
		barrels: vec![],
		barrel_count: 100,
		wave_count: 3,
		wave_timer: Cooldown::ready(Duration::from_secs_f32(10.0)),
		font,
		pickups: Game::pickup_locations()
			.into_iter()
			.map(|position| Pickup { position })
			.collect(),
		possible_pickups: vec![],
		messages: VecDeque::with_capacity(10),
		upgrades: Upgrade::upgrade_list(),
	};

	loop {
		let events = game.smitten.events();

		events.iter().for_each(|e| match e {
			SmittenEvent::MouseDown { button } => {
				game.shoot();
			}
			SmittenEvent::Keydown { key, .. } => match key {
				Some(Key::Q) => {
					game.player.decrement_weapon();
				}
				Some(Key::E) => {
					game.player.increment_weapon();
				}
				Some(Key::Row1) => {
					game.player.select_weapon(0);
				}
				Some(Key::Row2) => {
					game.player.select_weapon(1);
				}
				Some(Key::Row3) => {
					game.player.select_weapon(2);
				}
				Some(Key::Row4) => {
					game.player.select_weapon(3);
				}
				_ => (),
			},
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
	score_multiplier: Multiplier,
	score: f32,
	barrels: Vec<Barrel>,
	barrel_count: usize,
	wave_count: usize,
	wave_timer: Cooldown,
	font: FontId,
	pickups: Vec<Pickup>,
	possible_pickups: Vec<AmmoPickup>,
	messages: VecDeque<Alert>,
	upgrades: VecDeque<Upgrade>,
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

		for pickup in &self.pickups {
			self.rect(pickup.position, Game::PLAYER_DIM / 2.0, Color::RED);
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

	const WAVE_TIMER_HEIGHT: f32 = 0.5;

	fn draw_ui(&self) {
		let ghost = Color::rgba(0.0, 0.0, 0.0, 0.3);

		// Score
		self.smitten.write(
			self.font,
			&format!("{}", self.score),
			(HorizontalAnchor::Center(0.0), VerticalAnchor::Top(-0.75)),
			Color::BLACK,
			1.0,
		);

		// Multiplier
		let width = 3.0 * (1.0 - self.score_multiplier.percent());
		self.smitten.anchored_rect(
			(HorizontalAnchor::Center(0.0), VerticalAnchor::Top(-2.25)),
			(3.0, 0.1),
			Color::BLACK,
		);

		self.smitten.anchored_rect(
			(
				HorizontalAnchor::Center(width - 1.5),
				VerticalAnchor::Top(-2.25 + 0.175),
			),
			(0.175, 0.5),
			Color::BLACK,
		);

		self.smitten.write(
			self.font,
			format!("x{}", self.score_multiplier.current as usize),
			(HorizontalAnchor::Center(-1.25), VerticalAnchor::Top(-2.75)),
			Color::BLACK,
			0.5,
		);

		// Wave timer
		self.smitten.anchored_rect(
			(HorizontalAnchor::Left(0.0), VerticalAnchor::Top(0.0)),
			(DIM.0 as f32 / MUR as f32, Game::WAVE_TIMER_HEIGHT),
			ghost,
		);

		self.smitten.anchored_rect(
			(HorizontalAnchor::Left(0.0), VerticalAnchor::Top(0.0)),
			(
				(DIM.0 as f32 / MUR as f32) * (1.0 - self.wave_timer.percent()),
				Game::WAVE_TIMER_HEIGHT,
			),
			Color::BLUE,
		);

		// Message box
		if !self.messages.is_empty() {
			self.smitten.anchored_rect(
				(
					HorizontalAnchor::Right(0.0),
					VerticalAnchor::Top(-Game::WAVE_TIMER_HEIGHT),
				),
				(5.0, self.messages.len() as f32 / 1.5 + 0.25),
				ghost,
			);
			self.write_messages();
		}

		// Weapon details
		self.smitten.write(
			self.font,
			self.player.weapon().name(),
			(0.0, VerticalAnchor::Bottom(1.0)),
			Color::BLACK,
			0.5,
		);

		if let Ammunition::Limited { rounds, .. } = self.player.weapon().ammo() {
			self.smitten.write(
				self.font,
				format!("{}", rounds),
				(0.0, VerticalAnchor::Center(-1.0)),
				Color::BLACK,
				0.5,
			);
		}

		// Health
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

	fn write_messages(&self) {
		for (idx, msg) in self.messages.iter().enumerate() {
			self.smitten.write(
				self.font,
				&msg.message,
				(
					HorizontalAnchor::Right(0.0),
					VerticalAnchor::Top(-Game::WAVE_TIMER_HEIGHT - idx as f32 / 1.5),
				),
				msg.color(),
				0.5,
			)
		}
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
		self.player.tick(delta);
		self.check_pickups();

		let hits = Self::do_bullet_hits(&mut self.enemies, &mut self.bullets);
		Self::burry_dead(&mut self.enemies)
			.into_iter()
			.for_each(|e| self.enemy_killed(e));
		self.tick_enemies(delta);

		let barrel_hits = Self::do_bullet_hits(&mut self.barrels, &mut self.bullets);
		Self::burry_dead(&mut self.barrels);

		// Messages
		self.messages.retain_mut(|a| {
			a.lifetime.subtract(delta);
			!a.lifetime.is_ready()
		});
		self.score_multiplier.subtract(delta);
	}

	fn enemy_killed(&mut self, e: Enemy) {
		self.score += 100.0 * self.score_multiplier.current;
		self.score_multiplier.increment();

		let mut todo = vec![];
		loop {
			match self.upgrades.front() {
				None => return,
				Some(up) => {
					if up.score <= self.score {
						todo.push(self.upgrades.pop_front().unwrap());
					} else {
						break;
					}
				}
			}
		}

		for upgrade in todo {
			self.push_alert(Alert::with_color(format!("{}", upgrade.kind), Color::GREEN));

			macro_rules! cut_cooldown {
				($index:literal) => {
					self.player.weapons[$index].cooldown_mut().cooldown /= 2
				};
			}

			macro_rules! double_ammo {
				($index:literal) => {
					self.player.weapons[$index].ammo_mut().scale_magazine(2.0)
				};
			}

			macro_rules! double_damage {
				($index:literal) => {
					*self.player.weapons[$index].damage_mut() *= 2.0
				};
			}

			macro_rules! unlock {
				($index:literal $pickup:path) => {{
					self.player.weapons[$index].ammo_mut().reload();
					self.possible_pickups.push($pickup);
				}};
			}

			match upgrade.kind {
				UpgradeType::PistolFast => cut_cooldown!(0),
				UpgradeType::UziUnlock => unlock!(1 AmmoPickup::Uzi),
				UpgradeType::PistolDouble => double_damage!(0),
				UpgradeType::ShotgunUnlock => unlock!(2 AmmoPickup::Shotgun),
				UpgradeType::UziFast => cut_cooldown!(1),
				UpgradeType::WallUnlock => unlock!(3 AmmoPickup::Wall),
				UpgradeType::UziDoubleAmmo => double_ammo!(1),
				UpgradeType::ShotgunFast => cut_cooldown!(2),
			}
		}
	}

	pub fn shoot(&mut self) {
		if !self.player.weapon().can_fire() {
			return;
		}
		self.player.weapon_mut().cooldown_mut().reset();

		if self.player.selected_weapon != 3 {
			for mut bull in self.player.weapon().bullets(self.player.facing) {
				bull.position = self.player.position;

				self.bullets.push(bull);
			}
		} else {
			self.place_barrel();
		}

		self.player.weapon_mut().ammo_mut().decrement();

		// Switch back to the pistol if the current weapon just ran out of ammo
		if self.player.weapon().ammo().is_empty() {
			self.player.select_weapon(0);
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
	const WALL_WIDTH: f32 = 50.0;

	fn draw_walls(&self) {
		let room_width = Self::ROOM_WIDTH;
		let room_height = Self::ROOM_HEIGHT;

		let hww = Game::WALL_WIDTH / 2.0;
		let hrw = (room_width / 2.0) + hww - 0.5;
		let hrh = (room_height / 2.0) + hww - 0.5;

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
			self.rect(pos, dim, Color::grey(0.5))
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
				let pixel_gap = (MUR as f32 - 2.0) / MUR as f32;
				self.smitten.rect(
					pos,
					Vec2::new(pixel_gap, pixel_gap),
					Color::rgb(0.98, 0.98, 0.88),
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

			let mut quarter_center = room_dim / 2.0;
			quarter_center -= quarter_center / 2.0;

			// Top-Right, Bottom-Right, Bottom-Left, Top-Left, Center
			let mut corners = vec![
				room_dim,
				room_dim.invert(false, true),
				room_dim.invert(true, true),
				room_dim.invert(true, false),
				Vec2::new(-Game::WAVE_SPAWN_AREA / 2.0, -Game::WAVE_SPAWN_AREA / 2.0),
			];

			let in_center = self.player.position.x < quarter_center.x
				&& self.player.position.x > -quarter_center.x
				&& self.player.position.y < quarter_center.y
				&& self.player.position.y > -quarter_center.y;

			// What corner we don't want to spawn in.
			corners.swap_remove(
				match (
					self.player.position.x > 0.0,
					self.player.position.y > 0.0,
					in_center,
				) {
					(_, _, true) => 4,
					(true, true, false) => 0,
					(true, false, false) => 1,
					(false, false, false) => 2,
					(false, true, false) => 3,
				},
			);

			let randoms: Vec<Enemy> = std::iter::from_fn(move || {
				let corner = corners[thread_rng().gen_range(0..corners.len())];
				Some(
					corner
						+ Vec2::new(
							thread_rng().gen_range(0.0..Game::WAVE_SPAWN_AREA),
							thread_rng().gen_range(0.0..Game::WAVE_SPAWN_AREA),
						),
				)
			})
			.take(self.wave_count)
			.map(|position| Enemy {
				position,
				color: Color::YELLOW,
				health: 25.0,
				speed: 0.75,
				cooldown: Cooldown::ready(Duration::from_secs(2)),
				should_move_next_frame: true,
			})
			.collect();

			self.enemies.extend(randoms);
			self.wave_count += 1;
		}
	}

	const PICKUP_SPACING: f32 = 5.0;

	fn pickup_locations() -> Vec<Vec2> {
		let mut ret = vec![];

		let x_pickups = ((Game::ROOM_WIDTH - (Game::PICKUP_SPACING * 2.0)) / Game::PICKUP_SPACING)
			.floor() as u32
			- 2;
		let y_pickups = ((Game::ROOM_HEIGHT - (Game::PICKUP_SPACING * 2.0)) / Game::PICKUP_SPACING)
			.floor() as u32
			- 2;

		println!("pickup count: {x_pickups}x{y_pickups}");

		for x in 0..x_pickups {
			for y in 0..y_pickups {
				ret.push(Vec2::new(
					(-Game::ROOM_WIDTH / 2.0)
						+ (x as f32 * Game::PICKUP_SPACING + Game::PICKUP_SPACING * 2.5),
					(-Game::ROOM_HEIGHT / 2.0)
						+ (y as f32 * Game::PICKUP_SPACING + Game::PICKUP_SPACING * 2.5),
				))
			}
		}

		ret
	}

	fn check_pickups(&mut self) {
		let mut checked = vec![];

		let unchecked: Vec<Pickup> = self.pickups.drain(..).collect();
		for pickup in unchecked {
			if pickup.colides_with(&self.player) {
				if self.possible_pickups.len() > 0 {
					let r: usize = thread_rng().gen_range(0..self.possible_pickups.len());
					let pickup = self.possible_pickups[r];
					self.player.pickedup(pickup);
					self.push_alert(Alert::new(format!("{}", pickup)));
				}
			} else {
				checked.push(pickup);
			}
		}

		self.pickups.extend(checked);
	}

	fn push_alert(&mut self, alert: Alert) {
		self.messages.push_back(alert);

		if self.messages.len() > 10 {
			self.messages.pop_front();
		}
	}
}

#[derive(Debug)]
struct Player {
	position: Vec2,
	facing: Vec2,
	health: f32,
	weapons: Vec<Box<dyn Weapon>>,
	selected_weapon: usize,
}

impl Player {
	pub fn weapon(&self) -> &Box<dyn Weapon> {
		&self.weapons[self.selected_weapon]
	}

	pub fn weapon_mut(&mut self) -> &mut Box<dyn Weapon> {
		&mut self.weapons[self.selected_weapon]
	}

	pub fn tick(&mut self, delta: Duration) {
		self.weapon_mut().cooldown_mut().subtract(delta);
	}

	/// Returns a bool indicating if the indexed weapon could be selected
	pub fn select_weapon(&mut self, index: usize) -> bool {
		println!("selecting {index}");
		if index >= self.weapons.len() {
			false
		} else {
			if self.weapons[index].ammo().is_empty() {
				println!("Cannot select a weapon with no ammo");
				return false;
			}

			self.selected_weapon = index;
			true
		}
	}

	/// Does not roll over
	pub fn decrement_weapon(&mut self) -> bool {
		if self.selected_weapon != 0 {
			self.select_weapon(self.selected_weapon - 1)
		} else {
			false
		}
	}

	/// Does not roll over
	pub fn increment_weapon(&mut self) -> bool {
		if self.selected_weapon == self.weapons.len() - 1 {
			false
		} else {
			self.select_weapon(self.selected_weapon + 1)
		}
	}

	pub fn pickedup(&mut self, pickedup: AmmoPickup) {
		let weapon_index = match pickedup {
			AmmoPickup::Uzi => 1,
			AmmoPickup::Shotgun => 2,
			AmmoPickup::Wall => 3,
		};

		self.weapons[weapon_index].ammo_mut().reload();
	}
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
			weapons: vec![
				Box::new(Pistol::default()),
				Box::new(Uzi::default()),
				Box::new(Shotgun::default()),
				Box::new(Wall::default()),
			],
			selected_weapon: 0,
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

#[derive(Copy, Clone, Debug, PartialEq)]
enum AmmoPickup {
	Uzi,
	Shotgun,
	Wall,
}

#[derive(Clone, Debug)]
struct Alert {
	message: String,
	lifetime: Cooldown,
	color: Color,
}

impl Alert {
	pub fn new(message: String) -> Self {
		Self::with_color(message, Color::WHITE)
	}

	pub fn with_color(message: String, color: Color) -> Alert {
		Self {
			message,
			lifetime: Cooldown::waiting(Duration::from_secs(3)),
			color,
		}
	}

	pub fn color(&self) -> Color {
		let a = self.lifetime.percent();
		// I'm sure this is mathable I just don't know how to do it
		let a = if a > 0.66 { (1.0 - a) * 3.0 } else { 1.0 };

		let mut c = self.color;
		let a = lerp(c.a, 0.0, 1.0 - a);
		c.a = a;
		c
	}
}

impl std::fmt::Display for AmmoPickup {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let stat = match self {
			AmmoPickup::Uzi => "uzi ammo",
			AmmoPickup::Shotgun => "shotgun ammo",
			AmmoPickup::Wall => "wall ammo",
		};

		write!(f, "{}", stat)
	}
}

struct Upgrade {
	score: f32,
	kind: UpgradeType,
}

impl Upgrade {
	pub fn new(score: f32, kind: UpgradeType) -> Upgrade {
		Self { score, kind }
	}

	pub fn upgrade_list() -> VecDeque<Upgrade> {
		let mut ret = VecDeque::new();

		macro_rules! upgrade {
			($score:literal $kind:path) => {
				ret.push_back(Upgrade::new($score as f32, $kind));
			};

			($score:literal $kind:path, $score2:literal $kind2:path) => {
				upgrade!($score $kind);
				upgrade!($score2 $kind2);
			};

			($score:literal $kind:path, $score2:literal $kind2:path, $($scores:literal $kinds:path),+) => {
				upgrade!($score $kind);
				upgrade!($score2 $kind2, $($scores $kinds),+);
			}
		}

		upgrade!(
			300 UpgradeType::PistolFast,
			900 UpgradeType::UziUnlock,
			1700 UpgradeType::PistolDouble,
			2600 UpgradeType::ShotgunUnlock,
			3500 UpgradeType::UziFast,
			4100 UpgradeType::WallUnlock,
			4700 UpgradeType::UziDoubleAmmo,
			5200 UpgradeType::ShotgunFast
		);

		ret
	}
}

enum UpgradeType {
	PistolFast,
	UziUnlock,
	PistolDouble,
	ShotgunUnlock,
	UziFast,
	WallUnlock,
	UziDoubleAmmo,
	ShotgunFast,
}

impl std::fmt::Display for UpgradeType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let stat = match self {
			UpgradeType::PistolFast => "pistol fast fire",
			UpgradeType::UziUnlock => "uzi unlocked",
			UpgradeType::PistolDouble => "pistol double damge",
			UpgradeType::ShotgunUnlock => "shotgun unlocked",
			UpgradeType::UziFast => "uzi rapid fire",
			UpgradeType::WallUnlock => "wallls unlocked",
			UpgradeType::UziDoubleAmmo => "uzi double ammo",
			UpgradeType::ShotgunFast => "shotgun fast fire",
		};

		write!(f, "{}", stat)
	}
}

struct Multiplier {
	current: f32,
	cooldown: Cooldown,
}

impl Multiplier {
	pub fn subtract(&mut self, delta: Duration) {
		self.cooldown.subtract(delta);

		if self.cooldown.is_ready() {
			self.decrement();
		}
	}

	pub fn percent(&self) -> f32 {
		if self.current > 1.0 {
			self.cooldown.percent()
		} else {
			0.0
		}
	}

	pub fn decrement(&mut self) {
		if self.current == 1.0 {
			return;
		}

		self.current -= 1.0;
		self.set_cooldown();
	}

	fn set_cooldown(&mut self) {
		self.cooldown = Cooldown::waiting(Duration::from_millis(2000 - (self.current as u64 * 66)));
	}

	pub fn increment(&mut self) {
		if self.current == 30.0 {
			self.cooldown.reset();
			return;
		}

		self.current += 1.0;
		self.set_cooldown();
	}
}

impl Default for Multiplier {
	fn default() -> Self {
		Self {
			current: 1.0,
			cooldown: Cooldown::ready(Duration::from_secs(2)),
		}
	}
}

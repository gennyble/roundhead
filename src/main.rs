use smitten::{Color, Draw, Key, Smitten, Vec2};

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
	};

	loop {
		let events = game.smitten.events();

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

		// Draw
		game.smitten.clear();
		game.draw((0, 0), (1, 1), TURQUOISE);
		game.smitten.swap();
	}
}

// A higher level struct so I can keep a camera et al.
struct Game {
	smitten: Smitten,
	camera: Vec2,
}

impl Game {
	pub fn draw<P: Into<Vec2>, D: Into<Vec2>, R: Into<Draw>>(&self, pos: P, dim: D, draw: R) {
		self.smitten.rect(self.camera + pos.into(), dim, draw)
	}
}

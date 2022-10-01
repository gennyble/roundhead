use std::time::Duration;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Cooldown {
	cooldown: Duration,
	waiting: Duration,
	state: State,
}

impl Cooldown {
	pub fn ready(cooldown: Duration) -> Self {
		Self {
			cooldown,
			waiting: Duration::ZERO,
			state: State::Ready,
		}
	}

	pub fn is_ready(&self) -> bool {
		match self.state {
			State::Ready => true,
			State::Waiting => false,
		}
	}

	pub fn reset(&mut self) {
		self.waiting = self.cooldown;
		self.state = State::Waiting;
	}

	pub fn subtract(&mut self, delta: Duration) {
		match self.state {
			State::Waiting => {
				if delta >= self.waiting {
					self.waiting = Duration::ZERO;
					self.state = State::Ready;
				} else {
					self.waiting -= delta;
				}
			}
			State::Ready => (),
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
	Waiting,
	Ready,
}

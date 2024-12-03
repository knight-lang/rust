use std::io;

use crate::options::Options;
use crate::value::{Integer, KString};
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct Environment {
	opts: Options,
	rng: StdRng,
}

impl Default for Environment {
	fn default() -> Self {
		Self::new(Options::default())
	}
}

impl Environment {
	pub fn new(opts: Options) -> Self {
		// TODO: allow `rng` to be supplied by callers
		Self { opts, rng: StdRng::from_entropy() }
	}

	pub fn opts(&self) -> &Options {
		&self.opts
	}

	pub fn prompt(&mut self) -> crate::Result<Option<KString>> {
		let mut line = String::new();
		let amnt = std::io::stdin()
			.read_line(&mut line)
			.map_err(|err| crate::Error::IoError { func: "PROMPT", err })?;

		if amnt == 0 {
			return Ok(None);
		}

		if line.chars().last().map_or(false, |c| c == '\n') {
			line.pop();
		}

		if cfg!(feature = "knight_2_0_1") {
			while line.chars().last().map_or(false, |c| c == '\r') {
				line.pop();
			}
		} else {
			if line.chars().last().map_or(false, |c| c == '\r') {
				line.pop();
			}
		}

		Ok(Some(KString::new(line, &self.opts)?))
	}

	pub fn output(&mut self) -> impl io::Write {
		// TODO: eventually allow for capturing output within Knight programs
		std::io::stdout()
	}

	pub fn quit(&mut self, status: i32) -> crate::Result<std::convert::Infallible> {
		#[cfg(feature = "compliance")]
		if self.opts.compliance.check_quit_status_codes && !(0..=127).contains(&status) {
			// TODO: Mauybe have a custom error for this?
			return Err(
				crate::value::integer::IntegerError::DomainError("QUIT: not in bounds").into(),
			);
		}

		#[cfg(feature = "embedded")]
		if self.opts.embedded.dont_exit_when_quitting {
			return Err(crate::Error::Exit(status));
		}

		std::process::exit(status);
	}

	pub fn random(&mut self) -> crate::Result<Integer> {
		let min = match () {
			#[cfg(feature = "extensions")]
			_ if self.opts.extensions.breaking.random_can_be_negative => Integer::min(&self.opts).inner(),
			_ => 0,
		};

		let max = match () {
			#[cfg(feature = "compliance")]
			_ if self.opts.compliance.limit_rand_range => 0x7FFF,
			_ => Integer::max(&self.opts).inner(),
		};

		// We can do `new_unvalidated` as we clamp the min/max based on compliance.
		Ok(Integer::new_unvalidated_unchecked(self.rng.gen_range(min..=max)))
	}
}

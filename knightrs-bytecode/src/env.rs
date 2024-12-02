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
		Self { opts: Options::default(), rng: StdRng::from_entropy() }
	}
}

impl Environment {
	pub fn opts(&self) -> &Options {
		&self.opts
	}

	pub fn prompt(&mut self) -> crate::Result<Option<KString>> {
		todo!()
	}

	pub fn output(&mut self) -> impl io::Write {
		// TODO: eventually allow for capturing output within Knight programs
		std::io::stdout()
	}

	pub fn quit(&mut self, status: i32) -> crate::Result<std::convert::Infallible> {
		#[cfg(feature = "compliance")]
		if self.opts.compliance.check_quit_status_codes && !(0..=127).contains(&status) {
			// TODO: Mauybe have a custom error for this?
			return Err(crate::value::integer::IntegerError::Overflow('Q').into());
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
		Ok(Integer::new_unvalidated(self.rng.gen_range(min..=max)))
	}
}

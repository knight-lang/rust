use crate::containers::MaybeSendSync;
use std::io::{self, Write};

#[cfg(feature = "extensions")]
use crate::{env::Variable, value::Text};

pub trait Stdout: Write + MaybeSendSync {}
impl<T: Write + MaybeSendSync> Stdout for T {}

pub struct Output<'e> {
	default: Box<dyn Stdout + 'e>,

	#[cfg(feature = "extensions")]
	pipe: Option<Variable<'e>>,
}

impl Default for Output<'_> {
	fn default() -> Self {
		Self {
			default: Box::new(io::stdout()),

			#[cfg(feature = "extensions")]
			pipe: None,
		}
	}
}

impl<'e> Output<'e> {
	/// Sets the default stdout.
	///
	/// This doesn't affect any pipes which are enabled.
	pub fn set_stdout<S: Stdout + 'e>(&mut self, stdout: S) {
		self.default = Box::new(stdout);
	}

	#[cfg(feature = "extensions")]
	pub fn set_pipe(&mut self, variable: Variable<'e>) {
		self.pipe = Some(variable)
	}
}

impl Write for Output<'_> {
	fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
		#[cfg(feature = "extensions")]
		if let Some(pipe) = self.pipe.as_ref() {
			// The error case shouldn't happen if we call `write` from within Knight.
			let _ = pipe;
			let _: Text = todo!();
			// let text = String::from_utf8(bytes.to_vec())
			// 	.ok()
			// 	.and_then(|s| Text::try_from(s).ok())
			// 	.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "not utf8".to_string()))?;

			// pipe.assign(text.into());
			// return Ok(bytes.len());
		}

		self.default.write(bytes)
	}

	fn flush(&mut self) -> io::Result<()> {
		#[cfg(feature = "extensions")]
		if self.pipe.is_some() {
			return Ok(());
		}

		self.default.flush()
	}
}

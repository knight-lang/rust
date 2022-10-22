//! How Knight writes to stdout.

use super::Flags;
use crate::containers::MaybeSendSync;
use std::io::{self, Write};

/// A trait used for writing to stdout.
///
/// This exists instead of simply using [`Write`] because we only need `Send + Sync` when the
/// `multithreaded` feature is enabled, but we want a uniform interface.
pub trait Stdout: Write + MaybeSendSync {}
impl<T: Write + MaybeSendSync> Stdout for T {}

/// The type that's in charge of writing text to stdout.
///
/// # Redirection
/// If `extensions` is enabled, you can redirect anything written to stdout to a variable via
/// [`Output::set_redirect`]. It'll convert whatever's currently in the variable to a [`Text`](
/// crate::value::Text), and then append the stuff that's being written to the end. This will catch
/// both `DUMP` and `OUTPUT`'s output.
///
/// ```knight
/// ; = OUTPUT BLOCK out # redirect output to `out`
/// ; OUTPUT "hello"
/// ; OUTPUT "world\"
/// ; = OUTPUT NULL # return back to normal output
/// ; DUMP out #=> "hello\nworld"
/// ```
pub struct Output<'e> {
	default: Box<dyn Stdout + 'e>,

	#[cfg_attr(not(feature = "extensions"), allow(dead_code))]
	flags: &'e Flags,

	#[cfg(feature = "extensions")]
	redirect: Option<super::Variable>,
}

impl<'e> Output<'e> {
	pub(super) fn new(flags: &'e Flags) -> Self {
		Self {
			default: Box::new(io::stdout()),
			flags,

			#[cfg(feature = "extensions")]
			redirect: None,
		}
	}
	/// Sets the default stdout.
	///
	/// This doesn't affect any pipes which are enabled.
	pub fn set_stdout<S: Stdout + 'e>(&mut self, stdout: S) {
		self.default = Box::new(stdout);
	}

	/// Sets where stdout will be redirected to.
	#[cfg(feature = "extensions")]
	pub fn set_redirection(&mut self, variable: super::Variable) {
		self.redirect = Some(variable)
	}

	/// Sets where stdout will be assigned.
	#[cfg(feature = "extensions")]
	pub fn clear_redirection(&mut self) {
		self.redirect = None;
	}
}

impl Write for Output<'_> {
	fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
		#[cfg(feature = "extensions")]
		if let Some(redirect) = self.redirect.as_ref() {
			// The error case shouldn't happen if we call `write` from within Knight.
			let text = String::from_utf8(bytes.to_vec())
				.map_err(|e| e.to_string())
				.and_then(|s| crate::value::Text::new(s, self.flags).map_err(|e| e.to_string()))
				.or_else(|err| Err(io::Error::new(io::ErrorKind::InvalidData, err)))?;

			redirect.assign(text.into());
			return Ok(bytes.len());
		}

		self.default.write(bytes)
	}

	fn flush(&mut self) -> io::Result<()> {
		#[cfg(feature = "extensions")]
		if self.redirect.is_some() {
			return Ok(());
		}

		self.default.flush()
	}
}

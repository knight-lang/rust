use std::cmp::Ordering;

use crate::{program::JumpIndex, vm::Vm, Environment, Error, Result};

pub mod block;
pub mod boolean;
pub mod integer;
pub mod list;
pub mod null;
pub mod string;
pub use block::Block;
pub use boolean::{Boolean, ToBoolean};
use integer::IntegerError;
pub use integer::{Integer, ToInteger};
pub use list::{List, ToList};
pub use null::Null;
pub use string::{KString, ToKString};

/// A trait indicating a type has a name.
pub trait NamedType {
	/// The name of a type.
	fn type_name(&self) -> &'static str;
}

// Todo: more
#[derive(Default, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Value {
	#[default]
	Null,
	Boolean(Boolean),
	Integer(Integer),
	String(KString),
	List(List),
	Block(Block),
}

impl From<Boolean> for Value {
	fn from(b: Boolean) -> Self {
		Self::Boolean(b)
	}
}

impl From<Null> for Value {
	fn from(_: Null) -> Self {
		Self::Null
	}
}

impl From<Integer> for Value {
	fn from(integer: Integer) -> Self {
		Self::Integer(integer)
	}
}

impl From<KString> for Value {
	fn from(string: KString) -> Self {
		Self::String(string)
	}
}

impl From<List> for Value {
	fn from(list: List) -> Self {
		Self::List(list)
	}
}

impl From<Block> for Value {
	fn from(block: Block) -> Self {
		Self::Block(block)
	}
}

impl ToBoolean for Value {
	fn to_boolean(&self, env: &mut Environment) -> Result<Boolean> {
		match self {
			Self::Null => Null.to_boolean(env),
			Self::Boolean(boolean) => boolean.to_boolean(env),
			Self::Integer(integer) => integer.to_boolean(env),
			Self::String(string) => string.to_boolean(env),
			Self::List(list) => list.to_boolean(env),
			_ => Err(Error::ConversionNotDefined {
				to: Boolean::default().type_name(),
				from: self.type_name(),
			}),
		}
	}
}

impl ToInteger for Value {
	fn to_integer(&self, env: &mut Environment) -> Result<Integer> {
		match self {
			Self::Null => Null.to_integer(env),
			Self::Boolean(boolean) => boolean.to_integer(env),
			Self::Integer(integer) => integer.to_integer(env),
			Self::String(string) => string.to_integer(env),
			Self::List(list) => list.to_integer(env),
			_ => Err(Error::ConversionNotDefined {
				to: Integer::default().type_name(),
				from: self.type_name(),
			}),
		}
	}
}

impl ToKString for Value {
	fn to_kstring(&self, env: &mut Environment) -> Result<KString> {
		match self {
			Self::Null => Null.to_kstring(env),
			Self::Boolean(boolean) => boolean.to_kstring(env),
			Self::Integer(integer) => integer.to_kstring(env),
			Self::String(string) => string.to_kstring(env),
			Self::List(list) => list.to_kstring(env),
			_ => Err(Error::ConversionNotDefined {
				to: KString::default().type_name(),
				from: self.type_name(),
			}),
		}
	}
}

impl ToList for Value {
	fn to_list(&self, env: &mut Environment) -> Result<List> {
		match self {
			Self::Null => Null.to_list(env),
			Self::Boolean(boolean) => boolean.to_list(env),
			Self::Integer(integer) => integer.to_list(env),
			Self::String(string) => string.to_list(env),
			Self::List(list) => list.to_list(env),
			_ => Err(Error::ConversionNotDefined {
				to: List::default().type_name(),
				from: self.type_name(),
			}),
		}
	}
}

impl NamedType for Value {
	/// Fetch the type's name.
	#[must_use = "getting the type name by itself does nothing."]
	fn type_name(&self) -> &'static str {
		match self {
			Self::Null => Null.type_name(),
			Self::Boolean(boolean) => boolean.type_name(),
			Self::Integer(integer) => integer.type_name(),
			Self::String(string) => string.type_name(),
			Self::List(list) => list.type_name(),
			Self::Block(list) => "Block",
			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.type_name(),
		}
	}
}

impl Value {
	pub fn kn_dump(&self, env: &mut Environment) -> Result<()> {
		use std::io::Write;

		// TODO: move this into each type, so they can control it
		match self {
			Self::Null => {
				write!(env.output(), "null").map_err(|err| Error::IoError { func: "OUTPUT", err })
			}
			Self::Boolean(b) => {
				write!(env.output(), "{b}").map_err(|err| Error::IoError { func: "OUTPUT", err })
			}
			Self::Integer(i) => {
				write!(env.output(), "{i}").map_err(|err| Error::IoError { func: "OUTPUT", err })
			}
			Self::String(s) => write!(env.output(), "{:?}", s.as_str())
				.map_err(|err| Error::IoError { func: "OUTPUT", err }),
			Self::List(l) => {
				write!(env.output(), "[").map_err(|err| Error::IoError { func: "OUTPUT", err })?;
				for (idx, arg) in l.iter().enumerate() {
					if idx != 0 {
						write!(env.output(), ", ")
							.map_err(|err| Error::IoError { func: "OUTPUT", err })?;
					}
					arg.kn_dump(env)?;
				}
				write!(env.output(), "]").map_err(|err| Error::IoError { func: "OUTPUT", err })
			}
			#[cfg(feature = "compliance")]
			Self::Block(b) if env.opts().compliance.cant_dump_blocks => {
				Err(Error::TypeError { type_name: self.type_name(), function: "DUMP" })
			}

			Self::Block(b) => {
				write!(env.output(), "{b:?}").map_err(|err| Error::IoError { func: "OUTPUT", err })
			}
		}
	}

	/// Compares to arguments, knight-style. Coerces them too.
	pub fn kn_compare(
		&self,
		rhs: &Self,
		fn_name: &'static str,
		env: &mut Environment,
	) -> Result<Ordering> {
		match self {
			Self::Integer(lhs) => Ok(lhs.cmp(&rhs.to_integer(env)?)),
			Self::Boolean(lhs) => Ok(lhs.cmp(&rhs.to_boolean(env)?)),
			Self::String(lhs) => Ok(lhs.cmp(&rhs.to_kstring(env)?)),
			Self::List(lhs) => {
				let rhs = rhs.to_list(env)?;

				for (left, right) in lhs.iter().zip(&rhs) {
					match left.kn_compare(right, fn_name, env)? {
						Ordering::Equal => continue,
						other => return Ok(other),
					}
				}

				Ok(lhs.len().cmp(&rhs.len()))
			}

			other => Err(Error::TypeError { type_name: self.type_name(), function: fn_name }),
		}
	}

	/// Checks to see if two arguments are equal.
	///
	/// When `compliance.check_equals_params` is enabled, this can return an error if either argument
	/// is a block, or a list containing a block. Without `compliance.check_equals_params`, this
	/// never fails.
	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn kn_equals(&self, rhs: &Self, env: &mut Environment) -> Result<bool> {
		// Rust's `==` semantics here actually directly map on to how equality in Knight works.

		// In strict compliance mode, we can't use Blocks for `?`.
		#[cfg(feature = "compliance")]
		{
			fn forbid_block_params_in_is_equal(value: &Value) -> Result<()> {
				match value {
					Value::List(list) => {
						for ele in list {
							forbid_block_params_in_is_equal(ele)?;
						}
						Ok(())
					}
					Value::Block(_) => {
						// todo: better error message?
						Err(Error::TypeError { type_name: value.type_name(), function: "?" })
					}
					_ => Ok(()),
				}
			}

			if env.opts().compliance.check_equals_params {
				forbid_block_params_in_is_equal(self)?;
				forbid_block_params_in_is_equal(rhs)?;
			}
		}

		let _ = env;
		Ok(self == rhs)
	}

	pub fn kn_call(&self, vm: &mut Vm) -> Result<Value> {
		match self {
			Self::Block(block) => vm.run(*block),
			other => Err(Error::TypeError { type_name: other.type_name(), function: "CALL" }),
		}
	}

	pub fn kn_length(&self, env: &mut Environment) -> Result<Integer> {
		cfg_if! {
			if #[cfg(feature = "knight_2_0_1")] {
				let length_of_anything = true;
			} else if #[cfg(feature = "extensions")] {
				let length_of_anything = env.opts().extensions.builtin_fns.length_of_anything;
			} else {
				let length_of_anything = false;
			}
		};

		match self {
			Self::String(string) => {
				// Rust guarantees that `str::len` won't be larger than `isize::MAX`. Since we're always
				// using `i64`, if `usize == u32` or `usize == u64`, we can always cast the `isize` to
				// the `i64` without failure.
				//
				// With compliance enabled, it's possible that we are only checking for compliance on
				// integer bounds, and not on string lengths, so we do have to check in compliance mode.
				#[cfg(feature = "compliance")]
				if env.opts().compliance.i32_integer && !env.opts().compliance.check_container_length {
					return Ok(Integer::new_error(string.len() as i64, env.opts())?);
				}

				Ok(Integer::new_unvalidated(string.len() as i64))
			}

			Self::List(list) => {
				// (same guarantees as `Self::String`)
				#[cfg(feature = "compliance")]
				if env.opts().compliance.i32_integer && !env.opts().compliance.check_container_length {
					return Ok(Integer::new_error(list.len() as i64, env.opts())?);
				}

				Ok(Integer::new_unvalidated(list.len() as i64))
			}

			#[cfg(any(feature = "knight_2_0_1", feature = "extensions"))]
			Self::Integer(int) if length_of_anything => {
				Ok(Integer::new_unvalidated(int.number_of_digits() as _))
			}

			#[cfg(any(feature = "knight_2_0_1", feature = "extensions"))]
			Self::Boolean(true) if length_of_anything => Ok(Integer::new_unvalidated(1)),

			#[cfg(any(feature = "knight_2_0_1", feature = "extensions"))]
			Self::Boolean(false) | Self::Null if length_of_anything => Ok(Integer::new_unvalidated(0)),

			other => Err(Error::TypeError { type_name: other.type_name(), function: "LENGTH" }),
		}
	}

	pub fn kn_negate(&self, env: &mut Environment) -> Result<Integer> {
		#[cfg(feature = "extensions")]
		if env.opts().extensions.breaking.negate_reverses_collections {
			todo!();
		}

		Ok(self.to_integer(env)?.negate(env.opts())?)
	}

	pub fn kn_plus(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => Ok(integer.add(rhs.to_integer(env)?, env.opts())?.into()),
			Self::String(string) => Ok(string.concat(&rhs.to_kstring(env)?, env.opts())?.into()),
			Self::List(list) => list.concat(&rhs.to_list(env)?, env.opts()).map(Self::from),
			#[cfg(feature = "extensions")]
			Self::Boolean(lhs) if env.opts().extensions.builtin_fns.boolean => {
				Ok((lhs | rhs.to_boolean(env)?).into())
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.add(rhs, env),

			other => Err(Error::TypeError { type_name: other.type_name(), function: "+" }),
		}
	}

	pub fn kn_minus(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => Ok(integer.subtract(rhs.to_integer(env)?, env.opts())?.into()),

			#[cfg(feature = "extensions")]
			Self::String(string) if env.opts().extensions.builtin_fns.string => {
				Ok(string.remove_substr(&rhs.to_kstring(env)?).into())
			}

			#[cfg(feature = "extensions")]
			Self::List(list) if env.opts().extensions.builtin_fns.list => {
				list.difference(&rhs.to_list(env)?).map(Self::from)
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.subtract(rhs, env),

			other => Err(Error::TypeError { type_name: other.type_name(), function: "-" }),
		}
	}

	pub fn kn_asterisk(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => Ok(integer.multiply(rhs.to_integer(env)?, env.opts())?.into()),

			Self::String(lstr) => {
				let amount = usize::try_from(rhs.to_integer(env)?.inner())
					.or(Err(IntegerError::DomainError("repetition count is negative")))?;

				if amount.checked_mul(lstr.len()).map_or(true, |c| isize::MAX as usize <= c) {
					return Err(IntegerError::DomainError("repetition is too large").into());
				}

				Ok(lstr.repeat(amount, env.opts())?.into())
			}

			Self::List(list) => {
				let rhs = rhs;

				// Multiplying by a block is invalid, so we can do this as an extension.
				#[cfg(any())]
				#[cfg(feature = "extensions")]
				if env.opts().extensions.builtin_fns.list && matches!(rhs, Self::Ast(_)) {
					return list.map(rhs, env).map(Self::from);
				}

				let amount = usize::try_from(rhs.to_integer(env)?.inner())
					.or(Err(IntegerError::DomainError("repetition count is negative")))?;

				// No need to check for repetition length because `list.repeat` does it itself.
				list.repeat(amount, env.opts()).map(Self::from)
			}

			#[cfg(feature = "extensions")]
			Self::Boolean(lhs) if env.opts().extensions.builtin_fns.boolean => {
				Ok((lhs & rhs.to_boolean(env)?).into())
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.multiply(rhs, env),

			other => Err(Error::TypeError { type_name: other.type_name(), function: "*" }),
		}
	}

	pub fn kn_slash(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => Ok(integer.divide(rhs.to_integer(env)?, env.opts())?.into()),

			#[cfg(feature = "extensions")]
			Self::String(string) if env.opts().extensions.builtin_fns.string => {
				Ok(string.split(&rhs.to_kstring(env)?, env).into())
			}

			#[cfg(feature = "extensions")]
			Self::List(list) if env.opts().extensions.builtin_fns.list => {
				Ok(list.reduce(rhs, env)?.unwrap_or_default())
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.divide(rhs, env),

			other => Err(Error::TypeError { type_name: other.type_name(), function: "/" }),
		}
	}

	pub fn kn_percent(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => Ok(integer.remainder(rhs.to_integer(env)?, env.opts())?.into()),

			// #[cfg(feature = "string-extensions")]
			// Self::String(lstr) => {
			// 	let values = rhs.to_list(env)?;
			// 	let mut values_index = 0;

			// 	let mut formatted = String::new();
			// 	let mut chars = lstr.chars();

			// 	while let Some(chr) = chars.next() {
			// 		match chr {
			// 			'\\' => {
			// 				formatted.push(match chars.next().expect("<todo error for nothing next>") {
			// 					'n' => '\n',
			// 					'r' => '\r',
			// 					't' => '\t',
			// 					'{' => '{',
			// 					'}' => '}',
			// 					_ => panic!("todo: error for unknown escape code"),
			// 				});
			// 			}
			// 			'{' => {
			// 				if chars.next() != Some('}') {
			// 					panic!("todo, missing closing `}}`");
			// 				}
			// 				formatted.push_str(
			// 					&values
			// 						.as_slice()
			// 						.get(values_index)
			// 						.expect("no values left to format")
			// 						.to_kstring(env)?,
			// 				);
			// 				values_index += 1;
			// 			}
			// 			_ => formatted.push(chr),
			// 		}
			// 	}

			// 	Text::new(formatted).unwrap().into()
			// }
			#[cfg(feature = "extensions")]
			Self::List(list) if env.opts().extensions.builtin_fns.list => {
				list.filter(rhs, env).map(Self::from)
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.remainder(rhs, env),

			other => Err(Error::TypeError { type_name: other.type_name(), function: "%" }),
		}
	}

	pub fn kn_caret(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => Ok(integer.power(rhs.to_integer(env)?, env.opts())?.into()),
			Self::List(list) => list.join(&rhs.to_kstring(env)?, env).map(Self::from),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.power(rhs, env),

			other => Err(Error::TypeError { type_name: other.type_name(), function: "^" }),
		}
	}

	/// Gets the first element of `self`.
	///
	/// # Extensions
	/// If [integer extensions](crate::env::flags::Types::integer) are enabled, and `self` is an
	/// integer, the most significant digit is returned
	///
	/// # Errors
	/// If `self` is either a [`Text`] or a [`List`] and is empty, an [`Error::DomainError`] is
	/// returned. If `self`
	pub fn kn_head(&self, env: &mut Environment) -> Result<Self> {
		let _ = env;
		match self {
			Self::List(list) => list.head().ok_or(Error::DomainError("empty list")),
			Self::String(string) => string
				.head()
				.ok_or(Error::DomainError("empty string"))
				.map(|chr| KString::new_unvalidated(chr.to_string()).into()),

			// #[cfg(feature = "extensions")]
			// Self::Integer(integer) if env.flags().extensions.types.integer => Ok(integer.head().into()),

			// #[cfg(feature = "custom-types")]
			// Self::Custom(custom) => custom.head(env),
			other => Err(Error::TypeError { type_name: other.type_name(), function: "[" }),
		}
	}

	pub fn kn_tail(&self, env: &mut Environment) -> Result<Self> {
		let _ = env;
		match self {
			Self::List(list) => list.tail().ok_or(Error::DomainError("empty list")).map(Self::from),
			Self::String(string) => {
				string.tail().ok_or(Error::DomainError("empty string")).map(|x| KString::from(x).into())
			}

			// #[cfg(feature = "extensions")]
			// Self::Integer(integer) if env.flags().extensions.types.integer => Ok(integer.tail().into()),

			// #[cfg(feature = "custom-types")]
			// Self::Custom(custom) => custom.tail(env),
			other => Err(Error::TypeError { type_name: other.type_name(), function: "]" }),
		}
	}

	pub fn kn_ascii(&self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => {
				let chr = integer.chr(env.opts())?;
				Ok(KString::new_unvalidated(chr.to_string()).into())
			}
			Self::String(string) => Ok(string.ord(env.opts())?.into()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.ascii(env),

			other => Err(Error::TypeError { type_name: other.type_name(), function: "ASCII" }),
		}
	}

	pub fn kn_get(&self, start: &Value, len: &Value, env: &mut Environment) -> Result<Self> {
		#[cfg(feature = "custom-types")]
		if let Self::Custom(custom) = self {
			return custom.get(start, len, env);
		}

		let start = fix_len(self, start.to_integer(env)?, "GET", env)?;
		let len = usize::try_from(len.to_integer(env)?.inner())
			.or(Err(Error::DomainError("negative length")))?;

		match self {
			Self::List(list) => list.try_get(start..start + len).map(Self::from),

			Self::String(text) => text
				.get(start..start + len)
				.ok_or(Error::IndexOutOfBounds { len: text.len(), index: start + len })
				.map(ToOwned::to_owned)
				.map(Self::from),

			other => return Err(Error::TypeError { type_name: other.type_name(), function: "GET" }),
		}
	}

	pub fn kn_set(
		&self,
		start: &Value,
		len: &Value,
		replacement: &Value,
		env: &mut Environment,
	) -> Result<Self> {
		#[cfg(feature = "custom-types")]
		if let Self::Custom(custom) = self {
			return custom.set(start, len, replacement, env);
		}

		let start = fix_len(self, start.to_integer(env)?, "SET", env)?;
		let len = usize::try_from(len.to_integer(env)?.inner())
			.or(Err(Error::DomainError("negative length")))?;

		match self {
			Self::List(list) => {
				let replacement = replacement.to_list(env)?;
				let mut ret = Vec::new();

				ret.extend(list.iter().take(start).cloned());
				ret.extend(replacement.iter().cloned());
				ret.extend(list.iter().skip((start) + len).cloned());

				List::new(ret, env.opts()).map(Self::from)
			}
			Self::String(string) => {
				let replacement = replacement.to_kstring(env)?;

				// lol, todo, optimize me
				let mut builder = String::new();
				builder.push_str(string.get(..start).unwrap().as_str());
				builder.push_str(&replacement.as_str());
				builder.push_str(string.get(start + len..).unwrap().as_str());
				Ok(KString::new(builder, env.opts())?.into())
			}

			other => return Err(Error::TypeError { type_name: other.type_name(), function: "SET" }),
		}
	}
}

fn fix_len(
	container: &Value,
	#[cfg_attr(not(feature = "extensions"), allow(unused_mut))] mut start: Integer,
	function: &'static str,
	env: &mut Environment,
) -> Result<usize> {
	#[cfg(feature = "extensions")]
	if env.opts().extensions.negative_indexing && start < Integer::ZERO {
		let len = match container {
			Value::String(string) => string.len(),
			Value::List(list) => list.len(),

			#[cfg(feature = "custom-types")]
			Value::Custom(custom) => custom.length(env)?,

			other => return Err(Error::TypeError { type_name: other.type_name(), function }),
		};

		start = start.add(Integer::new_error(len as _, env.opts())?, env.opts())?;
	}

	let _ = (container, env);
	usize::try_from(start.inner()).or(Err(Error::DomainError("negative start position")))
}

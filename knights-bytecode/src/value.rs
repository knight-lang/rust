use std::cmp::Ordering;

use crate::{
	vm::{program::JumpIndex, Vm},
	Environment, Error, Result,
};

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
	pub fn dump(&self) {
		print!("{:?}", self)
	}

	pub fn compare(&self, rhs: &Self, env: &mut Environment) -> Result<Ordering> {
		match self {
			Self::Integer(int) => Ok(int.cmp(&rhs.to_integer(env)?)),
			_ => todo!(),
		}
	}

	pub fn is_equal(&self, rhs: &Self, env: &mut Environment) -> Result<bool> {
		match self {
			Self::Integer(int) => Ok(*int == rhs.to_integer(env)?),
			_ => todo!(),
		}
	}

	pub fn call(&self, vm: &mut Vm) -> Result<Value> {
		match self {
			Self::Block(block) => vm.child_stackframe(*block),
			_ => todo!(),
		}
	}

	pub fn length(&self, env: &mut Environment) -> Result<Integer> {
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
					return Ok(Integer::new(string.len() as i64, env.opts())?);
				}

				Ok(Integer::new_unvalidated(string.len() as i64))
			}

			Self::List(list) => {
				// (same guarantees as `Self::String`)
				#[cfg(feature = "compliance")]
				if env.opts().compliance.i32_integer && !env.opts().compliance.check_container_length {
					return Ok(Integer::new(list.len() as i64, env.opts())?);
				}

				Ok(Integer::new_unvalidated(list.len() as i64))
			}

			// TODO: Knight 2.0.1 extensions?
			other => Err(Error::TypeError { type_name: other.type_name(), function: "LENGTH" }),
		}
	}

	pub fn negate(&self, env: &mut Environment) -> Result<Integer> {
		#[cfg(feature = "extensions")]
		if env.opts().extensions.breaking.negate_reverses_collections {
			todo!();
		}

		Ok(self.to_integer(env)?.negate(env.opts())?)
	}

	pub fn op_plus(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
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

	pub fn op_minus(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
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

	pub fn op_asterisk(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
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

	pub fn op_slash(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
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

	pub fn op_percent(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
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

	pub fn op_caret(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => Ok(integer.power(rhs.to_integer(env)?, env.opts())?.into()),
			Self::List(list) => list.join(&rhs.to_kstring(env)?, env).map(Self::from),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.power(rhs, env),

			other => Err(Error::TypeError { type_name: other.type_name(), function: "^" }),
		}
	}
}

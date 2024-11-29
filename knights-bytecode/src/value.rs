use crate::{Environment, Error, Result};

pub mod boolean;
pub mod integer;
pub mod list;
pub mod null;
pub mod string;
pub use boolean::{Boolean, ToBoolean};
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
#[derive(Debug, Clone)]
pub enum Value {
	Null,
	Boolean(Boolean),
	Integer(Integer),
	String(KString),
	List(List),
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

impl ToBoolean for Value {
	fn to_boolean(&self, env: &mut Environment) -> Result<Boolean> {
		match self {
			Self::Null => Null.to_boolean(env),
			Self::Boolean(boolean) => boolean.to_boolean(env),
			Self::Integer(integer) => integer.to_boolean(env),
			Self::String(string) => string.to_boolean(env),
			Self::List(list) => list.to_boolean(env),
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
			// Self::Ast(_) => Ast::TYPENAME,
			// Self::Variable(_) => Variable::TYPENAME,
			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.type_name(),
		}
	}
}

impl Value {
	pub fn add(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => Ok(integer.add(rhs.to_integer(env)?, env.opts())?.into()),
			Self::String(string) => Ok(string.concat(&rhs.to_kstring(env)?, env.opts())?.into()),
			Self::List(list) => {
				list.concat(rhs.to_list(env)?.into_iter().cloned(), env.opts()).map(Self::from)
			}
			#[cfg(feature = "extensions")]
			Self::Boolean(lhs) if env.opts().extensions.types.boolean => {
				Ok((lhs | rhs.to_boolean(env)?).into())
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.add(rhs, env),

			other => Err(Error::TypeError { type_name: other.type_name(), function: "+" }),
		}
	}

	pub fn subtract(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => Ok(integer.subtract(rhs.to_integer(env)?, env.opts())?.into()),

			#[cfg(feature = "extensions")]
			Self::String(string) if env.opts().extensions.types.string => {
				Ok(string.remove_substr(&rhs.to_kstring(env)?).into())
			}

			#[cfg(feature = "extensions")]
			Self::List(list) if env.opts().extensions.types.list => {
				Ok(list.difference(&rhs.to_list(env)?, env.opts())?.into())
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.subtract(rhs, env),

			other => Err(Error::TypeError { type_name: other.type_name(), function: "-" }),
		}
	}

	// pub fn multiply(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
	// 	match self {
	// 		Self::Integer(integer) => {
	// 			integer.multiply(rhs.to_integer(env)?, env.opts()).map(Self::from)
	// 		}

	// 		Self::String(lstr) => {
	// 			let amount = usize::try_from(rhs.to_integer(env)?)
	// 				.or(Err(Error::DomainError("repetition count is negative")))?;

	// 			if amount.checked_mul(lstr.len()).map_or(true, |c| isize::MAX as usize <= c) {
	// 				return Err(Error::DomainError("repetition is too large"));
	// 			}

	// 			Ok(lstr.repeat(amount, env.opts())?.into())
	// 		}

	// 		Self::List(list) => {
	// 			let rhs = rhs;

	// 			// Multiplying by a block is invalid, so we can do this as an extension.
	// 			#[cfg(feature = "extensions")]
	// 			if env.opts().extensions.types.list && matches!(rhs, Self::Ast(_)) {
	// 				return list.map(rhs, env).map(Self::from);
	// 			}

	// 			let amount = usize::try_from(rhs.to_integer(env)?)
	// 				.or(Err(Error::DomainError("repetition count is negative")))?;

	// 			// No need to check for repetition length because `list.repeat` does it itself.
	// 			list.repeat(amount, env.opts()).map(Self::from)
	// 		}

	// 		#[cfg(feature = "extensions")]
	// 		Self::Boolean(lhs) if env.opts().extensions.types.boolean => {
	// 			Ok((lhs & rhs.to_boolean(env)?).into())
	// 		}

	// 		#[cfg(feature = "custom-types")]
	// 		Self::Custom(custom) => custom.multiply(rhs, env),

	// 		other => Err(Error::TypeError(other.typename(), "*")),
	// 	}
	// }

	// pub fn divide(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
	// 	match self {
	// 		Self::Integer(integer) => {
	// 			integer.divide(rhs.to_integer(env)?, env.opts()).map(Self::from)
	// 		}

	// 		#[cfg(feature = "extensions")]
	// 		Self::String(string) if env.opts().extensions.types.string => {
	// 			Ok(string.split(&rhs.to_text(env)?, env).into())
	// 		}

	// 		#[cfg(feature = "extensions")]
	// 		Self::List(list) if env.opts().extensions.types.list => {
	// 			Ok(list.reduce(rhs, env)?.unwrap_or_default())
	// 		}

	// 		#[cfg(feature = "custom-types")]
	// 		Self::Custom(custom) => custom.divide(rhs, env),

	// 		other => Err(Error::TypeError(other.typename(), "/")),
	// 	}
	// }

	// pub fn remainder(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
	// 	match self {
	// 		Self::Integer(integer) => {
	// 			integer.remainder(rhs.to_integer(env)?, env.opts()).map(Self::from)
	// 		}

	// 		// #[cfg(feature = "string-extensions")]
	// 		// Self::String(lstr) => {
	// 		// 	let values = rhs.to_list(env)?;
	// 		// 	let mut values_index = 0;

	// 		// 	let mut formatted = String::new();
	// 		// 	let mut chars = lstr.chars();

	// 		// 	while let Some(chr) = chars.next() {
	// 		// 		match chr {
	// 		// 			'\\' => {
	// 		// 				formatted.push(match chars.next().expect("<todo error for nothing next>") {
	// 		// 					'n' => '\n',
	// 		// 					'r' => '\r',
	// 		// 					't' => '\t',
	// 		// 					'{' => '{',
	// 		// 					'}' => '}',
	// 		// 					_ => panic!("todo: error for unknown escape code"),
	// 		// 				});
	// 		// 			}
	// 		// 			'{' => {
	// 		// 				if chars.next() != Some('}') {
	// 		// 					panic!("todo, missing closing `}}`");
	// 		// 				}
	// 		// 				formatted.push_str(
	// 		// 					&values
	// 		// 						.as_slice()
	// 		// 						.get(values_index)
	// 		// 						.expect("no values left to format")
	// 		// 						.to_text(env)?,
	// 		// 				);
	// 		// 				values_index += 1;
	// 		// 			}
	// 		// 			_ => formatted.push(chr),
	// 		// 		}
	// 		// 	}

	// 		// 	Text::new(formatted).unwrap().into()
	// 		// }
	// 		#[cfg(feature = "extensions")]
	// 		Self::List(list) if env.opts().extensions.types.list => list.filter(rhs, env).map(Self::from),

	// 		#[cfg(feature = "custom-types")]
	// 		Self::Custom(custom) => custom.remainder(rhs, env),

	// 		other => Err(Error::TypeError(other.typename(), "%")),
	// 	}
	// }

	// pub fn power(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
	// 	match self {
	// 		Self::Integer(integer) => integer.power(rhs.to_integer(env)?, env.opts()).map(Self::from),
	// 		Self::List(list) => list.join(&rhs.to_text(env)?, env).map(Self::from),

	// 		#[cfg(feature = "custom-types")]
	// 		Self::Custom(custom) => custom.power(rhs, env),

	// 		other => Err(Error::TypeError(other.typename(), "^")),
	// 	}
	// }
}

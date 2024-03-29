use crate::env::{Environment, Variable};
use crate::value::{
	Boolean, Integer, List, NamedType, Null, Runnable, Text, ToBoolean, ToInteger, ToList, ToText,
};
use crate::{Ast, Error, Result};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};

/// A Value within Knight.
#[derive(Default, Clone, PartialEq, Hash)]
#[non_exhaustive]
pub enum Value {
	/// Represents the `NULL` value.
	#[default]
	Null,

	/// Represents the `TRUE` and `FALSE` values.
	Boolean(Boolean),

	/// Represents integers.
	Integer(Integer),

	/// Represents a string.
	Text(Text),

	/// Represents a list of [`Value`]s.
	List(List),

	/// Represents a variable.
	Variable(Variable),

	/// Represents a block of code.
	Ast(Ast),

	/// Represents a custom type.
	#[cfg(feature = "custom-types")]
	#[cfg_attr(docsrs, doc(cfg(feature = "custom-types")))]
	Custom(crate::value::Custom),
}

impl Debug for Value {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::Null => Debug::fmt(&Null, f),
			Self::Boolean(boolean) => Debug::fmt(boolean, f),
			Self::Integer(integer) => Debug::fmt(integer, f),
			Self::Text(text) => Debug::fmt(text, f),
			Self::List(list) => Debug::fmt(list, f),
			Self::Variable(variable) => Debug::fmt(variable, f),
			Self::Ast(ast) => Debug::fmt(ast, f),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => Debug::fmt(custom, f),
		}
	}
}

impl From<Null> for Value {
	#[inline]
	fn from(_: Null) -> Self {
		Self::Null
	}
}

impl From<Boolean> for Value {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		Self::Boolean(boolean)
	}
}

impl From<Integer> for Value {
	#[inline]
	fn from(integer: Integer) -> Self {
		Self::Integer(integer)
	}
}

impl From<Text> for Value {
	#[inline]
	fn from(text: Text) -> Self {
		Self::Text(text)
	}
}

impl From<Variable> for Value {
	#[inline]
	fn from(variable: Variable) -> Self {
		Self::Variable(variable)
	}
}

impl From<Ast> for Value {
	#[inline]
	fn from(inp: Ast) -> Self {
		Self::Ast(inp)
	}
}

impl From<List> for Value {
	#[inline]
	fn from(list: List) -> Self {
		Self::List(list)
	}
}

#[cfg(feature = "custom-types")]
impl From<crate::value::Custom> for Value {
	#[inline]
	fn from(custom: crate::value::Custom) -> Self {
		Self::Custom(custom)
	}
}

impl ToBoolean for Value {
	fn to_boolean(&self, env: &mut Environment) -> Result<Boolean> {
		match *self {
			Self::Null => Null.to_boolean(env),
			Self::Boolean(boolean) => boolean.to_boolean(env),
			Self::Integer(integer) => integer.to_boolean(env),
			Self::Text(ref text) => text.to_boolean(env),
			Self::List(ref list) => list.to_boolean(env),

			#[cfg(feature = "custom-types")]
			Self::Custom(ref custom) => custom.to_boolean(env),

			_ => Err(Error::NoConversion { to: Boolean::TYPENAME, from: self.typename() }),
		}
	}
}

impl ToInteger for Value {
	fn to_integer(&self, env: &mut Environment) -> Result<Integer> {
		match *self {
			Self::Null => Null.to_integer(env),
			Self::Boolean(boolean) => boolean.to_integer(env),
			Self::Integer(integer) => integer.to_integer(env),
			Self::Text(ref text) => text.to_integer(env),
			Self::List(ref list) => list.to_integer(env),

			#[cfg(feature = "custom-types")]
			Self::Custom(ref custom) => custom.to_integer(env),

			_ => Err(Error::NoConversion { to: Integer::TYPENAME, from: self.typename() }),
		}
	}
}

impl ToText for Value {
	fn to_text(&self, env: &mut Environment) -> Result<Text> {
		match *self {
			Self::Null => Null.to_text(env),
			Self::Boolean(boolean) => boolean.to_text(env),
			Self::Integer(ref integer) => integer.to_text(env),
			Self::Text(ref text) => text.to_text(env),
			Self::List(ref list) => list.to_text(env),

			#[cfg(feature = "custom-types")]
			Self::Custom(ref custom) => custom.to_text(env),

			_ => Err(Error::NoConversion { to: Text::TYPENAME, from: self.typename() }),
		}
	}
}

impl ToList for Value {
	fn to_list(&self, env: &mut Environment) -> Result<List> {
		match *self {
			Self::Null => Null.to_list(env),
			Self::Boolean(boolean) => boolean.to_list(env),
			Self::Integer(integer) => integer.to_list(env),
			Self::Text(ref text) => text.to_list(env),
			Self::List(ref list) => list.to_list(env),

			#[cfg(feature = "custom-types")]
			Self::Custom(ref custom) => custom.to_list(env),

			_ => Err(Error::NoConversion { to: List::TYPENAME, from: self.typename() }),
		}
	}
}

impl Runnable for Value {
	fn run(&self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Variable(variable) => variable.run(env),
			Self::Ast(ast) => ast.run(env),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.run(env),

			_ => Ok(self.clone()),
		}
	}
}

impl Value {
	/// Fetch the type's name.
	#[must_use = "getting the type name by itself does nothing."]
	pub fn typename(&self) -> &'static str {
		match self {
			Self::Null => Null::TYPENAME,
			Self::Boolean(_) => Boolean::TYPENAME,
			Self::Integer(_) => Integer::TYPENAME,
			Self::Text(_) => Text::TYPENAME,
			Self::List(_) => List::TYPENAME,
			Self::Ast(_) => Ast::TYPENAME,
			Self::Variable(_) => Variable::TYPENAME,

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.typename(),
		}
	}

	/// Calls `self`.
	///
	/// # Errors
	/// If [`check_call_arg`](crate::env::flags::Compliance::check_call_arg) is enabled and `self`
	/// isn't a [`Value::Ast`], This will return a `TypeError`. Errors that result from calling
	/// [`run`](Self::run) are also propogated.
	pub fn call(&self, env: &mut Environment) -> Result<Self> {
		// When ensuring that `CALL` is only given values returned from `BLOCK`, we must ensure that
		// all arguments are `Value::Ast`s.
		#[cfg(feature = "compliance")]
		if env.flags().compliance.check_call_arg && !matches!(self, Value::Ast(_)) {
			return Err(Error::TypeError(self.typename(), "CALL"));
		}

		self.run(env)
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
	pub fn head(&self, env: &mut Environment) -> Result<Self> {
		let _ = env;
		match self {
			Self::List(list) => list.head().ok_or(Error::DomainError("empty list")),
			Self::Text(text) => text
				.head()
				.ok_or(Error::DomainError("empty text"))
				.map(|chr| unsafe { Text::new_unchecked(chr) }.into()),

			#[cfg(feature = "extensions")]
			Self::Integer(integer) if env.flags().extensions.types.integer => Ok(integer.head().into()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.head(env),

			other => Err(Error::TypeError(other.typename(), "[")),
		}
	}

	pub fn tail(&self, env: &mut Environment) -> Result<Self> {
		let _ = env;
		match self {
			Self::List(list) => list.tail().ok_or(Error::DomainError("empty list")).map(Self::from),
			Self::Text(text) => {
				text.tail().ok_or(Error::DomainError("empty text")).map(|x| Text::from(x).into())
			}

			#[cfg(feature = "extensions")]
			Self::Integer(integer) if env.flags().extensions.types.integer => Ok(integer.tail().into()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.tail(env),

			other => Err(Error::TypeError(other.typename(), "]")),
		}
	}

	pub fn length(&self, env: &mut Environment) -> Result<Self> {
		let _ = env;
		match self {
			Self::List(list) => Integer::try_from(list.len()).map(Self::from),
			Self::Text(text) => Integer::try_from(text.len()).map(Self::from),
			Self::Integer(int) => Ok(Integer::try_from(int.number_of_digits()).unwrap().into()),
			Self::Boolean(true) => Ok(Integer::ONE.into()),
			Self::Boolean(false) | Self::Null => Ok(Integer::ZERO.into()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => Integer::try_from(custom.length(env)?).map(Self::from),

			other => Err(Error::TypeError(other.typename(), "LENGTH")),
		}
	}

	pub fn ascii(&self, env: &mut Environment) -> Result<Self> {
		let _ = env;
		match self {
			Self::Integer(integer) => Ok({
				let chr = integer.chr(env.flags())?;
				unsafe { Text::new_unchecked(chr) }.into()
			}),
			Self::Text(text) => Ok(text.ord()?.into()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.ascii(env),

			other => Err(Error::TypeError(other.typename(), "ASCII")),
		}
	}

	pub fn add(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => integer.add(rhs.to_integer(env)?, env.flags()).map(Self::from),
			Self::Text(string) => Ok(string.concat(&rhs.to_text(env)?, env.flags())?.into()),
			Self::List(list) => list.concat(&rhs.to_list(env)?, env.flags()).map(Self::from),

			#[cfg(feature = "extensions")]
			Self::Boolean(lhs) if env.flags().extensions.types.boolean => {
				Ok((lhs | rhs.to_boolean(env)?).into())
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.add(rhs, env),

			other => Err(Error::TypeError(other.typename(), "+")),
		}
	}

	pub fn subtract(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => {
				integer.subtract(rhs.to_integer(env)?, env.flags()).map(Self::from)
			}

			#[cfg(feature = "extensions")]
			Self::Text(text) if env.flags().extensions.types.text => {
				Ok(text.remove_substr(&rhs.to_text(env)?).into())
			}

			#[cfg(feature = "extensions")]
			Self::List(list) if env.flags().extensions.types.list => {
				list.difference(&rhs.to_list(env)?).map(Self::from)
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.subtract(rhs, env),

			other => Err(Error::TypeError(other.typename(), "-")),
		}
	}

	pub fn multiply(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => {
				integer.multiply(rhs.to_integer(env)?, env.flags()).map(Self::from)
			}

			Self::Text(lstr) => {
				let amount = usize::try_from(rhs.to_integer(env)?)
					.or(Err(Error::DomainError("repetition count is negative")))?;

				if amount.checked_mul(lstr.len()).map_or(true, |c| isize::MAX as usize <= c) {
					return Err(Error::DomainError("repetition is too large"));
				}

				Ok(lstr.repeat(amount, env.flags())?.into())
			}

			Self::List(list) => {
				let rhs = rhs;

				// Multiplying by a block is invalid, so we can do this as an extension.
				#[cfg(feature = "extensions")]
				if env.flags().extensions.types.list && matches!(rhs, Self::Ast(_)) {
					return list.map(rhs, env).map(Self::from);
				}

				let amount = usize::try_from(rhs.to_integer(env)?)
					.or(Err(Error::DomainError("repetition count is negative")))?;

				// No need to check for repetition length because `list.repeat` does it itself.
				list.repeat(amount, env.flags()).map(Self::from)
			}

			#[cfg(feature = "extensions")]
			Self::Boolean(lhs) if env.flags().extensions.types.boolean => {
				Ok((lhs & rhs.to_boolean(env)?).into())
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.multiply(rhs, env),

			other => Err(Error::TypeError(other.typename(), "*")),
		}
	}

	pub fn divide(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => {
				integer.divide(rhs.to_integer(env)?, env.flags()).map(Self::from)
			}

			#[cfg(feature = "extensions")]
			Self::Text(text) if env.flags().extensions.types.text => {
				Ok(text.split(&rhs.to_text(env)?, env).into())
			}

			#[cfg(feature = "extensions")]
			Self::List(list) if env.flags().extensions.types.list => {
				Ok(list.reduce(rhs, env)?.unwrap_or_default())
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.divide(rhs, env),

			other => Err(Error::TypeError(other.typename(), "/")),
		}
	}

	pub fn remainder(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => {
				integer.remainder(rhs.to_integer(env)?, env.flags()).map(Self::from)
			}

			// #[cfg(feature = "string-extensions")]
			// Self::Text(lstr) => {
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
			// 						.to_text(env)?,
			// 				);
			// 				values_index += 1;
			// 			}
			// 			_ => formatted.push(chr),
			// 		}
			// 	}

			// 	Text::new(formatted).unwrap().into()
			// }
			#[cfg(feature = "extensions")]
			Self::List(list) if env.flags().extensions.types.list => list.filter(rhs, env).map(Self::from),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.remainder(rhs, env),

			other => Err(Error::TypeError(other.typename(), "%")),
		}
	}

	pub fn power(&self, rhs: &Self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Integer(integer) => integer.power(rhs.to_integer(env)?, env.flags()).map(Self::from),
			Self::List(list) => list.join(&rhs.to_text(env)?, env).map(Self::from),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.power(rhs, env),

			other => Err(Error::TypeError(other.typename(), "^")),
		}
	}

	pub fn compare(&self, rhs: &Self, env: &mut Environment) -> Result<Ordering> {
		match self {
			Value::Integer(integer) => Ok(integer.cmp(&rhs.to_integer(env)?)),
			Value::Boolean(boolean) => Ok(boolean.cmp(&rhs.to_boolean(env)?)),
			Value::Text(text) => Ok(text.cmp(&rhs.to_text(env)?)),
			Value::List(list) => {
				let rhs = rhs.to_list(env)?;

				for (left, right) in list.iter().zip(&rhs) {
					match left.compare(right, env)? {
						Ordering::Equal => {}
						other => return Ok(other),
					}
				}

				Ok(list.len().cmp(&rhs.len()))
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.compare(rhs, env),

			other => Err(Error::TypeError(other.typename(), "<cmp>")),
		}
	}

	pub fn equals(&self, rhs: &Self, env: &mut Environment<'_>) -> Result<bool> {
		#[cfg(feature = "compliance")]
		{
			fn check_for_strict_compliance(value: &Value) -> Result<()> {
				match value {
					Value::List(list) => {
						for ele in list {
							check_for_strict_compliance(ele)?;
						}
						Ok(())
					}
					Value::Ast(_) | Value::Variable(_) => Err(Error::TypeError(value.typename(), "?")),
					_ => Ok(()),
				}
			}

			if env.flags().compliance.check_equals_params {
				check_for_strict_compliance(self)?;
				check_for_strict_compliance(rhs)?;
			}
		}

		let _ = env;
		Ok(self == rhs)
	}

	pub fn assign(&self, value: Self, env: &mut Environment) -> Result<()> {
		let _ = env;

		if let Value::Variable(variable) = self {
			variable.assign(value);
			return Ok(());
		}

		#[cfg(feature = "extensions")]
		match self {
			Value::Variable(_) => unreachable!(),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.assign(value, env)?,

			Value::Ast(ast)
				if env.flags().extensions.assign_to.prompt
					&& ast.function().full_name() == "PROMPT" =>
			{
				match value {
					// `= PROMPT NULL` or `= PROMPT FALSE` makes it always return nothing.
					Value::Null | Value::Boolean(false) => env.prompt().eof(),

					// `= PROMPT TRUE` clears all replacements
					Value::Boolean(true) => env.prompt().reset_replacement(),

					// `= PROMPT "foo<newline>bar"` will add the two lines to the end of the buffer.
					// after the buffer's exhausted, it's assumed to be EOF.
					Value::Text(text) => env.prompt().add_lines(&text),

					// `= PROMPT BLOCK ...` will compute the new ast each time
					Value::Ast(ast) => env.prompt().set_ast(ast),

					// any other type is an error. (todo: maybe allow variables and evaluate them?)
					other => return Err(Error::TypeError(other.typename(), "=")),
				}

				return Ok(());
			}

			Value::Ast(ast)
				if env.flags().extensions.assign_to.output
					&& ast.function().full_name() == "OUTPUT" =>
			{
				match value {
					Value::Null => env.output().clear_redirection(),
					Value::Variable(var) => env.output().set_redirection(var),
					other => return Err(Error::TypeError(other.typename(), "=")),
				}
			}

			Value::Ast(ast)
				if env.flags().extensions.assign_to.prompt && ast.function().full_name() == "$" =>
			{
				let lines = value.to_text(env)?;
				env.add_to_system(lines);
				return Ok(());
			}

			other => match other.run(env)? {
				Value::List(_list) if env.flags().extensions.assign_to.list => todo!(),
				Value::Text(name) if env.flags().extensions.assign_to.text => {
					env.lookup(&name)?.assign(value);
					return Ok(());
				}
				_ => { /* fallthrough */ }
			},
		}

		Err(Error::TypeError(self.typename(), "="))
	}

	pub fn get(&self, start: &Self, len: &Self, env: &mut Environment) -> Result<Self> {
		#[cfg(feature = "custom-types")]
		if let Self::Custom(custom) = self {
			return custom.get(start, len, env);
		}

		let start = fix_len(self, start.to_integer(env)?, env)?;
		let len =
			usize::try_from(len.to_integer(env)?).or(Err(Error::DomainError("negative length")))?;

		match self {
			Self::List(list) => list.try_get(start..start + len).map(Self::from),

			Self::Text(text) => text
				.get(start..start + len)
				.ok_or(Error::IndexOutOfBounds { len: text.len(), index: start + len })
				.map(ToOwned::to_owned)
				.map(Self::from),

			other => return Err(Error::TypeError(other.typename(), "GET")),
		}
	}

	pub fn set(
		&self,
		start: &Self,
		len: &Self,
		replacement: Self,
		env: &mut Environment,
	) -> Result<Self> {
		#[cfg(feature = "custom-types")]
		if let Self::Custom(custom) = self {
			return custom.set(start, len, replacement, env);
		}

		let start = fix_len(self, start.to_integer(env)?, env)?;
		let len =
			usize::try_from(len.to_integer(env)?).or(Err(Error::DomainError("negative length")))?;

		match self {
			Self::List(list) => {
				// OPTIMIZE ME: cons?
				let replacement = replacement.to_list(env)?;
				let mut ret = Vec::new();

				ret.extend(list.iter().take(start).cloned());
				ret.extend(replacement.iter().cloned());
				ret.extend(list.iter().skip((start) + len).cloned());

				List::new(ret, env.flags()).map(Self::from)
			}
			Self::Text(text) => {
				let replacement = replacement.to_text(env)?;

				// lol, todo, optimize me
				let mut builder = Text::builder();
				builder.push(text.get(..start).unwrap());
				builder.push(&replacement);
				builder.push(text.get(start + len..).unwrap());
				Ok(builder.finish(env.flags())?.into())
			}

			other => return Err(Error::TypeError(other.typename(), "SET")),
		}
	}
}

fn fix_len(
	container: &Value,
	#[cfg_attr(not(feature = "extensions"), allow(unused_mut))] mut start: Integer,
	env: &mut Environment,
) -> Result<usize> {
	#[cfg(feature = "extensions")]
	if env.flags().extensions.negative_indexing && start < 0 {
		let len = match container {
			Value::Text(text) => text.len(),
			Value::List(list) => list.len(),

			#[cfg(feature = "custom-types")]
			Value::Custom(custom) => custom.length(env)?,

			other => return Err(Error::TypeError(other.typename(), "get/set")),
		};

		start = start.add(len.try_into()?, env.flags())?;
	}

	let _ = (container, env);
	usize::try_from(start).or(Err(Error::DomainError("negative start position")))
}

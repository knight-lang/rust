use crate::env::Environment;
use crate::value::integer::IntType;
use crate::value::text::Character;
use crate::value::{
	Boolean, Integer, List, NamedType, Null, Runnable, Text, ToBoolean, ToInteger, ToList, ToText,
};
use crate::{Ast, Error, Result, Variable};
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};

/// A Value within Knight.
#[derive(Default, Clone, PartialEq)]
pub enum Value<'e, I: IntType> {
	#[default]
	/// Represents the `NULL` value.
	Null,

	/// Represents the `TRUE` and `FALSE` values.
	Boolean(Boolean),

	/// Represents integers.
	Integer(Integer<I>),

	/// Represents a string.
	Text(Text),

	/// Represents a list of [`Value`]s.
	List(List<'e, I>),

	/// Represents a variable.
	Variable(Variable<'e, I>),

	/// Represents a block of code.
	Ast(Ast<'e, I>),

	/// Represents a custom type.
	#[cfg(feature = "custom-types")]
	#[cfg_attr(docsrs, doc(cfg(feature = "custom-types")))]
	Custom(crate::value::Custom<'e, I>),
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Value<'_>: Send, Sync);

impl<I: IntType> Debug for Value<'_, I> {
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

impl<I: IntType> From<Null> for Value<'_, I> {
	#[inline]
	fn from(_: Null) -> Self {
		Self::Null
	}
}

impl<I: IntType> From<Boolean> for Value<'_, I> {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		Self::Boolean(boolean)
	}
}

impl<I: IntType> From<Integer<I>> for Value<'_, I> {
	#[inline]
	fn from(integer: Integer<I>) -> Self {
		Self::Integer(integer)
	}
}

impl<I: IntType> From<Text> for Value<'_, I> {
	#[inline]
	fn from(text: Text) -> Self {
		Self::Text(text)
	}
}

impl<I: IntType> From<Character> for Value<'_, I> {
	#[inline]
	fn from(character: Character) -> Self {
		Self::Text(Text::from(character))
	}
}

impl<'e, I: IntType> From<Variable<'e, I>> for Value<'e, I> {
	#[inline]
	fn from(variable: Variable<'e, I>) -> Self {
		Self::Variable(variable)
	}
}

impl<'e, I: IntType> From<Ast<'e, I>> for Value<'e, I> {
	#[inline]
	fn from(inp: Ast<'e, I>) -> Self {
		Self::Ast(inp)
	}
}

impl<'e, I: IntType> From<List<'e, I>> for Value<'e, I> {
	#[inline]
	fn from(list: List<'e, I>) -> Self {
		Self::List(list)
	}
}

#[cfg(feature = "custom-types")]
impl<'e, I: IntType> From<crate::value::Custom<'e>> for Value<'e, I> {
	#[inline]
	fn from(custom: crate::value::Custom<'e>) -> Self {
		Self::Custom(custom)
	}
}

impl<I: IntType> Value<'_, I> {
	/// Fetch the type's name.
	#[must_use = "getting the type name by itself does nothing."]
	pub fn typename(&self) -> &'static str {
		match self {
			Self::Null => Null::TYPENAME,
			Self::Boolean(_) => Boolean::TYPENAME,
			Self::Integer(_) => Integer::<I>::TYPENAME,
			Self::Text(_) => Text::TYPENAME,
			Self::List(_) => List::<I>::TYPENAME,
			Self::Ast(_) => Ast::<I>::TYPENAME,
			Self::Variable(_) => Variable::<I>::TYPENAME,

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.typename(),
		}
	}
}

impl<'e, I: IntType> ToBoolean<'e, I> for Value<'e, I> {
	fn to_boolean(&self, env: &mut Environment<'e, I>) -> Result<Boolean> {
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

impl<'e, I: IntType> ToInteger<'e, I> for Value<'e, I> {
	fn to_integer(&self, env: &mut Environment<'e, I>) -> Result<Integer<I>> {
		match *self {
			Self::Null => Null.to_integer(env),
			Self::Boolean(boolean) => boolean.to_integer(env),
			Self::Integer(integer) => integer.to_integer(env),
			Self::Text(ref text) => text.to_integer(env),
			Self::List(ref list) => list.to_integer(env),
			#[cfg(feature = "custom-types")]
			Self::Custom(ref custom) => custom.to_integer(env),

			_ => Err(Error::NoConversion { to: Integer::<I>::TYPENAME, from: self.typename() }),
		}
	}
}

impl<'e, I: IntType> ToText<'e, I> for Value<'e, I> {
	fn to_text(&self, env: &mut Environment<'e, I>) -> Result<Text> {
		match *self {
			Self::Null => Null.to_text(env),
			Self::Boolean(boolean) => boolean.to_text(env),
			Self::Integer(integer) => integer.to_text(env),
			Self::Text(ref text) => text.to_text(env),
			Self::List(ref list) => list.to_text(env),

			#[cfg(feature = "custom-types")]
			Self::Custom(ref custom) => custom.to_text(env),

			_ => Err(Error::NoConversion { to: Text::TYPENAME, from: self.typename() }),
		}
	}
}

impl<'e, I: IntType> ToList<'e, I> for Value<'e, I> {
	fn to_list(&self, env: &mut Environment<'e, I>) -> Result<List<'e, I>> {
		match *self {
			Self::Null => Null.to_list(env),
			Self::Boolean(boolean) => boolean.to_list(env),
			Self::Integer(integer) => integer.to_list(env),
			Self::Text(ref text) => text.to_list(env),
			Self::List(ref list) => list.to_list(env),

			#[cfg(feature = "custom-types")]
			Self::Custom(ref custom) => custom.to_list(env),

			_ => Err(Error::NoConversion { to: List::<I>::TYPENAME, from: self.typename() }),
		}
	}
}

impl<'e, I: IntType> Runnable<'e, I> for Value<'e, I> {
	fn run(&self, env: &mut Environment<'e, I>) -> Result<Self> {
		match self {
			Self::Variable(variable) => variable.run(env),
			Self::Ast(ast) => ast.run(env),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.run(env),

			_ => Ok(self.clone()),
		}
	}
}

impl<'e, I: IntType> Value<'e, I> {
	pub fn call(&self, env: &mut Environment<'e, I>) -> Result<Self> {
		// When ensuring that `CALL` is only given values returned from `BLOCK`, we must ensure that all
		// arguments are `Value::Ast`s.
		#[cfg(feature = "compliance")]
		if env.flags().compliance.check_call_arg && !matches!(self, Value::Ast(_)) {
			return Err(Error::TypeError(self.typename(), "CALL"));
		}

		self.run(env)
	}

	pub fn head(&self, env: &mut Environment<'e, I>) -> Result<Self> {
		let _ = env;
		match self {
			Self::List(list) => list.head().ok_or(Error::DomainError("empty list")),
			Self::Text(text) => text.head().ok_or(Error::DomainError("empty text")).map(Self::from),

			#[cfg(feature = "extensions")]
			Self::Integer(integer) if env.flags().exts.tys.integer => Ok(integer.head().into()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.head(env),

			other => Err(Error::TypeError(other.typename(), "[")),
		}
	}

	pub fn tail(&self, env: &mut Environment<'e, I>) -> Result<Self> {
		let _ = env;
		match self {
			Self::List(list) => list.tail().ok_or(Error::DomainError("empty list")).map(Self::from),
			Self::Text(text) => text.tail().ok_or(Error::DomainError("empty text")).map(Self::from),

			#[cfg(feature = "extensions")]
			Self::Integer(integer) if env.flags().exts.tys.integer => Ok(integer.tail().into()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.tail(env),

			other => Err(Error::TypeError(other.typename(), "]")),
		}
	}

	pub fn length(&self, env: &mut Environment<'e, I>) -> Result<Self> {
		match self {
			Self::List(list) => Integer::try_from(list.len()).map(Self::from),
			Self::Text(text) => {
				debug_assert_eq!(text.len(), self.to_list(env).unwrap().len());
				Integer::try_from(text.len()).map(Self::from)
			}
			Self::Integer(int) if int.is_zero() => Ok(Integer::ONE.into()),
			Self::Integer(int) => Integer::try_from(int.log10()).map(Self::from),
			Self::Boolean(true) => Ok(Integer::ONE.into()),
			Self::Boolean(false) | Self::Null => Ok(Integer::ZERO.into()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.length(env),

			other => Err(Error::TypeError(other.typename(), "LENGTH")),
		}
	}

	pub fn ascii(&self, env: &mut Environment<'e, I>) -> Result<Self> {
		let _ = env;
		match self {
			Self::Integer(integer) => Ok(integer.chr(env.flags())?.into()),
			Self::Text(text) => Ok(text.ord()?.into()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.ascii(env),

			other => Err(Error::TypeError(other.typename(), "ASCII")),
		}
	}

	pub fn add(&self, rhs: &Self, env: &mut Environment<'e, I>) -> Result<Self> {
		match self {
			Self::Integer(integer) => integer.add(rhs.to_integer(env)?).map(Self::from),
			Self::Text(string) => Ok(string.concat(&rhs.to_text(env)?, env.flags())?.into()),
			Self::List(list) => list.concat(&rhs.to_list(env)?, env.flags()).map(Self::from),

			#[cfg(feature = "extensions")]
			Self::Boolean(lhs) if env.flags().exts.tys.boolean => Ok((lhs | rhs.to_boolean(env)?).into()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.add(rhs, env),

			other => Err(Error::TypeError(other.typename(), "+")),
		}
	}

	pub fn subtract(&self, rhs: &Self, env: &mut Environment<'e, I>) -> Result<Self> {
		match self {
			Self::Integer(integer) => integer.subtract(rhs.to_integer(env)?).map(Self::from),

			#[cfg(feature = "extensions")]
			Self::Text(text) if env.flags().exts.tys.text => {
				Ok(text.remove_substr(&rhs.to_text(env)?).into())
			}

			#[cfg(feature = "extensions")]
			Self::List(list) if env.flags().exts.tys.list => {
				list.difference(&rhs.to_list(env)?).map(Self::from)
			}

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.subtract(rhs, env),

			other => Err(Error::TypeError(other.typename(), "-")),
		}
	}

	pub fn multiply(&self, rhs: &Self, env: &mut Environment<'e, I>) -> Result<Self> {
		match self {
			Self::Integer(integer) => integer.multiply(rhs.to_integer(env)?).map(Self::from),

			Self::Text(lstr) => {
				let amount = usize::try_from(rhs.to_integer(env)?)
					.or(Err(Error::DomainError("repetition count is negative")))?;

				if isize::MAX as usize <= amount * lstr.len() {
					return Err(Error::DomainError("repetition is too large"));
				}

				Ok(lstr.repeat(amount, env.flags())?.into())
			}

			Self::List(list) => {
				let rhs = rhs;

				// Multiplying by a block is invalid, so we can do this as an extension.
				#[cfg(feature = "extensions")]
				if env.flags().exts.tys.list && matches!(rhs, Self::Ast(_)) {
					return list.map(rhs, env).map(Self::from);
				}

				let amount = usize::try_from(rhs.to_integer(env)?)
					.or(Err(Error::DomainError("repetition count is negative")))?;

				// No need to check for repetition length because `list.repeat` doesnt actually
				// make a list.

				list.repeat(amount, env.flags()).map(Self::from)
			}

			#[cfg(feature = "extensions")]
			Self::Boolean(lhs) if env.flags().exts.tys.boolean => Ok((lhs & rhs.to_boolean(env)?).into()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.multiply(rhs, env),

			other => Err(Error::TypeError(other.typename(), "*")),
		}
	}

	pub fn divide(&self, rhs: &Self, env: &mut Environment<'e, I>) -> Result<Self> {
		match self {
			Self::Integer(integer) => integer.divide(rhs.to_integer(env)?).map(Self::from),

			#[cfg(feature = "extensions")]
			Self::Text(text) if env.flags().exts.tys.text => Ok(text.split(&rhs.to_text(env)?, env).into()),

			#[cfg(feature = "extensions")]
			Self::List(list) if env.flags().exts.tys.list => Ok(list.reduce(rhs, env)?.unwrap_or_default()),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.divide(rhs, env),

			other => Err(Error::TypeError(other.typename(), "/")),
		}
	}

	pub fn remainder(&self, rhs: &Self, env: &mut Environment<'e, I>) -> Result<Self> {
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
			Self::List(list) if env.flags().exts.tys.list => list.filter(rhs, env).map(Self::from),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.remainder(rhs, env),

			other => Err(Error::TypeError(other.typename(), "%")),
		}
	}

	pub fn power(&self, rhs: &Self, env: &mut Environment<'e, I>) -> Result<Self> {
		match self {
			Self::Integer(integer) => integer.power(rhs.to_integer(env)?, env.flags()).map(Self::from),
			Self::List(list) => list.join(&rhs.to_text(env)?, env).map(Self::from),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.power(rhs, env),

			other => Err(Error::TypeError(other.typename(), "^")),
		}
	}
}

impl<'e, I: IntType> Value<'e, I> {
	pub fn compare(&self, rhs: &Self, env: &mut Environment<'e, I>) -> Result<Ordering> {
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

	pub fn equals(&self, rhs: &Self, env: &mut Environment<'_, I>) -> Result<bool> {
		#[cfg(feature = "compliance")]
		{
			fn check_for_strict_compliance<I: IntType>(value: &Value<'_, I>) -> Result<()> {
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

	pub fn assign(&self, value: Self, env: &mut Environment<'e, I>) -> Result<()> {
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
				if env.flags().exts.assign_to.prompt && ast.function().full_name() == "PROMPT" =>
			{
				match value {
					// `= PROMPT NULL` or `= PROMPT FALSE` makes it always return nothing.
					Value::Null | Value::Boolean(false) => env.prompt().close(),

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
				if env.flags().exts.assign_to.prompt && ast.function().full_name() == "$" =>
			{
				let lines = value.to_text(env)?;
				env.add_to_system(lines);
				return Ok(());
			}

			other => match other.run(env)? {
				Value::List(_list) if env.flags().exts.assign_to.list => todo!(),
				Value::Text(name) if env.flags().exts.assign_to.text => {
					env.lookup(&name)?.assign(value);
					return Ok(());
				}
				_ => { /* fallthrough */ }
			},
		}

		Err(Error::TypeError(self.typename(), "="))
	}

	pub fn get(&self, start: &Self, len: &Self, env: &mut Environment<'e, I>) -> Result<Self> {
		let start = fix_len(self, start.to_integer(env)?, env)?;
		let len =
			usize::try_from(len.to_integer(env)?).or(Err(Error::DomainError("negative length")))?;

		match self {
			Self::List(list) => list
				.get(start..start + len)
				.ok_or(Error::IndexOutOfBounds { len: list.len(), index: start + len })
				.map(Self::from),

			Self::Text(text) => text
				.get(start..start + len)
				.ok_or(Error::IndexOutOfBounds { len: text.len(), index: start + len })
				.map(ToOwned::to_owned)
				.map(Self::from),

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.get(start, len, env),

			other => return Err(Error::TypeError(other.typename(), "GET")),
		}
	}

	pub fn set(
		&self,
		start: &Self,
		len: &Self,
		replacement: &Self,
		env: &mut Environment<'e, I>,
	) -> Result<Self> {
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

			#[cfg(feature = "custom-types")]
			Self::Custom(custom) => custom.set(start, len, replacement, env),

			other => return Err(Error::TypeError(other.typename(), "SET")),
		}
	}
}

fn fix_len<'e, I: IntType>(
	#[cfg_attr(not(feature = "extensions"), allow(unused))] container: &Value<'e, I>,
	#[cfg_attr(not(feature = "extensions"), allow(unused_mut))] mut start: Integer<I>,
	#[cfg_attr(not(feature = "extensions"), allow(unused))] env: &mut Environment<'e, I>,
) -> Result<usize> {
	#[cfg(feature = "extensions")]
	if env.flags().exts.negative_indexing && start.is_negative() {
		let len = match container {
			Value::Text(text) => text.len(),
			Value::List(list) => list.len(),

			#[cfg(feature = "custom-types")]
			Value::Custom(list) => list.length(env)?.to_integer(env)?.try_into()?,

			other => return Err(Error::TypeError(other.typename(), "get/set")),
		};

		start = start.add(len.try_into()?)?;
	}

	usize::try_from(start).or(Err(Error::DomainError("negative start position")))
}

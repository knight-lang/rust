#![allow(non_snake_case)]

use crate::env::Options;
use crate::value::text::{Character, Encoding, TextSlice};
use crate::value::{Integer, List, Runnable, Text, ToBoolean, ToInteger, ToList, ToText};
use crate::{Environment, Error, Result, Value};
use std::collections::HashMap;
use std::sync::Arc;

use std::fmt::{self, Debug, Formatter};
use tap::prelude::*;

// A function in Knight
#[derive(Clone)]
pub struct Function<'e, E>(Arc<Inner<'e, E>>);

struct Inner<'e, E> {
	/// The code associated with this function
	func: fn(&[Value<'e, E>], &mut Environment<'e, E>) -> Result<Value<'e, E>>,

	/// The long-hand name of this function.
	///
	/// For extension functions that start with `X`, this should also start with it.
	name: Text<E>,

	/// The arity of the function.
	arity: usize,
}

impl<'e, E> Function<'e, E> {
	pub fn new(
		name: Text<E>,
		arity: usize,
		func: fn(&[Value<'e, E>], &mut Environment<'e, E>) -> Result<Value<'e, E>>,
	) -> Self {
		Self(Arc::new(Inner { name, arity, func }))
	}

	pub fn short_form(&self) -> Option<Character<E>> {
		match self.name().chars().next() {
			Some(c) if c == 'X' => None,
			other => other,
		}
	}

	pub fn name(&self) -> &TextSlice<E> {
		&self.0.name
	}

	pub fn arity(&self) -> usize {
		self.0.arity
	}

	pub fn call(&self, args: &[Value<'e, E>], env: &mut Environment<'e, E>) -> Result<Value<'e, E>> {
		(self.0.func)(args, env)
	}
}

impl<E> Eq for Function<'_, E> {}
impl<E> PartialEq for Function<'_, E> {
	fn eq(&self, rhs: &Self) -> bool {
		self.name() == rhs.name()
	}
}

impl<E> Debug for Function<'_, E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Function({})", self.name())
	}
}

pub fn default<'e, E: Encoding + 'e>(options: &Options) -> HashMap<Character<E>, Function<'e, E>> {
	let mut map = HashMap::new();

	macro_rules! insert {
		($($name:ident $(($ext_name:ident))?)*) => {
			$(
				{
					#[allow(unused_mut)]
					let mut insert = true;
					$(if !options.spec_extensions.$ext_name { insert = false })?
					if insert {
						let func = $name();
						map.insert(func.short_form().unwrap(), func);
					}
				}
			)*
		}
	}

	insert! {
		PROMPT RANDOM
		BLOCK CALL QUIT NOT NEG LENGTH DUMP OUTPUT ASCII BOX HEAD TAIL
		ADD SUBTRACT MULTIPLY DIVIDE MODULO POWER EQUALS LESS_THAN GREATER_THAN AND OR
			THEN ASSIGN WHILE
		IF GET SET

		VALUE (value_fn)
		EVAL (eval_fn)
		HANDLE (handle_fn)
		YEET (yeet_fn)
		USE (use_fn)
		SYSTEM (system_fn)
	}

	map
}

pub fn extensions<'e, E: Encoding + 'e>(options: &Options) -> HashMap<Text<E>, Function<'e, E>> {
	#[allow(unused_mut)]
	let mut map = HashMap::new();

	macro_rules! insert {
		($($name:ident ($feature:ident))*) => {
			$(
				if options.compiler.$feature {
					let func = $name();
					map.insert(func.name().to_owned(), func);
				}
			)*
		}
	}

	insert! {
		XSRAND (srand_fn)
		XREVERSE (range_fn)
		XRANGE (reverse_fn)
	}

	map
}

macro_rules! arity {
	() => (0);
	($_pat:ident $($rest:ident)*) => (1+arity!($($rest)*))
}
macro_rules! function {
	($name:literal, $env:pat, |$($args:ident),*| $body:expr) => {
		Function::new(
			unsafe { TextSlice::new_unchecked($name) }.to_owned(),
			arity!($($args)*),
			|args, $env| {
				let [$($args,)*]: &[Value<'_, _>; arity!($($args)*)] = args.try_into().unwrap();
				Ok($body.into())
			}
		)
	};
}

/// **4.1.4**: `PROMPT`
pub fn PROMPT<'e, E: Encoding>() -> Function<'e, E> {
	function!("PROMPT", env, |/* comment for rustfmt */| {
	if env.options().compiler.assign_to_prompt {
		if let Some(line) = env.get_next_prompt_line() {
			return Ok(line.into());
		}
	}

	let mut buf = String::new();

	// If we read an empty line, return null.
	if env.stdin().read_line(&mut buf)? == 0 {
		return Ok(Value::Null);
	}

	// remove trailing newlines
	match buf.pop() {
		Some('\n') => match buf.pop() {
			Some('\r') => {}
			Some(other) => buf.push(other), // ie `<anything>\n`
			None => {}
		},
		Some(other) => buf.push(other),
		None => {}
	}

	buf.try_conv::<Text<E>>()?
})
}

/// **4.1.5**: `RANDOM`
pub fn RANDOM<'e, E>() -> Function<'e, E> {
	function!("RANDOM", env, |/* comment for rustfmt */| {
	// note that `env.random()` is seedable with `XSRAND`
	env.random()
})
}

/// **4.2.2** `BOX`
pub fn BOX<'e, E>() -> Function<'e, E> {
	function!(",", env, |val| {
		// `boxed` is optimized over `vec![val.run(env)]`
		List::boxed(val.run(env)?)
	})
}

pub fn HEAD<'e, E: 'e>() -> Function<'e, E> {
	function!("[", env, |val| {
		match val.run(env)? {
			Value::List(list) => list.head().ok_or(Error::DomainError("empty list"))?,
			Value::Text(text) => text.head().ok_or(Error::DomainError("empty text"))?.into(),
			other => return Err(Error::TypeError(other.typename(), "[")),
		}
	})
}

pub fn TAIL<'e, E: 'e>() -> Function<'e, E> {
	function!("]", env, |val| {
		match val.run(env)? {
			Value::List(list) => {
				list.tail().ok_or(Error::DomainError("empty list"))?.conv::<Value<'_, E>>()
			}
			Value::Text(text) => text.tail().ok_or(Error::DomainError("empty text"))?.into(),
			other => return Err(Error::TypeError(other.typename(), "]")),
		}
	})
}

// The NOOP function literally just runs its argument.
pub fn NOOP<'e, E>() -> Function<'e, E> {
	function!(":", env, |arg| {
		debug_assert!(!matches!(arg, Value::Ast(_)));

		arg.run(env)? // We can't `.clone()` the arg in case we're given a variable name.
	})
}

/// **4.2.3** `BLOCK`  
pub fn BLOCK<'e, E>() -> Function<'e, E> {
	function!("BLOCK", env, |arg| {
		// Technically, according to the spec, only the return value from `BLOCK` can be used in `CALL`.
		// Since this function normally just returns whatever it's argument is, it's impossible to
		// distinguish an `Integer` returned from `BLOCK` and one simply given to `CALL`. As such, when
		// the `strict-call-argument` feature is enabled. We ensure that we _only_ return `Ast`s
		// from `BLOCK`, so `CALL` can verify them.
		if env.options().compliance.call_argument && !matches!(arg, Value::Ast(_)) {
			return Ok(crate::Ast::new(NOOP(), vec![arg.clone()].into()).into());
		}

		let _ = env;

		arg.clone()
	})
}

/// **4.2.4** `CALL`  
pub fn CALL<'e, E>() -> Function<'e, E> {
	function!("CALL", env, |arg| {
		let block = arg.run(env)?;

		// When ensuring that `CALL` is only given values returned from `BLOCK`, we must ensure that all
		// arguments are `Value::Ast`s.
		if env.options().compliance.call_argument && !matches!(block, Value::Ast(_)) {
			return Err(Error::TypeError(block.typename(), "CALL"));
		}

		block.run(env)?
	})
}

/// **4.2.6** `QUIT`  
pub fn QUIT<'e, E>() -> Function<'e, E> {
	function!("QUIT", env, |arg| {
		match arg.run(env)?.to_integer(env.options())?.try_conv::<i32>() {
			Ok(status)
				if !env.options().compliance.check_quit_argument || (0..=127).contains(&status) =>
			{
				return Err(Error::Quit(status))
			}
			_ => return Err(Error::DomainError("exit code out of bounds")),
		}

		// The `function!` macro calls `.into()` on the return value of this block,
		// so we need _something_ here so it can typecheck correctly.
		#[allow(unreachable_code)]
		Value::Null
	})
}

/// **4.2.7** `!`  
pub fn NOT<'e, E>() -> Function<'e, E> {
	function!("!", env, |arg| {
		// <blank line so rustfmt doesnt wrap onto the prev line>
		!arg.run(env)?.to_boolean(env.options())?
	})
}

/// **4.2.8** `LENGTH`  
pub fn LENGTH<'e, E>() -> Function<'e, E> {
	function!("LENGTH", env, |arg| {
		match arg.run(env)? {
			Value::Text(text) => {
				debug_assert_eq!(
					text.len(),
					Value::Text(text.clone()).to_list(env.options()).unwrap().len()
				);
				text.len().try_conv::<Integer>()?
			}
			Value::List(list) => list.len().try_conv::<Integer>()?,
			// TODO: integer base10 when that comes out.
			other => other.to_list(env.options())?.len().try_conv::<Integer>()?,
		}
	})
}

/// **4.2.9** `DUMP`  
pub fn DUMP<'e, E>() -> Function<'e, E> {
	function!("DUMP", env, |arg| {
		let value = arg.run(env)?;
		writeln!(env.stdout(), "{value:?}")?;
		value
	})
}

/// **4.2.10** `OUTPUT`  
pub fn OUTPUT<'e, E: Encoding>() -> Function<'e, E> {
	function!("OUTPUT", env, |arg| {
		let text = arg.run(env)?.to_text(env.options())?;
		let stdout = env.stdout();

		if let Some(stripped) = text.strip_suffix('\\') {
			write!(stdout, "{stripped}")?
		} else {
			writeln!(stdout, "{text}")?;
		}

		stdout.flush()?;

		Value::Null
	})
}

/// **4.2.11** `ASCII`  
pub fn ASCII<'e, E: Encoding>() -> Function<'e, E> {
	function!("ASCII", env, |arg| {
		match arg.run(env)? {
			Value::Integer(integer) => integer.chr()?.conv::<Value<'_, E>>(),
			Value::Text(text) => text.ord()?.conv::<Value<'_, E>>(),
			other => return Err(Error::TypeError(other.typename(), "ASCII")),
		}
	})
}

/// **4.2.12** `~`  
pub fn NEG<'e, E>() -> Function<'e, E> {
	function!("~", env, |arg| {
		// comment so it wont make it one line
		arg.run(env)?.to_integer(env.options())?.negate(env.options())?
	})
}

/// **4.3.1** `+`  
pub fn ADD<'e, E: Encoding>() -> Function<'e, E> {
	function!("+", env, |lhs, rhs| {
		match lhs.run(env)? {
			Value::Integer(integer) => integer
				.add(rhs.run(env)?.to_integer(env.options())?, env.options())?
				.conv::<Value<'_, E>>(),
			Value::Text(string) => string.concat(&rhs.run(env)?.to_text(env.options())?).into(),
			Value::List(list) => {
				list.concat(&rhs.run(env)?.to_list(env.options())?, env.options())?.into()
			}

			other => return Err(Error::TypeError(other.typename(), "+")),
		}
	})
}

/// **4.3.2** `-`  
pub fn SUBTRACT<'e, E>() -> Function<'e, E> {
	function!("-", env, |lhs, rhs| {
		match lhs.run(env)? {
			Value::Integer(integer) => integer
				.subtract(rhs.run(env)?.to_integer(env.options())?, env.options())?
				.conv::<Value<'_, E>>(),

			Value::List(list) if env.options().compiler.list_extensions => {
				list.difference(&rhs.run(env)?.to_list(env.options())?)?.into()
			}

			other => return Err(Error::TypeError(other.typename(), "-")),
		}
	})
}

/// **4.3.3** `*`  
pub fn MULTIPLY<'e, E: Encoding>() -> Function<'e, E> {
	function!("*", env, |lhs, rhs| {
		match lhs.run(env)? {
			Value::Integer(integer) => integer
				.multiply(rhs.run(env)?.to_integer(env.options())?, env.options())?
				.conv::<Value<'_, E>>(),

			Value::Text(lstr) => {
				let amount = rhs
					.run(env)?
					.to_integer(env.options())?
					.try_conv::<usize>()
					.or(Err(Error::DomainError("repetition count is negative")))?;

				if isize::MAX as usize <= amount * lstr.len() {
					return Err(Error::DomainError("repetition is too large"));
				}

				lstr.repeat(amount).conv::<Value<'_, E>>()
			}

			Value::List(list) => {
				let rhs = rhs.run(env)?;

				if env.options().compiler.list_extensions && matches!(rhs, Value::Ast(_)) {
					return Ok(list.map(&rhs, env)?.into());
				}

				let amount = rhs
					.to_integer(env.options())?
					.try_conv::<usize>()
					.or(Err(Error::DomainError("repetition count is negative")))?;

				// No need to check for repetition length because `list.repeat` doesnt actually
				// make a list.
				list.repeat(amount, env.options())?.conv::<Value<'_, E>>()
			}

			other => return Err(Error::TypeError(other.typename(), "*")),
		}
	})
}

/// **4.3.4** `/`  
pub fn DIVIDE<'e, E: Encoding>() -> Function<'e, E> {
	function!("/", env, |lhs, rhs| {
		match lhs.run(env)? {
			Value::Integer(integer) => integer
				.divide(rhs.run(env)?.to_integer(env.options())?, env.options())?
				.conv::<Value<'_, E>>(),

			Value::Text(text) if env.options().compiler.string_extensions => {
				text.split(&rhs.run(env)?.to_text(env.options())?, env.options()).into()
			}
			Value::List(list) if env.options().compiler.list_extensions => {
				list.reduce(&rhs.run(env)?, env)?.unwrap_or_default()
			}

			other => return Err(Error::TypeError(other.typename(), "/")),
		}
	})
}

/// **4.3.5** `%`  
pub fn MODULO<'e, E: Encoding>() -> Function<'e, E> {
	function!("%", env, |lhs, rhs| {
		match lhs.run(env)? {
			Value::Integer(integer) => integer
				.modulo(rhs.run(env)?.to_integer(env.options())?, env.options())?
				.conv::<Value<'_, E>>(),

			// #[cfg(feature = "string-extensions")]
			// Value::Text(lstr) => {
			// 	let values = rhs.run(env)?.to_list(env.options())?;
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
			// 						.to_text(env.options())?,
			// 				);
			// 				values_index += 1;
			// 			}
			// 			_ => formatted.push(chr),
			// 		}
			// 	}

			// 	Text::new(formatted).unwrap().into()
			// }
			Value::List(list) if env.options().compiler.list_extensions => {
				list.filter(&rhs.run(env)?, env)?.into()
			}

			other => return Err(Error::TypeError(other.typename(), "%")),
		}
	})
}

/// **4.3.6** `^`  
pub fn POWER<'e, E: Encoding>() -> Function<'e, E> {
	function!("^", env, |lhs, rhs| {
		match lhs.run(env)? {
			Value::Integer(integer) => integer
				.power(rhs.run(env)?.to_integer(env.options())?, env.options())?
				.conv::<Value<'_, E>>(),
			Value::List(list) => {
				list.join(&rhs.run(env)?.to_text(env.options())?, env.options())?.into()
			}
			other => return Err(Error::TypeError(other.typename(), "^")),
		}
	})
}

fn compare<E: Encoding>(
	lhs: &Value<E>,
	rhs: &Value<E>,
	opts: &Options,
) -> Result<std::cmp::Ordering> {
	match lhs {
		Value::Integer(lnum) => Ok(lnum.cmp(&rhs.to_integer(opts)?)),
		Value::Boolean(lbool) => Ok(lbool.cmp(&rhs.to_boolean(opts)?)),
		Value::Text(ltext) => Ok(ltext.cmp(&rhs.to_text(opts)?)),
		Value::List(list) => {
			let rhs = rhs.to_list(opts)?;

			// feels bad to be iterating over by-values.
			for (left, right) in list.iter().zip(&rhs) {
				match compare(left, right, opts)? {
					std::cmp::Ordering::Equal => {}
					other => return Ok(other),
				}
			}

			Ok(list.len().cmp(&rhs.len()))
		}
		other => Err(Error::TypeError(other.typename(), "<cmp>")),
	}
}

/// **4.3.7** `<`  
pub fn LESS_THAN<'e, E: Encoding>() -> Function<'e, E> {
	function!("<", env, |lhs, rhs| {
		compare(&lhs.run(env)?, &rhs.run(env)?, env.options())? == std::cmp::Ordering::Less
	})
}

/// **4.3.8** `>`  
pub fn GREATER_THAN<'e, E: Encoding>() -> Function<'e, E> {
	function!(">", env, |lhs, rhs| {
		compare(&lhs.run(env)?, &rhs.run(env)?, env.options())? == std::cmp::Ordering::Greater
	})
}

/// **4.3.9** `?`  
pub fn EQUALS<'e, E>() -> Function<'e, E> {
	function!("?", env, |lhs, rhs| {
		fn check_for_strict_compliance<E>(value: &Value<'_, E>) -> Result<()> {
			match value {
				Value::List(list) => {
					for ele in list {
						check_for_strict_compliance(&ele)?;
					}
					Ok(())
				}
				Value::Ast(_) | Value::Variable(_) => Err(Error::TypeError(value.typename(), "?")),
				_ => Ok(()),
			}
		}

		let l = lhs.run(env)?;
		let r = rhs.run(env)?;

		if env.options().compliance.strict_equality {
			check_for_strict_compliance(&l)?;
			check_for_strict_compliance(&r)?;
		}

		l == r
	})
}

/// **4.3.10** `&`  
pub fn AND<'e, E>() -> Function<'e, E> {
	function!("&", env, |lhs, rhs| {
		let l = lhs.run(env)?;

		if l.to_boolean(env.options())? {
			rhs.run(env)?
		} else {
			l
		}
	})
}

/// **4.3.11** `|`  
pub fn OR<'e, E>() -> Function<'e, E> {
	function!("|", env, |lhs, rhs| {
		let l = lhs.run(env)?;

		if l.to_boolean(env.options())? {
			l
		} else {
			rhs.run(env)?
		}
	})
}

/// **4.3.12** `;`  
pub fn THEN<'e, E>() -> Function<'e, E> {
	function!(";", env, |lhs, rhs| {
		lhs.run(env)?;
		rhs.run(env)?
	})
}

fn assign<'e, E: Encoding>(
	variable: &Value<'e, E>,
	value: Value<'e, E>,
	env: &mut Environment<'e, E>,
) -> Result<()> {
	match variable {
		Value::Variable(var) => {
			var.assign(value);
		}

		Value::Ast(ast) if env.options().compiler.assign_to_prompt && *ast.function() == PROMPT() => {
			env.add_to_prompt(value.to_text(env.options())?)
		}

		Value::Ast(ast) if env.options().compiler.assign_to_system && *ast.function() == SYSTEM() => {
			env.add_to_system(value.to_text(env.options())?)
		}

		other
			if !env.options().spec_extensions.assign_to_string
				&& !env.options().spec_extensions.assign_to_list =>
		{
			return Err(Error::TypeError(other.typename(), "="))
		}

		other => match other.run(env)? {
			Value::List(_list) if env.options().spec_extensions.assign_to_list => todo!(),

			Value::Text(name) if env.options().spec_extensions.assign_to_string => {
				env.lookup(&name)?.assign(value);
			}

			other => return Err(Error::TypeError(other.typename(), "=")),
		},
	}

	let _ = env;

	Ok(())
}

/// **4.3.13** `=`  
pub fn ASSIGN<'e, E: Encoding>() -> Function<'e, E> {
	function!("=", env, |var, value| {
		let ret = value.run(env)?;
		assign(var, ret.clone(), env)?;
		ret
	})
}

/// **4.3.14** `WHILE`  
pub fn WHILE<'e, E>() -> Function<'e, E> {
	function!("WHILE", env, |cond, body| {
		while cond.run(env)?.to_boolean(env.options())? {
			body.run(env)?;
		}

		Value::Null
	})
}

/// **4.4.1** `IF`  
pub fn IF<'e, E>() -> Function<'e, E> {
	function!("IF", env, |cond, iftrue, iffalse| {
		if cond.run(env)?.to_boolean(env.options())? {
			iftrue.run(env)?
		} else {
			iffalse.run(env)?
		}
	})
}

fn fix_len<E>(container: &Value<'_, E>, mut start: Integer, options: &Options) -> Result<usize> {
	if start.is_negative() && options.compiler.negative_indexing {
		let len = match container {
			Value::Text(text) => text.len(),
			Value::List(list) => list.len(),
			other => return Err(Error::TypeError(other.typename(), "get/set")),
		};

		start = start.add(len.try_into()?, options)?;
	}

	start.try_conv::<usize>().or(Err(Error::DomainError("negative start position")))
}

/// **4.4.2** `GET`  
pub fn GET<'e, E>() -> Function<'e, E> {
	function!("GET", env, |string, start, length| {
		let source = string.run(env)?;
		let start = fix_len(&source, start.run(env)?.to_integer(env.options())?, env.options())?;
		let length = length
			.run(env)?
			.to_integer(env.options())?
			.try_conv::<usize>()
			.or(Err(Error::DomainError("negative length")))?;

		match source {
			Value::List(list) => list
				.get(start..start + length)
				.ok_or_else(|| Error::IndexOutOfBounds { len: list.len(), index: start + length })?
				.conv::<Value<'_, E>>(),
			Value::Text(text) => text
				.get(start..start + length)
				.ok_or_else(|| Error::IndexOutOfBounds { len: text.len(), index: start + length })?
				.to_owned()
				.into(),
			other => return Err(Error::TypeError(other.typename(), "GET")),
		}
	})
}

/// **4.5.1** `SET`  
pub fn SET<'e, E: Encoding>() -> Function<'e, E> {
	function!("SET", env, |string, start, length, replacement| {
		let source = string.run(env)?;
		let start = fix_len(&source, start.run(env)?.to_integer(env.options())?, env.options())?;
		let length = length
			.run(env)?
			.to_integer(env.options())?
			.try_conv::<usize>()
			.or(Err(Error::DomainError("negative length")))?;
		let replacement_source = replacement.run(env)?;

		match source {
			Value::List(list) => {
				// FIXME: cons?

				let replacement = replacement_source.to_list(env.options())?;
				let mut ret = Vec::new();
				ret.extend(list.iter().take(start).cloned());
				ret.extend(replacement.iter().cloned());
				ret.extend(list.iter().skip((start) + length).cloned());

				List::try_from(ret)?.conv::<Value<'_, E>>()
			}
			Value::Text(text) => {
				let replacement = replacement_source.to_text(env.options())?;

				// lol, todo, optimize me
				let mut builder = Text::builder();
				builder.push(text.get(..start).unwrap());
				builder.push(&replacement);
				builder.push(text.get(start + length..).unwrap());
				builder.finish().into()
			}
			other => return Err(Error::TypeError(other.typename(), "SET")),
		}
	})
}

/// **6.1** `VALUE`
pub fn VALUE<'e, E: Encoding>() -> Function<'e, E> {
	function!("VALUE", env, |arg| {
		let name = arg.run(env)?.to_text(env.options())?;
		env.lookup(&name)?
	})
}

pub fn HANDLE<'e, E: Encoding>() -> Function<'e, E> {
	function!("HANDLE", env, |block, iferr| {
		let ERR_VAR_NAME = unsafe { TextSlice::new_unchecked("_") };

		match block.run(env) {
			Ok(value) => value,
			Err(err) => {
				// This is fallible, as the error string might have had something bad.
				let errmsg = err.to_string().try_conv::<Text<E>>()?;

				// Assign it to the error variable
				env.lookup(ERR_VAR_NAME).unwrap().assign(errmsg.into());

				// Finally, execute the RHS.
				iferr.run(env)?
			}
		}
	})
}

pub fn YEET<'e, E: Encoding>() -> Function<'e, E> {
	function!("YEET", env, |errmsg| {
		return Err(Error::Custom(errmsg.run(env)?.to_text(env.options())?.to_string().into()));
		#[allow(unreachable_code)]
		Value::Null
	})
}

/// **6.3** `USE`
pub fn USE<'e, E: Encoding>() -> Function<'e, E> {
	function!("USE", env, |arg| {
		let filename = arg.run(env)?.to_text(env.options())?;
		let contents = env.read_file(&filename)?;

		env.play(&contents)?
	})
}

/// **4.2.2** `EVAL`
pub fn EVAL<'e, E: Encoding>() -> Function<'e, E> {
	function!("EVAL", env, |val| {
		let code = val.run(env)?.to_text(env.options())?;
		env.play(&code)?
	})
}

/// **4.2.5** `` ` ``
pub fn SYSTEM<'e, E: Encoding>() -> Function<'e, E> {
	function!("$", env, |cmd, stdin| {
		let command = cmd.run(env)?.to_text(env.options())?;
		let stdin = match stdin.run(env)? {
			Value::Text(text) => Some(text),
			Value::Null => None,
			other => return Err(Error::TypeError(other.typename(), "$")),
		};

		env.run_command(&command, stdin.as_deref())?
	})
}

/// **Compiler extension**: SRAND
pub fn XSRAND<'e, E>() -> Function<'e, E> {
	function!("XSRAND", env, |arg| {
		let seed = arg.run(env)?.to_integer(env.options())?;
		env.srand(seed);
		Value::default()
	})
}

/// **Compiler extension**: REV
pub fn XREVERSE<'e, E>() -> Function<'e, E> {
	function!("XREVERSE", env, |arg| {
		let seed = arg.run(env)?.to_integer(env.options())?;
		env.srand(seed);
		Value::default()
	})
}

pub fn XRANGE<'e, E>() -> Function<'e, E> {
	function!("XRANGE", env, |start, stop| {
		match start.run(env)? {
			Value::Integer(start) => {
				let stop = stop.run(env)?.to_integer(env.options())?;

				match start <= stop {
					true => (i64::from(start)..i64::from(stop))
						.map(|x| Value::from(Integer::try_from(x).unwrap()))
						.collect::<Vec<Value<'_, _>>>()
						.try_conv::<List<'_, _>>()
						.expect("todo: out of bounds error")
						.conv::<Value<'_, _>>(),

					// #[cfg(feature = "negative-ranges")]
					// false => (stop..start).map(Value::from).rev().collect::<List>().into(),

					// #[cfg(not(feature = "negative-ranges"))]
					false => return Err(Error::DomainError("start is greater than stop")),
				}
			}

			Value::Text(_text) => {
				// let start = text.get(0).a;
				todo!()
			}

			other => return Err(Error::TypeError(other.typename(), "XRANGE")),
		}
	})
}

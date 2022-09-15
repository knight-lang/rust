use crate::value::text::{Character, TextSlice};
use crate::value::{Integer, List, Runnable, Text, ToBoolean, ToInteger, ToList, ToText};
use crate::{Environment, Error, Result, Value};
use std::collections::HashMap;

use std::fmt::{self, Debug, Formatter};
use tap::prelude::*;

/// A function in knight indicates
#[derive(Clone, Copy)]
pub struct Function {
	/// The code associated with this function
	pub func: for<'e> fn(&[Value<'e>], &mut Environment<'e>) -> Result<Value<'e>>,

	/// The long-hand name of this function.
	///
	/// For extension functions that start with `X`, this should also start with it.
	pub name: &'static TextSlice,

	/// The arity of the function.
	pub arity: usize,
}

impl Function {
	pub fn short_form(&self) -> Option<Character> {
		match self.name.chars().next() {
			Some(c) if c == 'X' => None,
			other => other,
		}
	}
}

impl Eq for Function {}
impl PartialEq for Function {
	fn eq(&self, rhs: &Self) -> bool {
		std::ptr::eq(self, rhs)
	}
}

impl Debug for Function {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Function({})", self.name)
	}
}

pub fn default() -> HashMap<Character, &'static Function> {
	let mut map = HashMap::new();

	macro_rules! insert {
		($($(#[$meta:meta])* $name:ident)*) => {
			$(
				$(#[$meta])*
				map.insert($name.short_form().unwrap(), &$name);
			)*
		}
	}

	insert! {
		PROMPT RANDOM
		BLOCK CALL QUIT NOT NEG LENGTH DUMP OUTPUT ASCII BOX HEAD TAIL
		ADD SUBTRACT MULTIPLY DIVIDE MODULO POWER EQUALS LESS_THAN GREATER_THAN AND OR
			THEN ASSIGN WHILE
		IF GET SET

		#[cfg(feature = "value-function")] VALUE
		#[cfg(feature = "eval-function")] EVAL
		#[cfg(feature = "handle-function")] HANDLE
		#[cfg(feature = "yeet-function")] YEET
		#[cfg(feature = "use-function")] USE
		#[cfg(feature = "system-function")] SYSTEM
	}

	map
}

pub fn extensions() -> HashMap<Text, &'static Function> {
	let mut map = HashMap::new();

	macro_rules! insert {
		($($feature:literal $name:ident)*) => {
			$(
				#[cfg(feature=$feature)]
				map.insert($name.name.try_into().unwrap(), &$name);
			)*
		}
	}

	insert! {
		"xsrand-function" XSRAND
		"xreverse-function" XREVERSE
		"xrange-function" XRANGE
	}

	map
}

macro_rules! arity {
	() => (0);
	($_pat:ident $($rest:ident)*) => (1+arity!($($rest)*))
}
macro_rules! function {
	($name:literal, $env:pat, |$($args:ident),*| $body:expr) => {
		Function {
			name: unsafe { TextSlice::new_unchecked($name) },
			arity: arity!($($args)*),
			func: |args, $env| {
				let [$($args,)*]: &[Value; arity!($($args)*)] = args.try_into().unwrap();
				Ok($body.into())
			}
		}
	};
}
macro_rules! function2 {
	($name:literal, $env:pat, |$($args:ident),*| $body:expr) => {
		Function {
			name: unsafe { TextSlice::new_unchecked($name) },
			arity: arity!($($args)*),
			func: |args, $env| {
				let [$($args,)*]: &[Value; arity!($($args)*)] = args.try_into().unwrap();
				Ok($body)
			}
		}
	};
}

/// **4.1.4**: `PROMPT`
pub const PROMPT: Function = function!("PROMPT", env, |/* comment for rustfmt */| {
	#[cfg(feature = "assign-to-prompt")]
	if let Some(line) = env.get_next_prompt_line() {
		return Ok(line.into());
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

	buf.try_conv::<Text>()?
});

/// **4.1.5**: `RANDOM`
pub const RANDOM: Function = function!("RANDOM", env, |/* comment for rustfmt */| {
	// note that `env.random()` is seedable with `XSRAND`
	env.random()
});

/// **4.2.2** `BOX`
pub const BOX: Function = function!(",", env, |val| {
	// `boxed` is optimized over `vec![val.run(env)]`
	List::boxed(val.run(env)?)
});

pub const HEAD: Function = function2!("[", env, |val| {
	match val.run(env)? {
		Value::List(list) => list.head().ok_or(Error::DomainError("empty list"))?,
		Value::Text(text) => text.head().ok_or(Error::DomainError("empty text"))?.into(),
		other => return Err(Error::TypeError(other.typename(), "[")),
	}
});

pub const TAIL: Function = function!("]", env, |val| {
	match val.run(env)? {
		Value::List(list) => list.tail().ok_or(Error::DomainError("empty list"))?.conv::<Value>(),
		Value::Text(text) => text.tail().ok_or(Error::DomainError("empty text"))?.into(),
		other => return Err(Error::TypeError(other.typename(), "]")),
	}
});

/// **4.2.3** `BLOCK`  
pub const BLOCK: Function = function!("BLOCK", env, |arg| {
	// Technically, according to the spec, only the return value from `BLOCK` can be used in `CALL`.
	// Since this function normally just returns whatever it's argument is, it's impossible to
	// distinguish an `Integer` returned from `BLOCK` and one simply given to `CALL`. As such, when
	// the `strict-call-argument` feature is enabled. We ensure that we _only_ return `Ast`s
	// from `BLOCK`, so `CALL` can verify them.
	#[cfg(feature = "strict-call-argument")]
	if !matches!(arg, Value::Ast(_)) {
		// The NOOP function literally just runs its argument.
		const NOOP: Function = function!(":", env, |arg| {
			debug_assert!(!matches!(arg, Value::Ast(_)));

			arg.run(env)? // We can't `.clone()` the arg in case we're given a variable name.
		});

		return Ok(crate::Ast::new(&NOOP, vec![arg.clone()].into()).into());
	}

	let _ = env;

	arg.clone()
});

/// **4.2.4** `CALL`  
pub const CALL: Function = function!("CALL", env, |arg| {
	let block = arg.run(env)?;

	// When ensuring that `CALL` is only given values returned from `BLOCK`, we must ensure that all
	// arguments are `Value::Ast`s.
	#[cfg(feature = "strict-call-argument")]
	if !matches!(block, Value::Ast(_)) {
		return Err(Error::TypeError(block.typename(), "CALL"));
	}

	block.run(env)?
});

/// **4.2.6** `QUIT`  
pub const QUIT: Function = function!("QUIT", env, |arg| {
	match arg.run(env)?.to_integer()?.try_conv::<i32>() {
		Ok(status) if !cfg!(feature = "strict-compliance") || (0..=127).contains(&status) => {
			return Err(Error::Quit(status))
		}
		_ => return Err(Error::DomainError("exit code out of bounds")),
	}

	// The `function!` macro calls `.into()` on the return value of this block,
	// so we need _something_ here so it can typecheck correctly.
	#[allow(unreachable_code)]
	Value::Null
});

/// **4.2.7** `!`  
pub const NOT: Function = function!("!", env, |arg| {
	// <blank line so rustfmt doesnt wrap onto the prev line>
	!arg.run(env)?.to_boolean()?
});

/// **4.2.8** `LENGTH`  
pub const LENGTH: Function = function!("LENGTH", env, |arg| {
	match arg.run(env)? {
		Value::Text(text) => {
			debug_assert_eq!(text.len(), Value::Text(text.clone()).to_list().unwrap().len());
			text.len().try_conv::<Integer>()?
		}
		Value::List(list) => list.len().try_conv::<Integer>()?,
		// TODO: integer base10 when that comes out.
		other => other.to_list()?.len().try_conv::<Integer>()?,
	}
});

/// **4.2.9** `DUMP`  
pub const DUMP: Function = function!("DUMP", env, |arg| {
	let value = arg.run(env)?;
	writeln!(env.stdout(), "{value:?}")?;
	value
});

/// **4.2.10** `OUTPUT`  
pub const OUTPUT: Function = function!("OUTPUT", env, |arg| {
	let text = arg.run(env)?.to_text()?;
	let stdout = env.stdout();

	if let Some(stripped) = text.strip_suffix('\\') {
		write!(stdout, "{stripped}")?
	} else {
		writeln!(stdout, "{text}")?;
	}

	stdout.flush()?;

	Value::Null
});

/// **4.2.11** `ASCII`  
pub const ASCII: Function = function!("ASCII", env, |arg| {
	match arg.run(env)? {
		Value::Integer(integer) => integer.chr()?.conv::<Value>(),
		Value::Text(text) => text.ord()?.conv::<Value>(),
		other => return Err(Error::TypeError(other.typename(), "ASCII")),
	}
});

/// **4.2.12** `~`  
pub const NEG: Function = function!("~", env, |arg| {
	// comment so it wont make it one line
	arg.run(env)?.to_integer()?.negate()?
});

/// **4.3.1** `+`  
pub const ADD: Function = function!("+", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(integer) => integer.add(rhs.run(env)?.to_integer()?)?.conv::<Value>(),
		Value::Text(string) => string.concat(&rhs.run(env)?.to_text()?).into(),
		Value::List(list) => list.concat(&rhs.run(env)?.to_list()?)?.into(),

		other => return Err(Error::TypeError(other.typename(), "+")),
	}
});

/// **4.3.2** `-`  
pub const SUBTRACT: Function = function!("-", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(integer) => integer.subtract(rhs.run(env)?.to_integer()?)?.conv::<Value>(),

		#[cfg(feature = "list-extensions")]
		Value::List(list) => list.difference(&rhs.run(env)?.to_list()?)?.into(),

		other => return Err(Error::TypeError(other.typename(), "-")),
	}
});

/// **4.3.3** `*`  
pub const MULTIPLY: Function = function!("*", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(integer) => integer.multiply(rhs.run(env)?.to_integer()?)?.conv::<Value>(),

		Value::Text(lstr) => {
			let amount = rhs
				.run(env)?
				.to_integer()?
				.try_conv::<usize>()
				.or(Err(Error::DomainError("repetition count is negative")))?;

			if isize::MAX as usize <= amount * lstr.len() {
				return Err(Error::DomainError("repetition is too large"));
			}

			lstr.repeat(amount).conv::<Value>()
		}

		Value::List(list) => {
			let rhs = rhs.run(env)?;

			#[cfg(feature = "list-extensions")]
			if matches!(rhs, Value::Ast(_)) {
				return Ok(list.map(&rhs, env)?.into());
			}

			let amount = rhs
				.to_integer()?
				.try_conv::<usize>()
				.or(Err(Error::DomainError("repetition count is negative")))?;

			// No need to check for repetition length because `list.repeat` doesnt actually
			// make a list.
			list.repeat(amount)?.conv::<Value>()
		}

		other => return Err(Error::TypeError(other.typename(), "*")),
	}
});

/// **4.3.4** `/`  
pub const DIVIDE: Function = function!("/", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(integer) => integer.divide(rhs.run(env)?.to_integer()?)?.conv::<Value>(),

		#[cfg(feature = "string-extensions")]
		Value::Text(text) => text.split(&rhs.run(env)?.to_text()?).into(),

		#[cfg(feature = "list-extensions")]
		Value::List(list) => list.reduce(&rhs.run(env)?, env)?.unwrap_or_default(),

		other => return Err(Error::TypeError(other.typename(), "/")),
	}
});

/// **4.3.5** `%`  
pub const MODULO: Function = function!("%", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(integer) => integer.modulo(rhs.run(env)?.to_integer()?)?.conv::<Value>(),

		// #[cfg(feature = "string-extensions")]
		// Value::Text(lstr) => {
		// 	let values = rhs.run(env)?.to_list()?;
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
		// 						.to_text()?,
		// 				);
		// 				values_index += 1;
		// 			}
		// 			_ => formatted.push(chr),
		// 		}
		// 	}

		// 	Text::new(formatted).unwrap().into()
		// }
		#[cfg(feature = "list-extensions")]
		Value::List(list) => list.filter(&rhs.run(env)?, env)?.into(),

		other => return Err(Error::TypeError(other.typename(), "%")),
	}
});

/// **4.3.6** `^`  
pub const POWER: Function = function!("^", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(integer) => integer.power(rhs.run(env)?.to_integer()?)?.conv::<Value>(),
		Value::List(list) => list.join(&rhs.run(env)?.to_text()?)?.into(),
		other => return Err(Error::TypeError(other.typename(), "^")),
	}
});

fn compare(lhs: &Value, rhs: &Value) -> Result<std::cmp::Ordering> {
	match lhs {
		Value::Integer(lnum) => Ok(lnum.cmp(&rhs.to_integer()?)),
		Value::Boolean(lbool) => Ok(lbool.cmp(&rhs.to_boolean()?)),
		Value::Text(ltext) => Ok(ltext.cmp(&rhs.to_text()?)),
		Value::List(list) => {
			let rhs = rhs.to_list()?;

			// feels bad to be iterating over by-values.
			for (left, right) in list.iter().zip(&rhs) {
				match compare(left, right)? {
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
pub const LESS_THAN: Function = function!("<", env, |lhs, rhs| {
	compare(&lhs.run(env)?, &rhs.run(env)?)? == std::cmp::Ordering::Less
});

/// **4.3.8** `>`  
pub const GREATER_THAN: Function = function!(">", env, |lhs, rhs| {
	compare(&lhs.run(env)?, &rhs.run(env)?)? == std::cmp::Ordering::Greater
});

/// **4.3.9** `?`  
pub const EQUALS: Function = function!("?", env, |lhs, rhs| {
	fn check_for_strict_compliance(value: &Value<'_>) -> Result<()> {
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

	if cfg!(feature = "strict-compliance") {
		check_for_strict_compliance(&l)?;
		check_for_strict_compliance(&r)?;
	}

	l == r
});

/// **4.3.10** `&`  
pub const AND: Function = function!("&", env, |lhs, rhs| {
	let l = lhs.run(env)?;

	if l.to_boolean()? {
		rhs.run(env)?
	} else {
		l
	}
});

/// **4.3.11** `|`  
pub const OR: Function = function!("|", env, |lhs, rhs| {
	let l = lhs.run(env)?;

	if l.to_boolean()? {
		l
	} else {
		rhs.run(env)?
	}
});

/// **4.3.12** `;`  
pub const THEN: Function = function!(";", env, |lhs, rhs| {
	lhs.run(env)?;
	rhs.run(env)?
});

fn assign<'e>(variable: &Value<'e>, value: Value<'e>, env: &mut Environment<'e>) -> Result<()> {
	match variable {
		Value::Variable(var) => {
			var.assign(value);
		}

		#[cfg(feature = "assign-to-prompt")]
		Value::Ast(ast) if ast.function().name == PROMPT.name => env.add_to_prompt(value.to_text()?),

		#[cfg(all(feature = "assign-to-system", feature = "system-function"))]
		Value::Ast(ast) if ast.function().name == SYSTEM.name => env.add_to_system(value.to_text()?),

		#[cfg(not(any(feature = "assign-to-strings", feature = "assign-to-lists")))]
		other => return Err(Error::TypeError(other.typename(), "=")),

		other => match other.run(env)? {
			#[cfg(feature = "assign-to-lists")]
			Value::List(_list) => todo!(),

			#[cfg(feature = "assign-to-strings")]
			Value::Text(name) => {
				env.lookup(&name)?.assign(value);
			}

			other => return Err(Error::TypeError(other.typename(), "=")),
		},
	}

	let _ = env;

	Ok(())
}

/// **4.3.13** `=`  
pub const ASSIGN: Function = function!("=", env, |var, value| {
	let ret = value.run(env)?;
	assign(var, ret.clone(), env)?;
	ret
});

/// **4.3.14** `WHILE`  
pub const WHILE: Function = function!("WHILE", env, |cond, body| {
	while cond.run(env)?.to_boolean()? {
		body.run(env)?;
	}

	Value::Null
});

/// **4.4.1** `IF`  
pub const IF: Function = function!("IF", env, |cond, iftrue, iffalse| {
	if cond.run(env)?.to_boolean()? {
		iftrue.run(env)?
	} else {
		iffalse.run(env)?
	}
});

fn fix_len(container: &Value<'_>, mut start: Integer) -> Result<usize> {
	if start.is_negative() && cfg!(feature = "negative-indexing") {
		let len = match container {
			Value::Text(text) => text.len(),
			Value::List(list) => list.len(),
			other => return Err(Error::TypeError(other.typename(), "get/set")),
		};

		start = start.add(len.try_into()?)?;
	}

	start.try_conv::<usize>().or(Err(Error::DomainError("negative start position")))
}

/// **4.4.2** `GET`  
pub const GET: Function = function!("GET", env, |string, start, length| {
	let source = string.run(env)?;
	let start = fix_len(&source, start.run(env)?.to_integer()?)?;
	let length = length
		.run(env)?
		.to_integer()?
		.try_conv::<usize>()
		.or(Err(Error::DomainError("negative length")))?;

	match source {
		Value::List(list) => list
			.get(start..start + length)
			.ok_or_else(|| Error::IndexOutOfBounds { len: list.len(), index: start + length })?
			.conv::<Value>(),
		Value::Text(text) => text
			.get(start..start + length)
			.ok_or_else(|| Error::IndexOutOfBounds { len: text.len(), index: start + length })?
			.to_owned()
			.into(),
		other => return Err(Error::TypeError(other.typename(), "GET")),
	}
});

/// **4.5.1** `SET`  
pub const SET: Function = function!("SET", env, |string, start, length, replacement| {
	let source = string.run(env)?;
	let start = fix_len(&source, start.run(env)?.to_integer()?)?;
	let length = length
		.run(env)?
		.to_integer()?
		.try_conv::<usize>()
		.or(Err(Error::DomainError("negative length")))?;
	let replacement_source = replacement.run(env)?;

	match source {
		Value::List(list) => {
			// FIXME: cons?

			let replacement = replacement_source.to_list()?;
			let mut ret = Vec::new();
			ret.extend(list.iter().take(start).cloned());
			ret.extend(replacement.iter().cloned());
			ret.extend(list.iter().skip((start) + length).cloned());

			List::try_from(ret)?.conv::<Value>()
		}
		Value::Text(text) => {
			let replacement = replacement_source.to_text()?;

			// lol, todo, optimize me
			let mut builder = Text::builder();
			builder.push(text.get(..start).unwrap());
			builder.push(&replacement);
			builder.push(text.get(start + length..).unwrap());
			builder.finish().into()
		}
		other => return Err(Error::TypeError(other.typename(), "SET")),
	}
});

/// **6.1** `VALUE`
#[cfg(feature = "value-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "value-function")))]
pub const VALUE: Function = function!("VALUE", env, |arg| {
	let name = arg.run(env)?.to_text()?;
	env.lookup(&name)?
});

#[cfg(feature = "handle-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "handle-function")))]
pub const HANDLE: Function = function!("HANDLE", env, |block, iferr| {
	const ERR_VAR_NAME: &crate::TextSlice = unsafe { crate::TextSlice::new_unchecked("_") };

	match block.run(env) {
		Ok(value) => value,
		Err(err) => {
			// This is fallible, as the error string might have had something bad.
			let errmsg = err.to_string().try_conv::<Text>()?;

			// Assign it to the error variable
			env.lookup(ERR_VAR_NAME).unwrap().assign(errmsg.into());

			// Finally, execute the RHS.
			iferr.run(env)?
		}
	}
});

#[cfg(feature = "yeet-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "yeet-function")))]
pub const YEET: Function = function!("YEET", env, |errmsg| {
	return Err(Error::Custom(errmsg.run(env)?.to_text()?.to_string().into()));
	#[allow(unreachable_code)]
	Value::Null
});

/// **6.3** `USE`
#[cfg(feature = "use-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "use-function")))]
pub const USE: Function = function!("USE", env, |arg| {
	let filename = arg.run(env)?.to_text()?;
	let contents = env.read_file(&filename)?;

	env.play(&contents)?
});

/// **4.2.2** `EVAL`
#[cfg(feature = "eval-function")]
pub const EVAL: Function = function!("EVAL", env, |val| {
	let code = val.run(env)?.to_text()?;
	env.play(&code)?
});

#[cfg(feature = "system-function")]
/// **4.2.5** `` ` ``
pub const SYSTEM: Function = function!("$", env, |cmd, stdin| {
	let command = cmd.run(env)?.to_text()?;
	let stdin = match stdin.run(env)? {
		Value::Text(text) => Some(text),
		Value::Null => None,
		other => return Err(Error::TypeError(other.typename(), "$")),
	};

	env.run_command(&command, stdin.as_deref())?
});

/// **Compiler extension**: SRAND
#[cfg(feature = "xsrand-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "xsrand-function")))]
pub const XSRAND: Function = function!("XSRAND", env, |arg| {
	let seed = arg.run(env)?.to_integer()?;
	env.srand(seed);
	Value::default()
});

/// **Compiler extension**: REV
#[cfg(feature = "xreverse-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "xreverse-function")))]
pub const XREVERSE: Function = function!("XREVERSE", env, |arg| {
	let seed = arg.run(env)?.to_integer()?;
	env.srand(seed);
	Value::default()
});

#[cfg(feature = "xrange-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "xrange-function")))]
pub const XRANGE: Function = function!("XRANGE", env, |start, stop| {
	match start.run(env)? {
		Value::Integer(start) => {
			let stop = stop.run(env)?.to_integer()?;

			match start <= stop {
				true => (i64::from(start)..i64::from(stop))
					.map(|x| Value::from(Integer::try_from(x).unwrap()))
					.collect::<Vec<Value<'_>>>()
					.try_conv::<List<'_>>()
					.expect("todo: out of bounds error")
					.conv::<Value>(),

				#[cfg(feature = "negative-ranges")]
				false => (stop..start).map(Value::from).rev().collect::<List>().into(),

				#[cfg(not(feature = "negative-ranges"))]
				false => return Err(Error::DomainError("start is greater than stop")),
			}
		}

		Value::Text(_text) => {
			// let start = text.get(0).a;
			todo!()
		}

		other => return Err(Error::TypeError(other.typename(), "XRANGE")),
	}
});

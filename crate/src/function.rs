use crate::{Environment, Error, Integer, List, Result, SharedText, Value};
use std::fmt::{self, Debug, Formatter};
use std::io::{BufRead, Write};
use tap::prelude::*;

/// A function in knight indicates
#[derive(Clone, Copy)]
pub struct Function {
	/// The code associated with this function
	pub func: fn(&[Value], &mut Environment) -> Result<Value>,

	/// The long-hand name of this function.
	///
	/// For extension functions that start with `X`, this should also start with it.
	pub name: &'static str,

	/// The arity of the function.
	pub arity: usize,
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

pub const fn fetch(name: char) -> Option<&'static Function> {
	const PROMPT_NAME: char = PROMPT.name.as_bytes()[0] as char;
	const RANDOM_NAME: char = RANDOM.name.as_bytes()[0] as char;
	const BLOCK_NAME: char = BLOCK.name.as_bytes()[0] as char;
	const CALL_NAME: char = CALL.name.as_bytes()[0] as char;
	const SYSTEM_NAME: char = SYSTEM.name.as_bytes()[0] as char;
	const QUIT_NAME: char = QUIT.name.as_bytes()[0] as char;
	const NOT_NAME: char = NOT.name.as_bytes()[0] as char;
	const LENGTH_NAME: char = LENGTH.name.as_bytes()[0] as char;
	const DUMP_NAME: char = DUMP.name.as_bytes()[0] as char;
	const OUTPUT_NAME: char = OUTPUT.name.as_bytes()[0] as char;
	const ASCII_NAME: char = ASCII.name.as_bytes()[0] as char;
	const NEG_NAME: char = NEG.name.as_bytes()[0] as char;
	const BOX_NAME: char = BOX.name.as_bytes()[0] as char;
	const ADD_NAME: char = ADD.name.as_bytes()[0] as char;
	const SUBTRACT_NAME: char = SUBTRACT.name.as_bytes()[0] as char;
	const MULTIPLY_NAME: char = MULTIPLY.name.as_bytes()[0] as char;
	const DIVIDE_NAME: char = DIVIDE.name.as_bytes()[0] as char;
	const MODULO_NAME: char = MODULO.name.as_bytes()[0] as char;
	const POWER_NAME: char = POWER.name.as_bytes()[0] as char;
	const EQUALS_NAME: char = EQUALS.name.as_bytes()[0] as char;
	const LESS_THAN_NAME: char = LESS_THAN.name.as_bytes()[0] as char;
	const GREATER_THAN_NAME: char = GREATER_THAN.name.as_bytes()[0] as char;
	const AND_NAME: char = AND.name.as_bytes()[0] as char;
	const OR_NAME: char = OR.name.as_bytes()[0] as char;
	const THEN_NAME: char = THEN.name.as_bytes()[0] as char;
	const ASSIGN_NAME: char = ASSIGN.name.as_bytes()[0] as char;
	const WHILE_NAME: char = WHILE.name.as_bytes()[0] as char;
	const RANGE_NAME: char = RANGE.name.as_bytes()[0] as char;
	const IF_NAME: char = IF.name.as_bytes()[0] as char;
	const GET_NAME: char = GET.name.as_bytes()[0] as char;
	const SET_NAME: char = SET.name.as_bytes()[0] as char;

	#[cfg(feature = "value-function")]
	const VALUE_NAME: char = VALUE.name.as_bytes()[0] as char;

	#[cfg(feature = "eval-function")]
	const EVAL_NAME: char = EVAL.name.as_bytes()[0] as char;

	#[cfg(feature = "handle-function")]
	const HANDLE_NAME: char = HANDLE.name.as_bytes()[0] as char;

	#[cfg(feature = "use-function")]
	const USE_NAME: char = USE.name.as_bytes()[0] as char;

	match name {
		PROMPT_NAME => Some(&PROMPT),
		RANDOM_NAME => Some(&RANDOM),

		BLOCK_NAME => Some(&BLOCK),
		CALL_NAME => Some(&CALL),
		SYSTEM_NAME => Some(&SYSTEM),
		QUIT_NAME => Some(&QUIT),
		NOT_NAME => Some(&NOT),
		LENGTH_NAME => Some(&LENGTH),
		DUMP_NAME => Some(&DUMP),
		OUTPUT_NAME => Some(&OUTPUT),
		ASCII_NAME => Some(&ASCII),
		NEG_NAME => Some(&NEG),
		BOX_NAME => Some(&BOX),
		'[' => Some(&UNBOX),
		']' => Some(&TAIL),

		ADD_NAME => Some(&ADD),
		SUBTRACT_NAME => Some(&SUBTRACT),
		MULTIPLY_NAME => Some(&MULTIPLY),
		DIVIDE_NAME => Some(&DIVIDE),
		MODULO_NAME => Some(&MODULO),
		POWER_NAME => Some(&POWER),
		EQUALS_NAME => Some(&EQUALS),
		LESS_THAN_NAME => Some(&LESS_THAN),
		GREATER_THAN_NAME => Some(&GREATER_THAN),
		AND_NAME => Some(&AND),
		OR_NAME => Some(&OR),
		THEN_NAME => Some(&THEN),
		ASSIGN_NAME => Some(&ASSIGN),
		WHILE_NAME => Some(&WHILE),
		RANGE_NAME => Some(&RANGE),

		IF_NAME => Some(&IF),
		GET_NAME => Some(&GET),
		SET_NAME => Some(&SET),

		#[cfg(feature = "value-function")]
		VALUE_NAME => Some(&VALUE),

		#[cfg(feature = "eval-function")]
		EVAL_NAME => Some(&EVAL),

		#[cfg(feature = "handle-function")]
		_ if name == HANDLE_NAME => Some(&HANDLE),

		#[cfg(feature = "use-function")]
		_ if name == USE_NAME => Some(&USE),

		_ => None,
	}
}

macro_rules! arity {
	() => (0);
	($_pat:ident $($rest:ident)*) => (1+arity!($($rest)*))
}
macro_rules! function {
	($name:literal, $env:pat, |$($args:ident),*| $body:expr) => {
		Function {
			name: $name,
			arity: arity!($($args)*),
			func: |args, $env| {
				let [$($args,)*]: &[Value; arity!($($args)*)] = args.try_into().unwrap();
				Ok($body.into())
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
	env.read_line(&mut buf)?;

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

	buf.try_conv::<SharedText>()?
});

/// **4.1.5**: `RANDOM`
pub const RANDOM: Function = function!("RANDOM", env, |/* comment for rustfmt */| env.random());

/// **4.2.2** `BOX`
pub const BOX: Function = function!(",", env, |val| {
	let value = val.run(env)?;

	List::from(vec![value])
});

pub const UNBOX: Function = function!("]", env, |val| {
	let value = val.run(env)?.to_list()?;

	value.get(0).unwrap()
});

pub const TAIL: Function = function!("]", env, |val| {
	let value = val.run(env)?.to_list()?;

	value.get(1..value.len()).unwrap()
});

/// **4.2.3** `BLOCK`  
pub const BLOCK: Function = function!("BLOCK", env, |arg| {
	// Technically, according to the spec, only the return value from `BLOCK` can be used in `CALL`.
	// Since this function normally just returns whatever it's argument is, it's impossible to
	// distinguish an `Integer` returned from `BLOCK` and one simply given to `CALL`. As such, when
	// the `strict-block-return-value` feature is enabled. We ensure that we _only_ return `Ast`s
	// from `BLOCK`, so `CALL` can verify them.
	#[cfg(feature = "strict-block-return-value")]
	if !matches!(arg, Value::Ast(_)) {
		// The NOOP function literally just runs its argument.
		const NOOP: Function = function!(":", env, |arg| {
			debug_assert!(!matches!(arg, Value::Ast(_)));

			arg.run(env) // We can't `.clone()` in case we're given a variable name.
		});

		return Ok(crate::Ast::new(&NOOP, vec![arg.clone()]).into());
	}

	let _ = env;

	arg.clone()
});

/// **4.2.4** `CALL`  
pub const CALL: Function = function!("CALL", env, |arg| {
	let block = arg.run(env)?;

	// When ensuring that `CALL` is only given values returned from `BLOCK`, we must ensure that all
	// arguments are `Value::Ast`s.
	#[cfg(feature = "strict-block-return-value")]
	if !matches!(block, Value::Ast(_)) {
		return Err(Error::TypeError(block.typename()));
	}

	block.run(env)?
});

/// **4.2.5** `` ` ``
pub const SYSTEM: Function = function!("`", env, |arg| {
	let command = arg.run(env)?.to_text()?;

	env.run_command(&command)?
});

/// **4.2.6** `QUIT`  
pub const QUIT: Function = function!("QUIT", env, |arg| {
	let status = arg
		.run(env)?
		.to_integer()?
		.try_conv::<i32>()
		.or(Err(Error::DomainError("exit code out of bounds")))?;

	// Technically, only values in the range `[0, 127]` are supported by the knight impl. However,
	// this compliance feature isn't really important enough to warrant its own config feature.
	#[cfg(feature = "strict-compliance")]
	if !(0..=127).contains(&status) {
		return Err(Error::DomainError("exit code out of bounds"));
	}

	return Err(Error::Quit(status));

	// The `function!` macro calls `.into()` on the return value of this block,
	// so we need _something_ here so it can typecheck correctly.
	#[allow(unreachable_code)]
	Value::Null
});

/// **4.2.7** `!`  
pub const NOT: Function = function!("!", env, |arg| !arg.run(env)?.to_bool()?);

/// **4.2.8** `LENGTH`  
pub const LENGTH: Function = function!("LENGTH", env, |arg| {
	match arg.run(env)? {
		Value::SharedText(text) => {
			debug_assert_eq!(text.len(), Value::SharedText(text.clone()).to_list().unwrap().len());
			text.len() as Integer
		}
		Value::List(list) => list.len() as Integer,
		// TODO: integer base10 when that comes out.
		other => other.to_list()?.len() as Integer,
	}
});

/// **4.2.9** `DUMP`  
pub const DUMP: Function = function!("DUMP", env, |arg| {
	let value = arg.run(env)?;
	writeln!(env, "{value:?}")?;
	value
});

/// **4.2.10** `OUTPUT`  
pub const OUTPUT: Function = function!("OUTPUT", env, |arg| {
	let text = arg.run(env)?.to_text()?;

	if let Some(stripped) = text.strip_suffix('\\') {
		write!(env, "{stripped}")?
	} else {
		writeln!(env, "{text}")?;
	}

	env.flush()?;

	Value::Null
});

/// **4.2.11** `ASCII`  
pub const ASCII: Function = function!("ASCII", env, |arg| {
	match arg.run(env)? {
		Value::Integer(num) => u32::try_from(num)
			.ok()
			.and_then(char::from_u32)
			.and_then(|chr| SharedText::new(chr).ok())
			.ok_or(Error::DomainError("number isn't a valid char"))?
			.conv::<Value>(),

		Value::SharedText(text) => text
			.chars()
			.next()
			.ok_or(Error::DomainError("empty string"))?
			.pipe(|x| x as Integer)
			.conv::<Value>(),

		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.2.12** `~`  
pub const NEG: Function = function!("~", env, |arg| {
	let num = arg.run(env)?.to_integer()?;

	cfg_if! {
		if #[cfg(feature = "checked-overflow")] {
			num.checked_neg().ok_or(Error::IntegerOverflow)?
		} else {
			num.wrapping_neg()
		}
	}
});

/// **4.3.1** `+`  
pub const ADD: Function = function!("+", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					lnum.checked_add(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
				} else {
					lnum.wrapping_add(rnum).conv::<Value>()
				}
			}
		}

		Value::SharedText(string) => string.concat(&rhs.run(env)?.to_text()?).into(),
		Value::List(list) => list.concat(&rhs.run(env)?.to_list()?).into(),

		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.2** `-`  
pub const SUBTRACT: Function = function!("-", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					lnum.checked_sub(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
				} else {
					lnum.wrapping_sub(rnum).conv::<Value>()
				}
			}
		}

		#[cfg(feature = "list-extensions")]
		Value::List(list) => list.difference(&rhs.run(env)?.to_list()?).into(),

		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.3** `*`  
pub const MULTIPLY: Function = function!("*", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					lnum.checked_mul(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
				} else {
					lnum.wrapping_mul(rnum).conv::<Value>()
				}
			}
		}

		Value::SharedText(lstr) => {
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
			list.repeat(amount).conv::<Value>()
		}

		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.4** `/`  
pub const DIVIDE: Function = function!("/", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			if rnum == 0 {
				return Err(Error::DivisionByZero);
			}

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					lnum.checked_div(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
				} else {
					lnum.wrapping_div(rnum).conv::<Value>()
				}
			}
		}

		#[cfg(feature = "string-extensions")]
		Value::SharedText(text) => text.split(&rhs.run(env)?.to_text()?).into(),

		#[cfg(feature = "list-extensions")]
		Value::List(list) => list.reduce(&rhs.run(env)?, env)?.unwrap_or_default(),

		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.5** `%`  
pub const MODULO: Function = function!("%", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			if rnum == 0 {
				return Err(Error::DivisionByZero);
			}

			if cfg!(feature = "strict-compliance") && rnum < 0 {
				return Err(Error::DomainError("modulo by a negative base"));
			}

			// TODO: check if `rem` actually follows the specs.
			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					lnum.checked_rem(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
				} else {
					lnum.wrapping_rem(rnum).conv::<Value>()
				}
			}
		}

		// #[cfg(feature = "string-extensions")]
		// Value::SharedText(lstr) => {
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

		// 	SharedText::new(formatted).unwrap().into()
		// }
		#[cfg(feature = "list-extensions")]
		Value::List(list) => list.filter(&rhs.run(env)?, env)?.into(),

		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.6** `^`  
pub const POWER: Function = function!("^", env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(base) => match (base, rhs.run(env)?.to_integer()?) {
			(_, Integer::MIN..=-1) => return Err(Error::DomainError("negative exponent")),
			(_, 0) => 1.into(),
			(0 | 1, _) => base.into(),

			#[cfg(feature = "checked-overflow")]
			(_, exponent) => {
				let exp =
					exponent.try_conv::<u32>().or(Err(Error::DomainError("negative exponent")))?;
				base.checked_pow(exp).ok_or(Error::IntegerOverflow)?.conv::<Value>()
			}
			#[cfg(not(feature = "checked-overflow"))]
			(_, exponent) => base.wrapping_pow(exponent as u32).conv::<Value>(),
		},

		Value::List(list) => list.join(&rhs.run(env)?.to_text()?)?.into(),

		other => return Err(Error::TypeError(other.typename())),
	}
});

fn compare(lhs: &Value, rhs: &Value) -> Result<std::cmp::Ordering> {
	match lhs {
		Value::Integer(lnum) => Ok(lnum.cmp(&rhs.to_integer()?)),
		Value::Boolean(lbool) => Ok(lbool.cmp(&rhs.to_bool()?)),
		Value::SharedText(ltext) => Ok(ltext.cmp(&rhs.to_text()?)),
		Value::List(list) => {
			let rhs = rhs.to_list()?;

			// feels bad to be iterating over by-values.
			for (left, right) in list.iter().zip(&rhs) {
				match compare(&left, &right)? {
					std::cmp::Ordering::Equal => {}
					other => return Ok(other),
				}
			}

			Ok(list.len().cmp(&rhs.len()))
		}
		other => Err(Error::TypeError(other.typename())),
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
	let l = lhs.run(env)?;
	let r = rhs.run(env)?;

	if cfg!(feature = "strict-compliance") {
		if matches!(l, Value::Ast(_) | Value::Variable(_)) {
			return Err(Error::TypeError(l.typename()));
		}

		if matches!(r, Value::Ast(_) | Value::Variable(_)) {
			return Err(Error::TypeError(r.typename()));
		}
	}

	l == r
});

/// **4.3.10** `&`  
pub const AND: Function = function!("&", env, |lhs, rhs| {
	let l = lhs.run(env)?;

	if l.to_bool()? {
		rhs.run(env)?
	} else {
		l
	}
});

/// **4.3.11** `|`  
pub const OR: Function = function!("|", env, |lhs, rhs| {
	let l = lhs.run(env)?;

	if l.to_bool()? {
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

fn assign(variable: &Value, value: Value, env: &mut Environment) -> Result<()> {
	match variable {
		Value::Variable(var) => {
			var.assign(value);
		}

		#[cfg(feature = "assign-to-prompt")]
		Value::Ast(ast) if ast.function().name == PROMPT.name => env.add_to_prompt(value.to_text()?),

		#[cfg(feature = "assign-to-system")]
		Value::Ast(ast) if ast.function().name == SYSTEM.name => env.add_to_system(value.to_text()?),

		#[cfg(feature = "list-extensions")]
		Value::Ast(_ast) => return assign(&variable.run(env)?, value, env),

		// #[cfg(feature = "assign-to-lists")]
		// Value::List(list) => {
		// 	if list.is_empty() {
		// 		panic!("todo: error for this case");
		// 	}
		// 	let rhs = value.run(env)?.to_list()?;

		// 	for (name, val) in list.iter().zip(&rhs) {
		// 		assign(&name, val, env)?;
		// 	}

		// 	match list.len().cmp(&rhs.len()) {
		// 		std::cmp::Ordering::Equal => {}
		// 		std::cmp::Ordering::Less => assign(
		// 			list.as_slice().iter().last().unwrap(),
		// 			rhs.as_slice()[list.len() - 1..].iter().cloned().collect::<List>().into(),
		// 			env,
		// 		)?,
		// 		std::cmp::Ordering::Greater => {
		// 			for extra in &list.as_slice()[rhs.len()..] {
		// 				assign(extra, Value::default(), env)?;
		// 			}
		// 		}
		// 	}
		// }
		#[cfg(feature = "assign-to-anything")]
		_ => {
			let name = variable.run(env)?.to_text()?;
			env.lookup(&name)?.assign(value);
		}

		#[cfg(not(feature = "assign-to-anything"))]
		other => return Err(Error::TypeError(other.name())),
	}

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
	while cond.run(env)?.to_bool()? {
		body.run(env)?;
	}

	Value::Null
});

/// **4.3.15** `RANGE`  
pub const RANGE: Function = function!(".", env, |start, stop| {
	match start.run(env)? {
		Value::Integer(start) => {
			let stop = stop.run(env)?.to_integer()?;

			match start <= stop {
				true => (start..stop).map(Value::from).collect::<List>().conv::<Value>(),

				#[cfg(feature = "negative-ranges")]
				false => (stop..start).map(Value::from).rev().collect::<List>().into(),

				#[cfg(not(feature = "negative-ranges"))]
				false => return Err(Error::DomainError("start is greater than stop")),
			}
		}

		Value::SharedText(_text) => {
			// let start = text.get(0).a;
			todo!()
		}

		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.4.1** `IF`  
pub const IF: Function = function!("IF", env, |cond, iftrue, iffalse| {
	if cond.run(env)?.to_bool()? {
		iftrue.run(env)?
	} else {
		iffalse.run(env)?
	}
});

/// **4.4.2** `GET`  
pub const GET: Function = function!("GET", env, |string, start, length| {
	let source = string.run(env)?;
	let mut start = start.run(env)?.to_integer()?;
	let length = length
		.run(env)?
		.to_integer()?
		.try_conv::<usize>()
		.or(Err(Error::DomainError("negative length")))?;

	match source {
		Value::List(list) => {
			if start < 0 && cfg!(feature = "negative-indexing") {
				start += list.len() as Integer;
			}
			let start =
				start.try_conv::<usize>().or(Err(Error::DomainError("negative start position")))?;

			// special case for `GET` with a length of zero returns just that element
			let fetched = /*if length == 0 {
				list.get(start)
			} else {*/
				list.get(start..start + length).map(Value::from)
			/*}*/;

			match fetched {
				Some(fetched) => fetched,

				#[cfg(feature = "no-oob-errors")]
				None => return Err(Error::IndexOutOfBounds { len: list.len(), index: start + length }),

				#[cfg(not(feature = "no-oob-errors"))]
				None => List::default().into(),
			}
		}
		Value::SharedText(text) => {
			if start < 0 && cfg!(feature = "negative-indexing") {
				start += text.len() as Integer;
			}

			let start =
				start.try_conv::<usize>().or(Err(Error::DomainError("negative start position")))?;

			match text.get(start..start + length) {
				Some(substring) => substring.to_owned().into(),

				#[cfg(feature = "no-oob-errors")]
				None => return Err(Error::IndexOutOfBounds { len: text.len(), index: start + length }),

				#[cfg(not(feature = "no-oob-errors"))]
				None => SharedText::default().into(),
			}
		}
		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.5.1** `SET`  
pub const SET: Function = function!("SET", env, |string, start, length, replacement| {
	let source = string.run(env)?;
	let mut start = start.run(env)?.to_integer()?;
	let length = length
		.run(env)?
		.to_integer()?
		.try_conv::<usize>()
		.or(Err(Error::DomainError("negative length")))?;
	let replacement_source = replacement.run(env)?;

	match source {
		Value::List(list) => {
			if start < 0 && cfg!(feature = "negative-indexing") {
				start += list.len() as Integer;
			}

			let start =
				start.try_conv::<usize>().or(Err(Error::DomainError("negative start position")))?;

			// FIXME: cons?

			let replacement = replacement_source.to_list()?;
			let mut ret = Vec::new();
			ret.extend(list.iter().take(start));
			ret.extend(replacement.iter());
			ret.extend(list.iter().skip((start) + length));

			List::from(ret).conv::<Value>()
		}
		Value::SharedText(text) => {
			let replacement = replacement_source.to_text()?;

			if start < 0 && cfg!(feature = "negative-indexing") {
				start += text.len() as Integer;
			}

			let start =
				start.try_conv::<usize>().or(Err(Error::DomainError("negative start position")))?;

			// TODO: `no-oob-errors` here
			// lol, todo, optimize me
			let mut builder = SharedText::builder();
			builder.push(&text.get(..start).unwrap());
			builder.push(&replacement);
			builder.push(&text.get(start + length..).unwrap());
			builder.finish().into()
		}
		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **6.1** `VALUE`
#[cfg(feature = "value-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "value-function")))]
pub const VALUE: Function = function!("V", env, |arg| {
	let name = arg.run(env)?.to_text()?;
	env.lookup(&name)?
});

/// **6.4** `HANDLE`
#[cfg(feature = "handle-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "handle-function")))]
pub const HANDLE: Function = function!("H", env, |block, iferr| {
	const ERR_VAR_NAME: &'static crate::Text = unsafe { crate::Text::new_unchecked("_") };

	match block.run(env) {
		Ok(value) => value,
		Err(err) => {
			// This is fallible, as the error string might have had something bad.
			let errmsg = err.to_string().try_conv::<SharedText>()?;

			// Assign it to the error variable
			env.lookup(ERR_VAR_NAME).unwrap().assign(errmsg.into());

			// Finally, execute the RHS.
			iferr.run(env)?
		}
	}
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
pub const EVAL: Function = function!("E", env, |val| {
	let code = val.run(env)?.to_text()?;
	env.play(&code)?
});

/// **Compiler extension**: SRAND
#[cfg(feature = "srand-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "srand-function")))]
pub const SRAND: Function = function!("XSRAND", env, |arg| {
	let seed = arg.run(env)?.to_integer()?;
	env.srand(seed);
	Value::default()
});

/// **Compiler extension**: REV
#[cfg(feature = "reverse-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "reverse-function")))]
pub const REVERSE: Function = function!("XREV", env, |arg| {
	let seed = arg.run(env)?.to_integer()?;
	env.srand(seed);
	Value::default()
});

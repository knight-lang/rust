use crate::{value::Number, Environment, Error, Result, SharedStr, Value};
use std::fmt::{self, Debug, Formatter};
use std::io::{BufRead, Write};
use tap::prelude::*;

#[derive(Clone, Copy)]
pub struct Function {
	/// The code associated with this function
	pub func: fn(&[Value], &mut Environment) -> Result<Value>,

	/// The single character name of this function
	pub name: char,

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

macro_rules! arity {
	() => (0);
	($_pat:ident $($rest:ident)*) => (1+arity!($($rest)*))
}

macro_rules! functions {
	(
		$env:ident;
		$($(#[$meta:meta])* fn $name:ident ($chr:literal $(,$args:ident)*) $body:block)*
	) => {
		pub fn fetch(name: char) -> Option<&'static Function> {
			match name {
				$($chr => Some(&$name),)*
				_ => None
			}
		}
		$(
			$(#[$meta])*
			pub const $name: Function = Function {
				name: $chr,
				arity: arity!($($args)*),
				func: |args, $env| {
					let [$($args,)*]: &[Value; arity!($($args)*)] = args.try_into().unwrap();
					Ok($body)
				}
			};
		)*
	};
}

functions! { env;
	fn PROMPT ('P') {
		let mut buf = String::new();

		env.read_line(&mut buf)?;

		// remove trailing newlines
		match buf.pop() {
			Some('\n') => match buf.pop() {
				Some('\r') => {},
				Some(other) => buf.push(other), // ie `<anything>\n`
				None => {}
			},
			Some(other) => buf.push(other),
			None => {}
		}

		SharedStr::try_from(buf)?.into()
	}

	fn RANDOM ('R') {
		env.random().into()
	}

	fn EVAL ('E', arg) {
		let input = arg.run(env)?.to_knstr()?;
		env.play(&input)?
	}

	fn BLOCK ('B', arg) {
		arg.clone()
	}

	fn CALL ('C', arg) {
		arg.run(env)?.run(env)?
	}

	fn SYSTEM ('`', arg) {
		let command = arg.run(env)?.to_knstr()?;
		env.run_command(&command)?.into()
	}

	fn QUIT ('Q', arg) {
		let status = arg
			.run(env)?
			.to_number()?
			.try_conv::<i32>()
			.or(Err(Error::DomainError("exit code out of bounds")))?;

		return Err(Error::Quit(status));
	}

	fn NOT ('!', arg) {
		(!arg.run(env)?.to_bool()?).into()
	}

	fn LENGTH ('L', arg) {
		(arg.run(env)?.to_knstr()?.len() as Number).into()
	}

	fn DUMP ('D', arg) {
		let value = arg.run(env)?;
		writeln!(env, "{value:?}")?;
		value
	}

	fn OUTPUT ('O', arg) {
		let text = arg.run(env)?.to_knstr()?;

		if text.chars().last() == Some('\\') {
			write!(env, "{}", &text[..text.len() - 2])?
		} else {
			writeln!(env, "{text}")?;
		}

		Value::Null
	}


	fn ASCII ('A', arg) {
		match arg.run(env)? {
			Value::Number(num) =>
				u32::try_from(num)
					.ok()
					.and_then(char::from_u32)
					.and_then(|chr| SharedStr::new(chr).ok())
					.ok_or(Error::DomainError("number isn't a valid char"))?
					.into(),

			Value::SharedStr(text) =>
				text.chars()
					.next()
					.ok_or(Error::DomainError("empty string"))?
					.pipe(|x| x as Number)
					.into(),

			other => return Err(Error::TypeError(other.typename()))
		}
	}

	fn NEG ('~', arg) {
		let num = arg.run(env)?.to_number()?;

		#[cfg(feature = "checked-overflow")]
		{ num.checked_neg().ok_or(Error::IntegerOverflow)?.into() }

		#[cfg(not(feature = "checked-overflow"))]
		{ num.wrapping_neg().into() }
	}

	fn ADD('+', lhs, rhs) {
		match lhs.run(env)? {
			Value::Number(lnum) => {
				let rnum = rhs.run(env)?.to_number()?;

				#[cfg(feature = "checked-overflow")]
				{ lnum.checked_add(rnum).ok_or(Error::IntegerOverflow)?.into() }

				#[cfg(not(feature = "checked-overflow"))]
				{ lnum.wrapping_add(rnum).into() }
			},
			Value::SharedStr(lstr) => {
				let rstr = rhs.run(env)?.to_knstr()?;
				let mut cat = String::with_capacity(lstr.len() + rstr.len());
				cat.push_str(&lstr);
				cat.push_str(&rstr);

				SharedStr::try_from(cat).unwrap().into()
			},

			other => return Err(Error::TypeError(other.typename()))
		}
	}

	fn SUBTRACT('-', lhs, rhs) {
		match lhs.run(env)? {
			Value::Number(lnum) => {
				let rnum = rhs.run(env)?.to_number()?;

				#[cfg(feature = "checked-overflow")]
				{ lnum.checked_sub(rnum).ok_or(Error::IntegerOverflow)?.into() }

				#[cfg(not(feature = "checked-overflow"))]
				{ lnum.wrapping_sub(rnum).into() }
			},
			other => return Err(Error::TypeError(other.typename()))
		}
	}

	fn MULTIPLY('*', lhs, rhs) {
		match lhs.run(env)? {
			Value::Number(lnum) => {
				let rnum = rhs.run(env)?.to_number()?;

				#[cfg(feature = "checked-overflow")]
				{ lnum.checked_mul(rnum).ok_or(Error::IntegerOverflow)?.into() }

				#[cfg(not(feature = "checked-overflow"))]
				{ lnum.wrapping_mul(rnum).into() }
			},
			Value::SharedStr(lstr) => {
				// clean me up
				let amount = rhs
					.run(env)?
					.to_number()?
					.try_conv::<usize>()
					.map_err(|_| Error::DomainError("repetition length not within bounds"))?;

				if amount * lstr.len() >= (isize::MAX as usize) {
					return Err(Error::DomainError("repetition length not within bounds"));
				}

				lstr.repeat(amount).try_conv::<SharedStr>().unwrap().into()
			},
			other => return Err(Error::TypeError(other.typename()))
		}
	}

	fn DIVIDE('/', lhs, rhs) {
		match lhs.run(env)? {
			Value::Number(lnum) => {
				let rnum = rhs.run(env)?.to_number()?;

				if rnum == 0 {
					return Err(Error::DivisionByZero);
				}

				#[cfg(feature = "checked-overflow")]
				{ lnum.checked_div(rnum).ok_or(Error::IntegerOverflow)?.into() }

				#[cfg(not(feature = "checked-overflow"))]
				{ lnum.wrapping_div(rnum).into() }
			},
			other => return Err(Error::TypeError(other.typename()))
		}
	}

	fn MODULO('%', lhs, rhs) {
		match lhs.run(env)? {
			Value::Number(lnum) => {
				let rnum = rhs.run(env)?.to_number()?;

				if rnum == 0 {
					return Err(Error::DivisionByZero);
				}

				#[cfg(feature = "checked-overflow")]
				{ lnum.checked_rem(rnum).ok_or(Error::IntegerOverflow)?.into() }

				#[cfg(not(feature = "checked-overflow"))]
				{ lnum.wrapping_rem(rnum).into() }
			},
			other => return Err(Error::TypeError(other.typename()))
		}
	}

	fn POWER('^', lhs, rhs) {
		// TODO: verify this is actually power work
		match lhs.run(env)? {
			Value::Number(lnum) => {
				let rnum = rhs.run(env)?
					.to_number()?
					.try_conv::<u32>()
					.or(Err(Error::DomainError("invalid exponent")))?;

				#[cfg(feature = "checked-overflow")]
				{ lnum.checked_pow(rnum).ok_or(Error::IntegerOverflow)?.into() }

				#[cfg(not(feature = "checked-overflow"))]
				{ lnum.wrapping_pow(rnum).into() }
			}
			other => return Err(Error::TypeError(other.typename()))
		}
	}

	fn EQUALS('?', lhs, rhs) {
		let l = lhs.run(env)?;
		let r = rhs.run(env)?;

		if cfg!(feature = "strict-compliance") {
			if !l.is_builtin_type() {
				return Err(Error::TypeError(l.typename()));
			}

			if !r.is_builtin_type() {
				return Err(Error::TypeError(r.typename()));
			}
		}

		(l == r).into()
	}

	fn LESS_THAN('<', lhs, rhs) {
		match lhs.run(env)? {
			Value::Number(lnum) => (lnum < rhs.run(env)?.to_number()?).into(),
			Value::Boolean(lbool) => (!lbool & rhs.run(env)?.to_bool()?).into(),
			Value::SharedStr(ltext) => (ltext < rhs.run(env)?.to_knstr()?).into(),
			other => return Err(Error::TypeError(other.typename()))
		}
	}

	fn GREATER_THAN('>', lhs, rhs) {
		match lhs.run(env)? {
			Value::Number(lnum) => (lnum > rhs.run(env)?.to_number()?).into(),
			Value::Boolean(lbool) => (lbool & !rhs.run(env)?.to_bool()?).into(),
			Value::SharedStr(ltext) => (ltext > rhs.run(env)?.to_knstr()?).into(),
			other => return Err(Error::TypeError(other.typename()))
		}
	}

	fn AND('&', lhs, rhs) {
		let l = lhs.run(env)?;

		if l.to_bool()? {
			rhs.run(env)?
		} else {
			l
		}
	}

	fn OR('|', lhs, rhs) {
		let l = lhs.run(env)?;

		if l.to_bool()? {
			l
		} else {
			rhs.run(env)?
		}
	}

	fn THEN(';', lhs, rhs) {
		lhs.run(env)?;
		rhs.run(env)?
	}

	fn ASSIGN('=', var, value) {
		let variable = if let Value::Variable(v) = var {
			v
		} else {
			return Err(Error::TypeError(var.typename()));
		};

		let ran = value.run(env)?;
		variable.assign(ran.clone());
		ran
	}

	fn WHILE('W', cond, body) {
		while cond.run(env)?.to_bool()? {
			body.run(env)?;
		}

		Value::Null
	}

	fn IF('I', cond, iftrue, iffalse) {
		if cond.run(env)?.to_bool()? {
			iftrue.run(env)?
		} else {
			iffalse.run(env)?
		}
	}

	fn GET('G', string, start, length) {
		let string = string.run(env)?.to_knstr()?;
		let start = start.run(env)?.to_number()?.try_conv::<usize>().expect("todo");
		let length = length.run(env)?.to_number()?.try_conv::<usize>().expect("todo");

		// lol, todo, optimize me
		string
			.get(start..=start + length)
			.expect("todo: error for out of bounds")
			.to_boxed()
			.conv::<SharedStr>()
			.into()
	}

	fn SUBSTITUTE('S', string, start, length, replacement) {
		let string = string.run(env)?.to_knstr()?;
		let start = start.run(env)?.to_number()?.try_conv::<usize>().expect("todo");
		let length = length.run(env)?.to_number()?.try_conv::<usize>().expect("todo");
		let replacement = replacement.run(env)?.to_knstr()?;

		// lol, todo, optimize me
		let mut s = String::new();
		s.push_str(&string[..start]);
		s.push_str(&replacement);
		s.push_str(&string[start..]);
		s.try_conv::<SharedStr>().unwrap().into()
	}
}

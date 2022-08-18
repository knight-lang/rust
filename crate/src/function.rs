use crate::{value::Number, Environment, Error, Result, Text, Value};
use std::fmt::{self, Debug, Formatter};
use std::io::Write;
use tap::prelude::*;

#[derive(Clone, Copy)]
pub struct Function {
	/// The code associated with this function
	pub func: fn(&[Value], &mut Environment<'_>) -> Result<Value>,

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
		todo!();
	}

	fn RANDOM ('R') {
		todo!();
	}

	fn EVAL ('E', arg) {
		let _ = arg; todo!();
	}

	fn BLOCK ('B', arg) {
		arg.clone()
	}

	fn CALL ('C', arg) {
		arg.run(env)?.run(env)?
	}

	fn SYSTEM ('`', arg) {
		let _ = arg; todo!();
	}

	fn QUIT ('Q', arg) {
		let _ = arg; todo!();
	}

	fn NOT ('!', arg) {
		arg.run(env)?
			.to_bool()?
			.pipe(|x| !x)
			.into()
	}

	fn LENGTH ('L', arg) {
		arg.run(env)?
			.to_text()?
			.len()
			.pipe(|x| x as Number)
			.into()
	}

	fn DUMP ('D', arg) {
		let value = arg.run(env)?;
		writeln!(env, "{value:?}")?;
		value
	}

	fn OUTPUT ('O', arg) {
		let text = arg.run(env)?.to_text()?;

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
					.and_then(|chr| Text::new(chr).ok())
					.ok_or(Error::DomainError("number isn't in bounds"))?
					.into(),

			Value::Text(text) =>
				text.chars()
					.next()
					.ok_or(Error::DomainError("Empty string"))?
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
			Value::Text(_text) => todo!(),
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
			Value::Text(_text) => todo!(),
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
			Value::Text(_text) => todo!(),
			other => return Err(Error::TypeError(other.typename()))
		}
	}

	fn GREATER_THAN('>', lhs, rhs) {
		let _ = (lhs, rhs); todo!();
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
		if let Value::Variable(v) = var {
			let r = value.run(env)?;
			v.assign(r.clone());
			r
		} else {
			return Err(Error::TypeError(var.typename()));
		}
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
		let _ = (string, start, length); todo!();
	}

	fn SUBSTITUTE('S', string, start, length, replacement) {
		let _ = (string, start, length, replacement); todo!();
	}
}

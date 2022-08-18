use crate::{value::Number, Environment, Error, Result, Text, Value};
use std::fmt::{self, Debug, Formatter};
use std::io::Write;

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
		(!arg.run(env)?.to_bool()?).into()
	}

	fn LENGTH ('L', arg) {
		(arg.run(env)?.to_text()?.len() as Number).into()
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

		Default::default()
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

			Value::Text(text) => (text.chars().next()
				.ok_or(Error::DomainError("Empty string"))? as Number).into(),

			other => return Err(Error::TypeError(ASCII.name, other.typename()))
		}
	}


	fn NEG ('~', arg) {
		arg.run(env)?
			.to_number()?
			.checked_neg()
			.ok_or(Error::IntegerOverflow)?
			.into()
	}

}

// insert!('+', 2, add);
// insert!('-', 2, subtract);
// insert!('*', 2, multiply);
// insert!('/', 2, divide);
// insert!('%', 2, modulo);
// insert!('^', 2, power);
// insert!('?', 2, equals);
// insert!('<', 2, less_than);
// insert!('>', 2, greater_than);
// insert!('&', 2, and);
// insert!('|', 2, or);
// insert!(';', 2, then);
// insert!('=', 2, assign);
// insert!('W', 2, r#while);

// insert!('I', 3, r#if);
// insert!('G', 3, get);
// insert!('S', 4, substitute);

// #[cfg(feature = "variable-lookup")]
// insert!('V', 1, variable_lookup);

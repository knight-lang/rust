use crate::value::text::{Character, TextSlice};
use crate::value::{List, Runnable, Text, ToBoolean, ToInteger, ToText};
use crate::{Environment, Error, Result, Value};
use std::cmp::Ordering;
use std::collections::HashMap;

use std::fmt::{self, Debug, Formatter};

/// A runnable function in Knight, e.g. `+`.
#[derive(Clone, Copy)]
pub struct Function {
	/// The code associated with this function
	pub func: for<'e> fn(&[Value<'e>], &mut Environment<'e>) -> Result<Value<'e>>,

	/// The long-hand name of this function.
	///
	/// For extension functions that start with `X`, this should also start with it.
	pub name: &'static TextSlice,

	/// The arity of the function, i.e. how many arguments it takes.
	pub arity: usize,
}

impl Function {
	/// Gets the shorthand name for `self`. Returns `None` if it's an `X` function.
	pub fn short_form(&self) -> Option<Character> {
		match self.name.chars().next() {
			Some(c) if c == 'X' => None,
			other => other,
		}
	}
}

impl Eq for Function {}
impl PartialEq for Function {
	/// Functions are only equal if they're identical.
	fn eq(&self, rhs: &Self) -> bool {
		std::ptr::eq(self, rhs)
	}
}

impl Debug for Function {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Function({})", self.name)
	}
}

pub(crate) fn default() -> HashMap<Character, &'static Function> {
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
		ADD SUBTRACT MULTIPLY DIVIDE REMAINDER POWER EQUALS LESS_THAN GREATER_THAN AND OR
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

pub(crate) fn extensions() -> HashMap<Text, &'static Function> {
	#[allow(unused_mut)]
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
				Ok($body)
			}
		}
	};
}

/// **4.1.4**: `PROMPT`
pub static PROMPT: Function = function!("PROMPT", env, |/* comment for rustfmt */| {
	// env.prompt().read_line(env)?.map(Value::from).unwrap_or_default()
	let _ = env;
	todo!();
	#[allow(unreachable_code)]
	Value::Null
});

/// **4.1.5**: `RANDOM`
pub static RANDOM: Function = function!("RANDOM", env, |/* comment for rustfmt */| {
	// note that `env.random()` is seedable with `XSRAND`
	env.random().into()
});

/// **4.2.2** `BOX`
pub static BOX: Function = function!(",", env, |val| {
	// `boxed` is optimized over `vec![val.run(env)]`
	List::boxed(val.run(env)?).into()
});

pub static HEAD: Function = function!("[", env, |val| {
	// <comment for a single line>
	val.run(env)?.head(env)?
});

pub static TAIL: Function = function!("]", env, |val| {
	// <comment for a single line>
	val.run(env)?.tail(env)?
});

/// **4.2.3** `BLOCK`  
pub static BLOCK: Function = function!("BLOCK", _env, |arg| {
	// Technically, according to the spec, only the return value from `BLOCK` can be used in `CALL`.
	// Since this function normally just returns whatever it's argument is, it's impossible to
	// distinguish an `Integer` returned from `BLOCK` and one simply given to `CALL`. As such, when
	// the `strict-call-argument` feature is enabled. We ensure that we _only_ return `Ast`s
	// from `BLOCK`, so `CALL` can verify them.
	#[cfg(not(feature = "lax-compliance"))]
	if !matches!(arg, Value::Ast(_)) {
		// The NOOP function literally just runs its argument.
		static NOOP: Function = function!(":", env, |arg| {
			debug_assert!(!matches!(arg, Value::Ast(_)));

			arg.run(env)? // We can't simply `.clone()` the arg in case we're given a variable name.
		});

		return Ok(crate::Ast::new(&NOOP, vec![arg.clone()].into()).into());
	}

	arg.clone()
});

/// **4.2.4** `CALL`  
pub static CALL: Function = function!("CALL", env, |arg| {
	let block = arg.run(env)?;

	// When ensuring that `CALL` is only given values returned from `BLOCK`, we must ensure that all
	// arguments are `Value::Ast`s.
	#[cfg(not(feature = "lax-compliance"))]
	if !matches!(block, Value::Ast(_)) {
		return Err(Error::TypeError(block.typename(), "CALL"));
	}

	block.run(env)?
});

/// **4.2.6** `QUIT`  
pub static QUIT: Function = function!("QUIT", env, |arg| {
	match i32::try_from(arg.run(env)?.to_integer()?) {
		Ok(status) if !env.flags().compliance.check_quit_bounds || (0..=127).contains(&status) => {
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
pub static NOT: Function = function!("!", env, |arg| {
	// <blank line so rustfmt doesnt wrap onto the prev line>
	(!arg.run(env)?.to_boolean()?).into()
});

/// **4.2.8** `LENGTH`  
pub static LENGTH: Function = function!("LENGTH", env, |arg| {
	//
	arg.run(env)?.length(env)?
});

/// **4.2.9** `DUMP`  
pub static DUMP: Function = function!("DUMP", env, |arg| {
	let value = arg.run(env)?;
	write!(env.stdout(), "{value:?}")?;
	value
});

/// **4.2.10** `OUTPUT`  
pub static OUTPUT: Function = function!("OUTPUT", env, |arg| {
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
pub static ASCII: Function = function!("ASCII", env, |arg| {
	//
	arg.run(env)?.ascii(env)?
});

/// **4.2.12** `~`  
pub static NEG: Function = function!("~", env, |arg| {
	// comment so it wont make it one line
	arg.run(env)?.to_integer()?.negate()?.into()
});

/// **4.3.1** `+`  
pub static ADD: Function = function!("+", env, |lhs, rhs| {
	//
	lhs.run(env)?.add(&rhs.run(env)?, env)?
});

/// **4.3.2** `-`  
pub static SUBTRACT: Function = function!("-", env, |lhs, rhs| {
	//
	lhs.run(env)?.subtract(&rhs.run(env)?, env)?
});

/// **4.3.3** `*`  
pub static MULTIPLY: Function = function!("*", env, |lhs, rhs| {
	//
	lhs.run(env)?.multiply(&rhs.run(env)?, env)?
});

/// **4.3.4** `/`  
pub static DIVIDE: Function = function!("/", env, |lhs, rhs| {
	//
	lhs.run(env)?.divide(&rhs.run(env)?, env)?
});

/// **4.3.5** `%`  
pub static REMAINDER: Function = function!("%", env, |lhs, rhs| {
	//
	lhs.run(env)?.remainder(&rhs.run(env)?, env)?
});

/// **4.3.6** `^`  
pub static POWER: Function = function!("^", env, |lhs, rhs| {
	//
	lhs.run(env)?.power(&rhs.run(env)?, env)?
});

/// **4.3.7** `<`  
pub static LESS_THAN: Function = function!("<", env, |lhs, rhs| {
	(lhs.run(env)?.compare(&rhs.run(env)?, env)? == Ordering::Less).into()
});

/// **4.3.8** `>`  
pub static GREATER_THAN: Function = function!(">", env, |lhs, rhs| {
	(lhs.run(env)?.compare(&rhs.run(env)?, env)? == Ordering::Greater).into()
});

/// **4.3.9** `?`  
pub static EQUALS: Function = function!("?", env, |lhs, rhs| {
	//
	lhs.run(env)?.equals(&rhs.run(env)?, env)?.into()
});

/// **4.3.10** `&`  
pub static AND: Function = function!("&", env, |lhs, rhs| {
	let condition = lhs.run(env)?;

	if condition.to_boolean()? {
		return rhs.run(env);
	}

	condition
});

/// **4.3.11** `|`  
pub static OR: Function = function!("|", env, |lhs, rhs| {
	let condition = lhs.run(env)?;

	if !condition.to_boolean()? {
		return rhs.run(env);
	}

	condition
});

/// **4.3.12** `;`  
pub static THEN: Function = function!(";", env, |lhs, rhs| {
	lhs.run(env)?;
	rhs.run(env)?
});

/// **4.3.13** `=`  
pub static ASSIGN: Function = function!("=", env, |variable, value| {
	let ran = value.run(env)?;
	variable.assign(ran.clone(), env)?;
	ran
});

/// **4.3.14** `WHILE`  
pub static WHILE: Function = function!("WHILE", env, |condition, body| {
	while condition.run(env)?.to_boolean()? {
		body.run(env)?;
	}

	Value::Null
});

/// **4.4.1** `IF`  
pub static IF: Function = function!("IF", env, |condition, iftrue, iffalse| {
	if condition.run(env)?.to_boolean()? {
		iftrue.run(env)?
	} else {
		iffalse.run(env)?
	}
});

/// **4.4.2** `GET`  
pub static GET: Function = function!("GET", env, |source, start, length| {
	//
	source.run(env)?.get(&start.run(env)?, &length.run(env)?, env)?
});

/// **4.5.1** `SET`  
pub static SET: Function = function!("SET", env, |source, start, length, replacement| {
	//
	source.run(env)?.set(&start.run(env)?, &length.run(env)?, &replacement.run(env)?, env)?
});

/// **6.1** `VALUE`
#[cfg(feature = "value-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "value-function")))]
pub static VALUE: Function = function!("VALUE", env, |arg| {
	let name = arg.run(env)?.to_text()?;
	env.lookup(&name)?.into()
});

#[cfg(feature = "handle-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "handle-function")))]
pub static HANDLE: Function = function!("HANDLE", env, |block, iferr| {
	const ERR_VAR_NAME: &crate::TextSlice = unsafe { crate::TextSlice::new_unchecked("_") };

	match block.run(env) {
		Ok(value) => value,
		Err(err) => {
			// This is fallible, as the error string might have had something bad.
			let errmsg = Text::try_from(err.to_string())?;

			// Assign it to the error variable
			env.lookup(ERR_VAR_NAME).unwrap().assign(errmsg.into());

			// Finally, execute the RHS.
			iferr.run(env)?
		}
	}
});

#[cfg(feature = "yeet-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "yeet-function")))]
pub static YEET: Function = function!("YEET", env, |errmsg| {
	return Err(Error::Custom(errmsg.run(env)?.to_text()?.to_string().into()));

	#[allow(unreachable_code)]
	Value::Null
});

/// **6.3** `USE`
#[cfg(feature = "use-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "use-function")))]
pub static USE: Function = function!("USE", env, |arg| {
	let filename = arg.run(env)?.to_text()?;
	let contents = env.read_file(&filename)?;

	env.play(&contents)?
});

/// **4.2.2** `EVAL`
#[cfg(feature = "eval-function")]
pub static EVAL: Function = function!("EVAL", env, |val| {
	let code = val.run(env)?.to_text()?;
	env.play(&code)?
});

#[cfg(feature = "extensions")]
/// **4.2.5** `` ` ``
pub static SYSTEM: Function = function!("$", env, |cmd, stdin| {
	let command = cmd.run(env)?.to_text()?;
	let stdin = match stdin.run(env)? {
		Value::Text(text) => Some(text),
		Value::Null => None,
		other => return Err(Error::TypeError(other.typename(), "$")),
	};

	env.run_command(&command, stdin.as_deref())?.into()
});

/// **Compiler extension**: SRAND
#[cfg(feature = "xsrand-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "xsrand-function")))]
pub static XSRAND: Function = function!("XSRAND", env, |arg| {
	let seed = arg.run(env)?.to_integer()?;
	env.srand(seed);
	Value::Null
});

/// **Compiler extension**: REV
#[cfg(feature = "xreverse-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "xreverse-function")))]
pub static XREVERSE: Function = function!("XREVERSE", env, |arg| {
	match arg.run(env)? {
		Value::Text(text) => {
			text.chars().collect::<Vec<Character>>().into_iter().rev().collect::<Text>().into()
		}
		Value::List(list) => {
			let mut eles = list.iter().cloned().collect::<Vec<Value<'_>>>();
			eles.reverse();
			List::try_from(eles).unwrap().into()
		}
		other => return Err(Error::TypeError(other.typename(), "XRANGE")),
	}
});

#[cfg(feature = "xrange-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "xrange-function")))]
pub static XRANGE: Function = function!("XRANGE", env, |start, stop| {
	match start.run(env)? {
		Value::Integer(start) => {
			let stop = stop.run(env)?.to_integer()?;

			match start <= stop {
				true => List::try_from(
					(i64::from(start)..i64::from(stop))
						.map(|x| Value::from(Integer::try_from(x).unwrap()))
						.collect::<Vec<Value<'_>>>(),
				)
				.expect("todo: out of bounds error")
				.into(),

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

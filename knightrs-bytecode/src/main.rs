#![allow(unused)]

use std::default;
use std::path::Path;

use knightrs_bytecode::env::Environment;
use knightrs_bytecode::parser::*;
use knightrs_bytecode::program::*;
use knightrs_bytecode::strings::KnStr;
use knightrs_bytecode::value::*;
use knightrs_bytecode::vm::*;
use knightrs_bytecode::Options;

fn run(
	env: &mut Environment,
	program: &str,
	argv: impl Iterator<Item = String>,
) -> Result<(), String> {
	let mut parser = Parser::new(env, Some(Path::new("-e")), &program).map_err(|s| s.to_string())?;

	let program = parser.parse_program().map_err(|err| err.to_string())?;

	// dbg!(&program);

	Vm::new(&program, env).run_entire_program(argv).map_err(|e| e.to_string()).and(Ok(()))
}

fn main() {
	use knightrs_bytecode::gc::*;
	use knightrs_bytecode::value2 as v2;
	let mut gc = Gc::new(Default::default());

	let mut greeting = v2::Value::from(v2::KnString::new(
		KnStr::new_unvalidated("hello worldhello worldhello worldhello worldhello worldhello world"),
		&gc,
	));

	let mut list = v2::Value::from(v2::List::boxed(greeting, &gc));

	gc.add_root(list);

	dbg!(list);

	unsafe {
		gc.shutdown();
	}
}

fn main2() {
	let mut env = Environment::new({
		let mut opts = Options::default();
		#[cfg(feature = "extensions")]
		{
			// opts.extensions.negative_indexing = true;
			opts.extensions.argv = true;
			opts.extensions.functions.eval = true;
			opts.extensions.functions.value = true;
			opts.extensions.builtin_fns.assign_to_strings = true;
			opts.extensions.builtin_fns.assign_to_random = true;
			opts.extensions.syntax.control_flow = true;
		}
		#[cfg(feature = "compliance")]
		{
			opts.compliance.check_container_length = true;
			opts.compliance.i32_integer = true;
			opts.compliance.check_overflow = true;
			opts.compliance.check_integer_function_bounds = true;
			opts.compliance.variable_name_length = true;
			opts.compliance.variable_count = true;
			opts.compliance.forbid_trailing_tokens = true;
			opts.compliance.check_equals_params = true;
			opts.compliance.limit_rand_range = true;
			opts.compliance.cant_dump_blocks = true;
			opts.compliance.check_quit_status_codes = true;
			opts.compliance.disallow_negative_int_to_list = true;
			opts.qol.check_parens = true;
		}

		opts
	});

	let mut args = std::env::args().skip(1);
	let program = match args.next().as_deref() {
		Some("-f") => std::fs::read_to_string(args.next().expect("missing expr for -f"))
			.expect("cannot open file"),
		Some("-e") => args.next().expect("missing expr for -e"),
		_ => panic!("invalid option: -e or -f only"),
	};

	match run(&mut env, &program, args) {
		Ok(()) => {}
		Err(err) => {
			eprintln!("error: {err}");
			std::process::exit(1)
		}
	}
}

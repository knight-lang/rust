#![allow(unused)]

use std::path::Path;

use knightrs_bytecode::env::Environment;
use knightrs_bytecode::parser::*;
use knightrs_bytecode::program::*;
use knightrs_bytecode::strings::StringSlice;
use knightrs_bytecode::value::*;
use knightrs_bytecode::vm::*;
use knightrs_bytecode::Options;

fn run(env: &mut Environment, program: &str) -> Result<(), String> {
	let mut parser = Parser::new(env, Some(Path::new("-e")), &program).map_err(|s| s.to_string())?;

	let program = parser.parse_program().map_err(|err| err.to_string())?;

	Vm::new(&program, env).run_entire_program().map_err(|e| e.to_string()).and(Ok(()))
}

fn main() {
	let mut env = Environment::new({
		let mut opts = Options::default();
		#[cfg(feature = "extensions")]
		{
			// opts.extensions.negative_indexing = true;
			opts.extensions.eval = true;
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

	match run(&mut env, &program) {
		Ok(()) => {}
		Err(err) => eprintln!("error: {err}"),
	}
}

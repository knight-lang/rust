#![allow(unused)]
mod cli;

use std::default;
use std::path::Path;

use cli::CliOpts;
use knightrs_bytecode::env::Environment;
use knightrs_bytecode::gc::Gc;
use knightrs_bytecode::parser::*;
use knightrs_bytecode::program::*;
use knightrs_bytecode::strings::KnStr;
use knightrs_bytecode::value::*;
use knightrs_bytecode::value::{ToKnString, ToList};
use knightrs_bytecode::vm::*;
use knightrs_bytecode::Options;
use source_location::ProgramSource;

fn run(
	env: &mut Environment<'_>,
	source: ProgramSource<'_>,
	program: &str,
	argv: impl Iterator<Item = String>,
) -> Result<(), String> {
	let gc = env.gc();
	let mut parser = Parser::new(env, source, &program).map_err(|s| s.to_string())?;

	gc.pause();
	let program = parser.parse_program().map_err(|err| err.to_string())?;

	// dbg!(&program);

	let mut vm = Vm::new(&program, env);
	// gc.add_mark_fn(|| vm.mark());
	gc.unpause();

	vm.run_entire_program(argv).map_err(|e| e.to_string()).and(Ok(()))
}

fn main1() {
	use knightrs_bytecode::gc::*;
	use knightrs_bytecode::value as v2;
	let gc = Gc::new(Default::default());
	unsafe {
		gc.run(|gc| {
			let mut env = Environment::new(Default::default(), &gc);

			let greeting = v2::KnString::new_unvalidated(
				"hello worldhello worldhello worldhello worldhello worldhello world".into(),
				&gc,
			);

			dbg!(greeting.to_list(&mut env));

			let mut list = unsafe {
				greeting.with_inner(|greeting| {
					let list = v2::List::boxed(greeting.into(), &gc);
					list
					// v2::Value::from(list.make_permanent())
				})
			};

			// dbg!(list.make_permanent().to_knstring(env));

			dbg!(list);
		})
	}

	// 	let int = v2::Integer::new_unvalidated(1234);
	// 	let int_str = int.to_knstring(&mut env).unwrap();

	// 	unsafe {
	// 		gc.mark_and_sweep();
	// 	}

	// 	dbg!(*int_str);
}

fn main() {
	let cliopts = CliOpts::from_argv();

	unsafe {
		let gc = Gc::default();
		gc.run(|gc| {
			let mut env = Environment::new(
				cliopts.options().clone(), // TODO: remove this clone
				/*{
					let mut opts = Options::default();
					#[cfg(feature = "check-variables")]
					{
						opts.check_variables = true;
					}

					#[cfg(feature = "check-parens")]
					{
						opts.check_parens = true;
					}

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
						opts.compliance.limit_rand_range = true;
						opts.compliance.check_quit_status_codes = true;
						opts.compliance.strict_conversions = true;
						opts.compliance.disable_all_extensions = true;
						opts.compliance.no_block_conversions = true;
					}

					opts
				},*/
				&gc,
			);

			// TODO: args
			for maybe_oops in cliopts.source_iter() {
				if let Err(err) = maybe_oops.map(|(program, source)| {
					run(&mut env, source, &program, cliopts.argv()).map_err(|s| s.to_string())
				}) {
					eprintln!("error: {err}");
					std::process::exit(1);
				}
			}
			// 	}
			// 	let mut args = std::env::args().skip(1);
			// 	let program = expr;
			// 	// let (program, source) = match args.next().as_deref() {
			// 	// 	Some("-f") => (
			// 	// 		std::fs::read_to_string(args.next().expect("missing expr for -f"))
			// 	// 			.expect("cannot open file"),
			// 	// 		ProgramSource::File(Path::new("<file>")),
			// 	// 	),
			// 	// 	Some("-e") => (args.next().expect("missing expr for -e"), ProgramSource::ExprFlag),
			// 	// 	_ => panic!("invalid option: -e or -f only"),
			// 	// };
			// 	let source = ProgramSource::ExprFlag;

			// 	match run(&mut env, source, &program, args) {
			// 		Ok(()) => {}
			// 		Err(err) => {
			// 			eprintln!("error: {err}");
			// 			std::process::exit(1)
			// 		}
			// 	}
		});
	}
}

use std::path::PathBuf;

use clap::{
	arg, command, error, value_parser, Arg, ArgAction, Args, Command, CommandFactory, Parser,
};
use knightrs_bytecode::{
	parser::source_location::ProgramSource,
	strings::{Encoding, KnStr},
	value::KnString,
	Options, Result,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
	/// The expression to execute; multiple may be provided, and are run in-order.
	///
	/// Mutually exclusive with `file`.
	#[arg(short, long, overrides_with = "file")]
	expression: Vec<String>,

	/// The list of files to read programs from. Mutually exclusive with `-e`.
	#[arg(short, long)]
	file: Vec<PathBuf>,

	/// Additional arguments
	#[arg(trailing_var_arg = true)]
	argv: Vec<String>,
	// .next_help_heading(heading)

	/***************************************************************************
	 *                                Debugger                                 *
	 ***************************************************************************/
	/// Enable all debugger features
	#[arg(short, long, overrides_with = "_no_debugger")]
	debugger: bool,
	/// Undoes debugger
	#[arg(long, hide_short_help = true)]
	_no_debugger: bool, // underscore because nothing checks for it

	/// Print out stacktraces
	#[arg(long, hide_short_help = true, overrides_with = "no_stacktrace")]
	stacktrace: bool,
	/// Undoes stacktrace
	#[arg(long, hide_short_help = true)]
	no_stacktrace: bool,

	/// Ensure variables are always assigned
	#[arg(long, hide_short_help = true, overrides_with = "no_check_variables")]
	check_variables: bool,
	/// Undoes check_variables
	#[arg(long, hide_short_help = true)]
	no_check_variables: bool,

	/// Ensure parens are always balanced
	#[arg(long, hide_short_help = true, overrides_with = "no_check_parens")]
	check_parens: bool,
	/// Undoes check_parens
	#[arg(long, hide_short_help = true)]
	no_check_parens: bool,

	/***************************************************************************
	 *                               Compliance                                *
	 ***************************************************************************/
	/// Enable all compliance features
	///
	/// Unlike --strict-compliance, this does not disable extensions
	#[arg(short = 'c', long, overrides_with = "_no_compliance")]
	compliance: bool,
	/// Undoes a previous --compliance
	#[arg(long, hide_short_help = true)]
	_no_compliance: bool, // underscore because nothing checks for it

	/// Like --compliance, except non-compliant flags are ignored
	///
	/// Unlike --strict-compliance, this does not disable extensions
	#[arg(short = 'C', long, overrides_with = "_no_strict_compliance")]
	strict_compliance: bool,
	/// Undoes a previous --strict_compliance
	#[arg(long, hide_short_help = true)]
	_no_strict_compliance: bool, // underscore because nothing checks for it

	/// Ensure containers are at most INTMAX elements long.
	#[arg(long, hide_short_help = true, overrides_with = "no_check_container_len")]
	check_container_len: bool,
	/// Undoes check_container_len
	#[arg(long, hide_short_help = true)]
	no_check_container_len: bool,

	/// Constrain integers to 32 bits
	#[arg(
		short = '3',
		long = "32-bit-int",
		hide_short_help = true,
		overrides_with = "no_i32_bit_ints"
	)]
	i32_bit_ints: bool,
	/// Undoes i32_integers
	#[arg(long = "no-32-bit-int", hide_short_help = true)]
	no_i32_bit_ints: bool,

	/// Check for overflow in arithmetic operations
	#[arg(long, hide_short_help = true, overrides_with = "no_check_overflow")]
	check_overflow: bool,
	/// Undoes check_overflow
	#[arg(long, hide_short_help = true)]
	no_check_overflow: bool,

	/// Check function bounds for integer functions.
	// TODO: maybe have this, along with `check_overflow` be function-specific?
	#[arg(long, hide_short_help = true, overrides_with = "no_check_int_fn_bounds")]
	check_int_fn_bounds: bool,
	/// Undoes check_int_fn_bounds
	#[arg(long, hide_short_help = true)]
	no_check_int_fn_bounds: bool,

	/// Ensure variables are at most 127 chars long
	#[arg(long, hide_short_help = true, overrides_with = "no_validate_variable_name_len")]
	validate_variable_name_len: bool,
	/// Undoes validate_variable_name_len
	#[arg(long, hide_short_help = true)]
	no_validate_variable_name_len: bool,

	/// Ensure at most 65535 variables are used
	#[arg(long, hide_short_help = true, overrides_with = "no_validate_variable_count")]
	validate_variable_count: bool,
	/// Undoes validate_variable_count
	#[arg(long, hide_short_help = true)]
	no_validate_variable_count: bool,

	/// Require programs to be exactly one token long.
	///
	/// Without this option, trailing tokens are ignored
	#[arg(long, hide_short_help = true, overrides_with = "no_forbid_trailing_tokens")]
	forbid_trailing_tokens: bool,
	/// Undoes forbid_trailing_tokens
	#[arg(long, hide_short_help = true)]
	no_forbid_trailing_tokens: bool,

	/// Only support BLOCKs in functions that the spec permits.
	///
	/// Without this, a handful of functions (such as `?` and `DUMP`) support them.
	#[arg(long, hide_short_help = true, overrides_with = "no_strict_blocks")]
	strict_blocks: bool,
	/// Undoes strict_blocks
	#[arg(long, hide_short_help = true)]
	no_strict_blocks: bool,

	/// Don't allow blocks to be converted to other types
	#[arg(long, hide_short_help = true, overrides_with = "no_no_block_conversions")]
	no_block_conversions: bool,
	/// Undoes no_block_conversions
	#[arg(long, hide_short_help = true)]
	no_no_block_conversions: bool,

	/// Limit `RANDOM` to be only from 0 to 32767
	#[arg(long, hide_short_help = true, overrides_with = "no_limit_random_range")]
	limit_random_range: bool,
	/// Undoes limit_random_range
	#[arg(long, hide_short_help = true)]
	no_limit_random_range: bool,

	/// Require `QUIT` to be called with ints from 0 to 127.
	#[arg(long, hide_short_help = true, overrides_with = "no_check_quit_status_code")]
	check_quit_status_code: bool,
	/// Undoes check_quit_status_code
	#[arg(long, hide_short_help = true)]
	no_check_quit_status_code: bool,

	/// Forbid some conversions (such as boolean -> list) that are allowed as an extension.
	#[arg(long, hide_short_help = true, overrides_with = "no_strict_conversions")]
	strict_conversions: bool,
	/// Undoes strict_conversions
	#[arg(long, hide_short_help = true)]
	no_strict_conversions: bool,

	/***************************************************************************
	 *                               Extensions                                *
	 ***************************************************************************/
	/// Enables all extensions
	#[arg(short = 'E', long, overrides_with = "_no_extensions")]
	extensions: bool,
	/// Undoes a previous --extensions
	#[arg(long, hide_short_help = true)]
	_no_extensions: bool, // underscore because nothing checks for it

	/// Enable all extension types. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_types")]
	ext_types: bool,
	/// Undoes ext_types
	#[arg(long, hide_short_help = true)]
	no_ext_types: bool,

	/// Enable floats. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_float")]
	ext_float: bool,
	/// Undoes ext_float
	#[arg(long, hide_short_help = true)]
	no_ext_float: bool,

	/// Enable hashmaps. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_hashmap")]
	ext_hashmap: bool,
	/// Undoes ext_hashmap
	#[arg(long, hide_short_help = true)]
	no_ext_hashmap: bool,

	/// Enable classes. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_classes")]
	ext_classes: bool,
	/// Undoes ext_classes
	#[arg(long, hide_short_help = true)]
	no_ext_classes: bool,

	/// Enable negative indexing
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_neg_indexing")]
	ext_neg_indexing: bool,
	/// Undoes ext_neg_indexing
	#[arg(long, hide_short_help = true)]
	no_ext_neg_indexing: bool,

	/// Enable the extension functions. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_fns")]
	ext_fns: bool,
	/// Undoes ext_fns
	#[arg(long, hide_short_help = true)]
	no_ext_fns: bool,

	/// Enable the EVAL function.
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_fn_eval")]
	ext_fn_eval: bool,
	/// Undoes ext_fn_eval
	#[arg(long, hide_short_help = true)]
	no_ext_fn_eval: bool,

	/// Enable the VALUE function.
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_fn_value")]
	ext_fn_value: bool,
	/// Undoes ext_fn_value
	#[arg(long, hide_short_help = true)]
	no_ext_fn_value: bool,

	/// Enable the ` function.
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_fn_system")]
	ext_fn_system: bool,
	#[arg(long, hide_short_help = true)]
	no_ext_fn_system: bool,

	/// Add support for the `_argv` variable, which is additional arguments on the cli.
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_argv")]
	ext_argv: bool,
	/// Undoes ext_argv
	#[arg(long, hide_short_help = true)]
	no_ext_argv: bool,

	/// Enable "breaking changes"
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_breaking_changes")]
	ext_breaking_changes: bool,
	/// Undoes ext_breaking_changes
	#[arg(long, hide_short_help = true)]
	no_ext_breaking_changes: bool,

	/// UNIMPLEMENTED; enable `~COLL` to reverse it.
	#[arg(
		long,
		hide_short_help = true,
		overrides_with = "no_ext_breaking_changes_negate_rev_collection"
	)]
	ext_breaking_changes_negate_rev_collection: bool,
	/// Undoes ext_breaking_changes_negate_rev_collection
	#[arg(long, hide_short_help = true)]
	no_ext_breaking_changes_negate_rev_collection: bool,

	/// UNIMPLEMENTED; enable `~COLL` to reverse it.
	#[arg(
		long,
		hide_short_help = true,
		overrides_with = "no_ext_breaking_changes_rand_can_be_negative"
	)]
	ext_breaking_changes_rand_can_be_negative: bool,
	/// Undoes ext_breaking_changes_rand_can_be_negative
	#[arg(long, hide_short_help = true)]
	no_ext_breaking_changes_rand_can_be_negative: bool,

	/// Enable list literals
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_list_literal")]
	ext_list_literal: bool,
	/// Undoes ext_list_literal
	#[arg(long, hide_short_help = true)]
	no_ext_list_literal: bool,

	/// Enables string interpolation
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_string_interpolation")]
	ext_string_interpolation: bool,
	/// Undoes ext_string_interpolation
	#[arg(long, hide_short_help = true)]
	no_ext_string_interpolation: bool,

	/// Enables control flow functions like XBREAK, XCONTINUE, XRETURN
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_control_flow")]
	ext_control_flow: bool,
	/// Undoes ext_control_flow
	#[arg(long, hide_short_help = true)]
	no_ext_control_flow: bool,

	/// Enables boolean extension functions
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_builtin_fns_boolean")]
	ext_builtin_fns_boolean: bool,
	/// Undoes ext_builtin_fns_boolean
	#[arg(long, hide_short_help = true)]
	no_ext_builtin_fns_boolean: bool,

	/// Enables string extension functions
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_builtin_fns_string")]
	ext_builtin_fns_string: bool,
	/// Undoes ext_builtin_fns_string
	#[arg(long, hide_short_help = true)]
	no_ext_builtin_fns_string: bool,

	/// Enables list extension functions
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_builtin_fns_list")]
	ext_builtin_fns_list: bool,
	/// Undoes ext_builtin_fns_list
	#[arg(long, hide_short_help = true)]
	no_ext_builtin_fns_list: bool,

	/// Enables integer extension functions
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_builtin_fns_integer")]
	ext_builtin_fns_integer: bool,
	/// Undoes ext_builtin_fns_integer
	#[arg(long, hide_short_help = true)]
	no_ext_builtin_fns_integer: bool,

	/// Enables null extension functions
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_builtin_fns_null")]
	ext_builtin_fns_null: bool,
	/// Undoes ext_builtin_fns_null
	#[arg(long, hide_short_help = true)]
	no_ext_builtin_fns_null: bool,

	/// Enables assigning to strings, which assigns to that variable
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_builtin_fns_assign_to_strings")]
	ext_builtin_fns_assign_to_strings: bool,
	/// Undoes ext_builtin_fns_assign_to_strings
	#[arg(long, hide_short_help = true)]
	no_ext_builtin_fns_assign_to_strings: bool,

	/// Enables assigning to random, which seeds it
	#[arg(long, hide_short_help = true, overrides_with = "no_ext_builtin_fns_assign_to_random")]
	ext_builtin_fns_assign_to_random: bool,
	/// Undoes ext_builtin_fns_assign_to_random
	#[arg(long, hide_short_help = true)]
	no_ext_builtin_fns_assign_to_random: bool,

	/***************************************************************************
	 *                                Embedded                                 *
	 ***************************************************************************/
	/// Enable all "embedded" features
	#[arg(long)]
	embedded: bool,

	/// Should exit when quitting the app
	#[arg(long, hide_short_help = true)]
	exit_when_quit: bool,
}

impl Cli {
	pub fn options(&self) -> clap::error::Result<Options> {
		let mut opts = Options::default();

		macro_rules! check_option {
			(
				feature = $feature:literal, default = $default:expr;
				$(opts.$($target:ident).+ = $yes:ident, $no:ident;)*
			) => {
				let default = $default;

				#[cfg(not(feature = $feature))]
				if default $(|| self.$yes)* {
					return Err(clap::Error::raw(
						clap::error::ErrorKind::ArgumentConflict,
						concat!("feature ", $feature, " is not enabled!")
					));
				}

				#[cfg(feature = $feature)]
				{
					$(
						opts.$($target).+ = if self.$yes {
							true
						} else if self.$no {
							false
						} else {
							default
						};
					)*
				}
			};
		}

		check_option! {
			feature = "debugger", default = self.debugger;

			opts.debugger.stacktrace = stacktrace, no_stacktrace;
			opts.check_parens = check_parens, no_check_parens;
			opts.check_variables = check_variables, no_check_variables;
		}

		check_option! {
			feature = "embedded", default = self.embedded;
			// opts.embedded.dont_exit_when_quitting = dont_exit_when_quitting, no_dont_exit_when_quitting;
		}

		check_option! {
			feature = "compliance", default = self.compliance || self.strict_compliance;

			opts.compliance.i32_integer = i32_bit_ints, no_i32_bit_ints;
			opts.compliance.check_overflow = check_overflow, no_check_overflow;
			opts.compliance.check_integer_function_bounds = check_int_fn_bounds, no_check_int_fn_bounds;
			opts.compliance.variable_name_length = validate_variable_name_len, no_validate_variable_name_len;
			opts.compliance.variable_count = validate_variable_count, no_validate_variable_count;
			opts.compliance.forbid_trailing_tokens = forbid_trailing_tokens, no_forbid_trailing_tokens;
			opts.compliance.strict_blocks = strict_blocks, no_strict_blocks;
			opts.compliance.no_block_conversions = no_block_conversions, no_no_block_conversions;
			opts.compliance.limit_rand_range = limit_random_range, no_limit_random_range;
			opts.compliance.check_quit_status_codes = check_quit_status_code, no_check_quit_status_code;
			opts.compliance.strict_conversions = strict_conversions, no_strict_conversions;
		}

		if !self.strict_compliance {
			check_option! {
				feature = "extensions", default = self.extensions;


				// opts.extensions.types.types = ext_types, no_ext_types;
				opts.extensions.types.floats = ext_float, no_ext_float;
				opts.extensions.types.hashmaps = ext_hashmap, no_ext_hashmap;
				opts.extensions.types.classes = ext_classes, no_ext_classes;
				opts.extensions.negative_indexing = ext_neg_indexing, no_ext_neg_indexing;

				// opts.extensions.functions = ext_fns, no_ext_fns;
				opts.extensions.functions.eval = ext_fn_eval, no_ext_fn_eval;
				opts.extensions.functions.value = ext_fn_value, no_ext_fn_value;
				opts.extensions.functions.system = ext_fn_system, no_ext_fn_system;
				opts.extensions.argv = ext_argv, no_ext_argv;

				// opts.extensions.breaking = ext_breaking_changes, no_ext_breaking_changes;
				opts.extensions.breaking.negate_reverses_collections = ext_breaking_changes_negate_rev_collection, no_ext_breaking_changes_negate_rev_collection;
				opts.extensions.breaking.random_can_be_negative = ext_breaking_changes_rand_can_be_negative, no_ext_breaking_changes_rand_can_be_negative;

				// syntax
				opts.extensions.syntax.list_literals = ext_list_literal, no_ext_list_literal;
				opts.extensions.syntax.string_interpolation = ext_string_interpolation, no_ext_string_interpolation;
				opts.extensions.syntax.control_flow = ext_control_flow, no_ext_control_flow;

				// builtin fns
				opts.extensions.builtin_fns.boolean = ext_builtin_fns_boolean, no_ext_builtin_fns_boolean;
				opts.extensions.builtin_fns.string = ext_builtin_fns_string, no_ext_builtin_fns_string;
				opts.extensions.builtin_fns.list = ext_builtin_fns_list, no_ext_builtin_fns_list;
				opts.extensions.builtin_fns.integer = ext_builtin_fns_integer, no_ext_builtin_fns_integer;
				opts.extensions.builtin_fns.null = ext_builtin_fns_null, no_ext_builtin_fns_null;
				opts.extensions.builtin_fns.assign_to_strings = ext_builtin_fns_assign_to_strings, no_ext_builtin_fns_assign_to_strings;
				opts.extensions.builtin_fns.assign_to_random = ext_builtin_fns_assign_to_random, no_ext_builtin_fns_assign_to_random;
			}
		}

		Ok(opts)
	}
}

pub struct CliOpts {
	options: Options,
	cli: Cli,
}

impl CliOpts {
	pub fn from_argv() -> Self {
		let cli = Cli::parse();

		let options = match cli.options() {
			Ok(opts) => opts,
			Err(err) => err.format(&mut Cli::command()).exit(),
		};

		if cli.expression.is_empty() && cli.file.is_empty() {
			Cli::command()
				.error(error::ErrorKind::MissingRequiredArgument, "either -e or a file must be given")
				.exit();
		}

		debug_assert!(
			cli.expression.is_empty() || cli.file.is_empty(),
			"exaclty one of -e or a file mustve been given?"
		);

		if !cli.argv.is_empty() && {
			#[cfg(feature = "extensions")]
			{
				!options.extensions.argv
			}
			#[cfg(not(feature = "extensions"))]
			{
				true
			}
		} {
			Cli::command()
				.error(
					error::ErrorKind::TooManyValues,
					"additional options may not be supplied unless --ext-argv is enabled",
				)
				.exit();
		}

		Self { options, cli }
	}

	pub fn options(&self) -> &Options {
		&self.options
	}

	pub fn argv(&self) -> impl Iterator<Item = String> {
		self.cli.argv.clone().into_iter()
	}

	pub fn source_iter<'s>(
		&'s self,
	) -> Box<dyn Iterator<Item = std::io::Result<(String, ProgramSource<'s>)>> + 's> {
		if !self.cli.expression.is_empty() {
			Box::new(
				// TODO: remove this clone
				self.cli.expression.iter().map(|source| Ok((source.clone(), ProgramSource::ExprFlag))),
			)
		} else {
			Box::new(self.cli.file.iter().map(|path| {
				let source = std::fs::read_to_string(path)?;
				Ok((source, ProgramSource::File(path)))
			}))
		}
	}
}

use std::path::PathBuf;

use clap::{arg, command, value_parser, Arg, ArgAction, Args, Command, Parser};
use knightrs_bytecode::{strings::Encoding, Options};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
	#[arg(short, long)]
	expression: Option<String>,

	#[arg(short, long)]
	file: Option<String>,
	// .next_help_heading(heading)

	/***************************************************************************
	 *                                Debugger                                 *
	 ***************************************************************************/
	/// Enables all Debugger features
	#[arg(short, long)]
	debugger: bool,

	/// Print out stacktraces
	#[arg(long, hide_short_help = true, overrides_with = "_no_stacktrace")]
	stacktrace: bool,
	/// Undoes stacktrace
	#[arg(long, hide_short_help = true)]
	_no_stacktrace: bool,

	/// Ensure variables are always assigned
	#[arg(long, hide_short_help = true)]
	check_variables: bool,

	/// Ensure parens are always balanced
	#[arg(long, hide_short_help = true)]
	check_parens: bool,

	/***************************************************************************
	 *                                Embedded                                 *
	 ***************************************************************************/
	/// Enables all "embedded" features
	#[arg(long)]
	embedded: bool,

	/// Should exit when quitting the app
	#[arg(long, hide_short_help = true)]
	exit_when_quit: bool,

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
	_no_compliance: bool,

	/// Like --compliance, except non-compliant flags are ignored
	///
	/// Unlike --strict-compliance, this does not disable extensions
	#[arg(short = 'C', long, overrides_with = "_no_strict_compliance")]
	strict_compliance: bool,
	/// Undoes a previous --strict_compliance
	#[arg(long, hide_short_help = true)]
	_no_strict_compliance: bool,

	/// Ensure containers are at most INTMAX elements long.
	#[arg(long, hide_short_help = true)]
	check_container_len: bool,

	/// Constrain integers to 32 bits
	#[arg(short = '3', long, hide_short_help = true)]
	i32_integers: bool,

	/// Check for overflow in arithmetic operations
	#[arg(long, hide_short_help = true)]
	check_overflow: bool,

	/// Check function bounds for integer functions.
	// TODO: maybe have this, along with `check_overflow` be function-specific?
	#[arg(long, hide_short_help = true)]
	check_int_fn_bounds: bool,

	/// Ensure variables are at most 127 chars long
	#[arg(long, hide_short_help = true)]
	validate_variable_name_len: bool,

	/// Ensure at most 65535 variables are used
	#[arg(long, hide_short_help = true)]
	validate_variable_count: bool,

	/// Require programs to be exactly one token long.
	///
	/// Without this option, trailing tokens are ignored
	#[arg(long, hide_short_help = true)]
	forbid_trailing_tokens: bool,

	/// Only support BLOCKs in functions that the spec permits.
	///
	/// Without this, a handful of functions (such as `?` and `DUMP`) support them.
	#[arg(long, hide_short_help = true)]
	strict_blocks: bool,

	/// Don't allow blocks to be converted to other types
	#[arg(long, hide_short_help = true)]
	no_block_conversions: bool,

	/// Limit `RANDOM` to be only from 0 to 32767
	#[arg(long, hide_short_help = true)]
	limit_random_range: bool,

	/// Require `QUIT` to be called with ints from 0 to 127.
	#[arg(long, hide_short_help = true)]
	check_quit_status_code: bool,

	/// Forbid some conversions (such as boolean -> list) that are allowed as an extension.
	#[arg(long, hide_short_help = true)]
	strict_conversions: bool,

	/***************************************************************************
	 *                               Extensions                                *
	 ***************************************************************************/
	/// Enables all extensions
	#[arg(short = 'E', long, overrides_with = "_no_extensions")]
	extensions: bool,
	/// Undoes a previous --extensions
	#[arg(long, hide_short_help = true)]
	_no_extensions: bool,

	/// Enable all extension types. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true)]
	ext_types: bool,

	/// Enable floats. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true)]
	ext_float: bool,

	/// Enable hashmaps. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true)]
	ext_hashmap: bool,

	/// Enable classes. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true)]
	ext_classes: bool,

	/// Enable negative indexing
	#[arg(long, hide_short_help = true)]
	ext_neg_indexing: bool,

	/// Enable the extension functions. UNIMPLEMENTED.
	#[arg(long, hide_short_help = true)]
	ext_fns: bool,

	/// Enable the EVAL function.
	#[arg(long, hide_short_help = true)]
	ext_eval: bool,

	/// Enable the VALUE function.
	#[arg(long, hide_short_help = true)]
	ext_value: bool,

	/// Add support for the `_argv` variable, which is additional arguments on the cli.
	#[arg(long, hide_short_help = true)]
	ext_argv: bool,
	//     --[no-]ext-argv
	//     --[no-]ext-breaking-changes
	//     --[no-]ext-breaking-changes-negate-rev-collection
	//     --[no-]ext-breaking-changes-rand-can-be-negative
	//     --[no-]ext-list-literal          not implemented
	//     --[no-]ext-string-interopolation not implemented
	//     --[no-]ext-control-flow          XBREAK,XCONTINUE, XRETURN; partially working
	//     --[no-]ext-builtin-fns-boolean
	//     --[no-]ext-builtin-fns-string
	//     --[no-]ext-builtin-fns-list
	//     --[no-]ext-builtin-fns-integer
	//     --[no-]ext-builtin-fns-null
	//     --[no-]ext-builtin-fns-assign-to-strings
	//     --[no-]ext-builtin-fns-assign-to-random
}

impl Cli {
	pub fn options(&self) -> Result<Options, clap::Error> {
		let opts = Options::default();

		if self.debugger {
			#[cfg(not(feature = "debugger"))]
			{
				return Err();
			}
			#[cfg(feature = "debugger")]
			{}
		}

		Ok(opts)
	}
}

#[derive(Args)]
// #[clap(flatten)]
struct Embedded2 {
	// Enable all embedded features
	embedded_x: bool,

	// Exit when quitting
	exit_when_quit_x: bool,
}

//     /// Optional name to operate on
//     name: Option<String>,

//     /// Sets a custom config file
//     #[arg(short, long, value_name = "FILE")]
//     config: Option<PathBuf>,

//     /// Turn debugging information on
//     #[arg(short, long, action = clap::ArgAction::Count)]
//     debug: u8,

//     #[command(subcommand)]
//     command: Option<Commands>,
// }

// #[derive(Subcommand)]
// enum Commands {
//     /// does testing things
//     Run {
//         /// lists test values
//         #[arg(short, long)]
//         list: bool,
//     },
// }

pub fn mainx() {
	let cli = Cli::parse();
	dbg!(cli);
	//     let m = command!()
	//         .arg(
	//             arg!(
	//                 -c --config <FILE> "Sets a custom config file"
	//             )
	//             // We don't have syntax yet for optional options, so manually calling `required`
	//             .required(false)
	//             .value_parser(value_parser!(PathBuf)),
	//         )
	//         .arg(arg!(-q --qol "Enables all QOL features"))

	//         .next_help_heading("Embedded")
	//         .arg(arg!(--embedded "Enables all embedded features"))
	//         .args(Embedded2)
	//         .arg(
	//             Arg::new("return-when-exit")
	//                 .long("return-when-exit")
	//                 .help("Exit when quitting")
	//                 .hide_short_help(true))
	//         .next_help_heading("QOL")
	//         .arg(arg!(--stacktrace "Print out stacktraces"))
	//         .arg(arg!(--check_variables "Ensure variables are assigned first"))
	//         .arg(arg!(--check_parens "Ensure parens are balanced"))
	// // # Quality of Life
	// // -q, --[no-]qol             Enables all qol features
	// //      --[no-]stacktrace      Print out stacktraces
	// //      --[no-]check-variables
	// //      --[no-]check-parens
	//         .get_matches();
	//     dbg!(m);

	//     // // You can check the value provided by positional arguments, or option arguments
	//     // if let Some(name) = cli.name.as_deref() {
	//     //     println!("Value for name: {name}");
	//     // }

	//     // if let Some(config_path) = cli.config.as_deref() {
	//     //     println!("Value for config: {}", config_path.display());
	//     // }

	//     // // You can see how many times a particular flag or argument occurred
	//     // // Note, only flags can have multiple occurrences
	//     // match cli.debug {
	//     //     0 => println!("Debug mode is off"),
	//     //     1 => println!("Debug mode is kind of on"),
	//     //     2 => println!("Debug mode is on"),
	//     //     _ => println!("Don't be crazy"),
	//     // }

	//     // // You can check for the existence of subcommands, and if found use their
	//     // // matches just as you would the top level cmd
	//     // match &cli.command {
	//     //     Some(Commands::Run { list }) => {
	//     //         if *list {
	//     //             println!("Printing testing lists...");
	//     //         } else {
	//     //             println!("Not printing testing lists...");
	//     //         }
	//     //     }
	//     //     None => {}
	//     // }

	//     // Continued program logic goes here...
}

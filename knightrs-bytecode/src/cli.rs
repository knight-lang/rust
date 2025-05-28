use std::path::PathBuf;

use clap::{arg, command, value_parser, Arg, ArgAction, Args, Command, Parser};
use knightrs_bytecode::strings::Encoding;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    expression: String,

    #[arg(short, long)]
    file: String,
        // .next_help_heading(heading)


    // QOL


    /// Enables all qol features
    #[arg(short, long)]
    qol: bool,

    /// Print out stacktraces
    #[arg(long)]
    stacktrace: bool,

    /// Ensure variables are always assigned
    #[arg(long)]
    check_variables: bool,

    /// Ensure parens are always balanced
    #[arg(long)]
    check_parens: bool,

    /// Enables all Quality-of-Life features

    // #[arg(flatten)]
    // encoding: Option<Embedded>,

    #[arg(long, help_heading="Q")]
    embedded: String,
    #[arg(long)]
    embedded1: bool,
    #[arg(long, hide_short_help=true)]
    al: bool,


    // pub encoding: Encoding,

    // #[cfg(feature = "compliance")]
    // pub compliance: Compliance,

    // #[cfg(feature = "extensions")]
    // pub extensions: Extensions,

    // #[cfg(feature = "qol")]
    // pub qol: QualityOfLife,

    // #[cfg(feature = "embedded")]
    // pub embedded: Embedded,

    // #[cfg(feature = "check-variables")]
    // pub check_variables: bool,

    // #[cfg(feature = "check-parens")]
    // pub check_parens: bool, // TODO: also make this strict compliance
}

#[derive(Args)]
// #[clap(flatten)]
struct Embedded2 {
    // Enable all embedded features
    embedded_x: bool,

    // Exit when quitting
    exit_when_quit_x: bool
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
    // let cli = Cli::parse();
    let m = command!()
        .arg(
            arg!(
                -c --config <FILE> "Sets a custom config file"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(false)
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(arg!(-q --qol "Enables all QOL features"))

        .next_help_heading("Embedded")
        .arg(arg!(--embedded "Enables all embedded features"))
        .args(Embedded2)
        .arg(
            Arg::new("return-when-exit")
                .long("return-when-exit")
                .help("Exit when quitting")
                .hide_short_help(true))
        .next_help_heading("QOL")
        .arg(arg!(--stacktrace "Print out stacktraces"))
        .arg(arg!(--check_variables "Ensure variables are assigned first"))
        .arg(arg!(--check_parens "Ensure parens are balanced"))
// # Quality of Life
// -q, --[no-]qol             Enables all qol features
//      --[no-]stacktrace      Print out stacktraces
//      --[no-]check-variables
//      --[no-]check-parens
        .get_matches();
    dbg!(m);

    // // You can check the value provided by positional arguments, or option arguments
    // if let Some(name) = cli.name.as_deref() {
    //     println!("Value for name: {name}");
    // }

    // if let Some(config_path) = cli.config.as_deref() {
    //     println!("Value for config: {}", config_path.display());
    // }

    // // You can see how many times a particular flag or argument occurred
    // // Note, only flags can have multiple occurrences
    // match cli.debug {
    //     0 => println!("Debug mode is off"),
    //     1 => println!("Debug mode is kind of on"),
    //     2 => println!("Debug mode is on"),
    //     _ => println!("Don't be crazy"),
    // }

    // // You can check for the existence of subcommands, and if found use their
    // // matches just as you would the top level cmd
    // match &cli.command {
    //     Some(Commands::Run { list }) => {
    //         if *list {
    //             println!("Printing testing lists...");
    //         } else {
    //             println!("Not printing testing lists...");
    //         }
    //     }
    //     None => {}
    // }

    // Continued program logic goes here...
}

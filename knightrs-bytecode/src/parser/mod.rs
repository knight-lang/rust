mod error;
mod parser;
mod source_location;
mod variable_name;

pub use error::*;
pub use parser::*;
pub use source_location::SourceLocation;
pub use variable_name::VariableName;

// pub trait Parseable_OLD {
// 	type Output;

// 	fn parse(parser: &mut Parser<'_, '_>) -> Result<Output, ParseError>;
// }

// // safety: cannot do invalid things with the builder.
// pub unsafe trait Compilable {
// 	fn parse(compiler: &mut Compiler<'_, '_>) -> Result<bool, ParseError>;
// }

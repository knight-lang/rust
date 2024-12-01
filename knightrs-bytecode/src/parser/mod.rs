mod error;
mod parser;
mod source_location;
mod variable_name;

pub use error::*;
pub use parser::*;
pub use source_location::SourceLocation;
pub use variable_name::VariableName;

pub trait Parseable {
	type Output;

	fn parse(parser: &mut Parser<'_, '_>) -> Result<Option<Self::Output>, ParseError>;
}

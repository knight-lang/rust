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

	fn parse<'path>(
		parser: &mut Parser<'_, '_, 'path>,
	) -> Result<Option<Self::Output>, ParseError<'path>>;
}

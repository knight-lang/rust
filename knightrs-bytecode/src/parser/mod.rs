mod error;
mod parser;
mod source_location;
mod variable_name;

pub use error::*;
pub use parser::*;
pub use source_location::SourceLocation;
pub use variable_name::VariableName;

pub trait Parseable<'src, 'path, 'gc> {
	type Output;

	fn parse(
		parser: &mut Parser<'_, 'src, 'path, 'gc>,
	) -> Result<Option<Self::Output>, ParseError<'path>>;
}

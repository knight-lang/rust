mod ast;
mod error;
mod parser;
mod source_location;
mod variable_name;

pub use ast::*;
pub use error::*;
pub use parser::*;
pub use source_location::SourceLocation;
pub use variable_name::VariableName;

pub trait Parseable {
	fn parse(parser: &mut Parser<'_, '_>) -> Result<Option<Ast>, ParseError>;
}

use super::Span;

/// A [`Token`], representing a source program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'src> {
	/// The contents of the token.
	pub span: Span<'src>,

	/// What kind of token it is.
	pub kind: TokenKind,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
	// Specials
	Integer,
	String,
	Variable,

	// Arity 0
	True      = 'T' as _,
	False     = 'F' as _,
	Null      = 'N' as _,
	EmptyList = '@' as _,
	Prompt    = 'P' as _,
	Random    = 'R' as _,

	// Arity 1
	NoOp   = ':' as _,
	Block  = 'B' as _,
	Call   = 'C' as _,
	Quit   = 'Q' as _,
	Dump   = 'D' as _,
	Output = 'O' as _,
	Length = 'L' as _,
	Not    = '!' as _,
	Negate = '~' as _,
	Ascii  = 'A' as _,
	Box    = ',' as _,
	Head   = '[' as _,
	Tail   = ']' as _,

	// Arity 2
	Add       = '+' as _,
	Subtract  = '-' as _,
	Multiply  = '*' as _,
	Divide    = '/' as _,
	Remainder = '%' as _,
	Power     = '^' as _,
	Less      = '<' as _,
	Greater   = '>' as _,
	Equals    = '?' as _,
	And       = '&' as _,
	Or        = '|' as _,
	Then      = ';' as _,
	Assign    = '=' as _,
	While     = 'W' as _,

	// Arity 3
	If  = 'I' as _,
	Get = 'G' as _,

	// Arity 4
	Set = 'S' as _,
}

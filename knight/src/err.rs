#[derive(Debug)]
pub enum Error {
	UndefinedVariable(Box<str>)
}

pub type Result<T> = std::result::Result<T, Error>;
use core::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

// https://users.rust-lang.org/t/what-is-stderror-and-how-exactly-does-propagate-errors/86267
// impl for StdError which is alias of std::error::Error
// needed for the Box<dyn Err>
impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        &self.message
    }
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.message)
    }
}


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


/// error with a stable code, used for `--json` output
#[derive(Debug, PartialEq, Eq)]
pub struct CakeError {
    pub code: &'static str,
    pub message: String,
}

impl std::error::Error for CakeError {}
impl fmt::Display for CakeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.message)
    }
}

pub fn coded(code: &'static str, message: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(CakeError {
        code,
        message: message.into(),
    })
}

pub fn not_found(id: &str) -> Box<dyn std::error::Error> {
    coded("not_found", format!("{} not found.", id))
}

pub fn invalid(message: impl Into<String>) -> Box<dyn std::error::Error> {
    coded("invalid_input", message)
}

pub fn conflict(message: impl Into<String>) -> Box<dyn std::error::Error> {
    coded("conflict", message)
}

pub fn not_published(message: impl Into<String>) -> Box<dyn std::error::Error> {
    coded("not_published", message)
}

pub fn git(message: impl Into<String>) -> Box<dyn std::error::Error> {
    coded("git", message)
}

/// stable code of an error: coded errors keep theirs, parse errors are
/// invalid input, everything else is "other"
pub fn code(e: &(dyn std::error::Error + 'static)) -> &'static str {
    if let Some(c) = e.downcast_ref::<CakeError>() {
        c.code
    } else if e.is::<ParseError>() {
        "invalid_input"
    } else {
        "other"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_of_coded_parse_and_other_errors() {
        assert_eq!(code(not_found("x").as_ref()), "not_found");
        assert_eq!(not_found("x").to_string(), "x not found.");
        let parse: Box<dyn std::error::Error> = Box::new(ParseError { message: "bad".into() });
        assert_eq!(code(parse.as_ref()), "invalid_input");
        let other: Box<dyn std::error::Error> = "plain".into();
        assert_eq!(code(other.as_ref()), "other");
    }
}

use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ParseError {
    ReadLineError,
    ParseError,
    ReadHeaderError,
}

impl ParseError {
    fn as_str(self) -> &'static str {
        match self {
            ParseError::ReadLineError => "ReadLinError",
            ParseError::ParseError => "ParseError",
            ParseError::ReadHeaderError => "ReadHeaderError",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Error for ParseError {
    // fn source(&self) -> Option<&(dyn Error + 'static)> {
    //     Some(&self)
    // }
}

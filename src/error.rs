use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ServerError {
    ReadLineError,
    ParseError,
    ReadHeaderError,
}

impl ServerError {
    fn as_str(self) -> &'static str {
        match self {
            ServerError::ReadLineError => "ReadLinError",
            ServerError::ParseError => "ParseError",
            ServerError::ReadHeaderError => "ReadHeaderError",
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Error for ServerError {
    // fn source(&self) -> Option<&(dyn Error + 'static)> {
    //     Some(&self)
    // }
}

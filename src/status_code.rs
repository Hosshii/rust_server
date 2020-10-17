use self::StatusCode::{Accepted, BadRequest, Continue, Created, Found, NotFound};
use std::fmt;
use std::str::FromStr;
use uncased;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StatusCode {
    Continue,
    Ok,
    Created,
    Accepted,
    Found,
    BadRequest,
    NotFound,
}

impl StatusCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Continue => "Continue",
            StatusCode::Ok => "Ok",
            Created => "Created",
            Accepted => "Accepted",
            Found => "Found",
            BadRequest => "Bad Request",
            NotFound => "Not Found",
        }
    }
    pub fn as_num(&self) -> usize {
        match self {
            Continue => 100,
            StatusCode::Ok => 200,
            Created => 201,
            Accepted => 202,
            Found => 302,
            BadRequest => 400,
            NotFound => 404,
        }
    }
    pub fn from_num(code: usize) -> Result<Self, ()> {
        match code {
            x if x == Continue.as_num() => Ok(Continue),
            x if x == StatusCode::Ok.as_num() => Ok(StatusCode::Ok),
            x if x == Created.as_num() => Ok(Created),
            x if x == Accepted.as_num() => Ok(Accepted),
            x if x == Found.as_num() => Ok(Found),
            x if x == BadRequest.as_num() => Ok(BadRequest),
            x if x == NotFound.as_num() => Ok(NotFound),
            _ => Err(()),
        }
    }
}

impl FromStr for StatusCode {
    type Err = ();
    fn from_str(s: &str) -> Result<StatusCode, ()> {
        match s {
            x if uncased::eq(x, Continue.as_str()) => Ok(Continue),
            x if uncased::eq(x, StatusCode::Ok.as_str()) => Ok(StatusCode::Ok),
            x if uncased::eq(x, Created.as_str()) => Ok(Created),
            x if uncased::eq(x, Accepted.as_str()) => Ok(Accepted),
            x if uncased::eq(x, Found.as_str()) => Ok(Found),
            x if uncased::eq(x, BadRequest.as_str()) => Ok(BadRequest),
            x if uncased::eq(x, NotFound.as_str()) => Ok(NotFound),
            _ => Err(()),
        }
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

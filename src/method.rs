use self::Method::*;
use std::fmt;
use std::str::FromStr;
use uncased;

/// Representation of Http methods.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Method {
    Get,
    Post,
    Delete,
    Patch,
    Put,
    Other,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Get => "GET",
            Post => "POST",
            Delete => "DELETE",
            Patch => "PATCH",
            Put => "PUT",
            Other => "Other",
        }
    }
}

impl FromStr for Method {
    type Err = ();
    fn from_str(s: &str) -> Result<Method, ()> {
        match s {
            x if uncased::eq(x, Get.as_str()) => Ok(Get),
            x if uncased::eq(x, Post.as_str()) => Ok(Post),
            x if uncased::eq(x, Delete.as_str()) => Ok(Delete),
            x if uncased::eq(x, Patch.as_str()) => Ok(Patch),
            x if uncased::eq(x, Put.as_str()) => Ok(Put),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

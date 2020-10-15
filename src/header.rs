use self::ContentType::*;
use self::HttpHeader::*;
use std::str::FromStr;
use uncased;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HttpHeader {
    UserAgent,
    Accept,
    ContentLength,
    ContentType,
}

impl HttpHeader {
    pub fn as_str(self) -> &'static str {
        match self {
            UserAgent => "User-Agent",
            Accept => "Accept",
            ContentLength => "Content-Length",
            ContentType => "Content-Type",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ContentType {
    TextPlain,
    TextHtml,
    ApplicationJson,
    ImageJpeg,
    ImagePng,
}

impl ContentType {
    pub fn as_str(self) -> &'static str {
        match self {
            TextPlain => "text/plain",
            TextHtml => "text/html",
            ApplicationJson => "application/json",
            ImageJpeg => "image/jpeg",
            ImagePng => "image/png",
        }
    }
}

impl FromStr for ContentType {
    type Err = ();
    fn from_str(s: &str) -> Result<ContentType, ()> {
        match s {
            x if uncased::eq(x, TextPlain.as_str()) => Ok(TextPlain),
            x if uncased::eq(x, TextHtml.as_str()) => Ok(TextHtml),
            x if uncased::eq(x, ApplicationJson.as_str()) => Ok(ApplicationJson),
            x if uncased::eq(x, ImageJpeg.as_str()) => Ok(ImageJpeg),
            x if uncased::eq(x, ImagePng.as_str()) => Ok(ImagePng),
            _ => Err(()),
        }
    }
}

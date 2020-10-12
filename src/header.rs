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

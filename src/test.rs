pub trait Handler {
    fn serve_http(self);
}

pub struct DefaultHandler;

impl Handler for DefaultHandler {
    fn serve_http(self) {}
}

impl DefaultHandler {
    fn new() -> Self {
        DefaultHandler {}
    }
}

pub struct Server<'a> {
    handler: Option<&'a dyn Handler>,
}

struct ServeHandler<'a> {
    server: &'a Server<'a>,
}

impl<'a> ServeHandler<'a> {
    pub fn serve_http(&self) {
        let mut handler;
        if let Some(x) = self.server.handler {
            handler = x;
        }
        let handle: &dyn Handler = &DefaultHandler::new();
        handler = handle;
    }
}

use http::{Method, Request};
use regex::{Captures, Regex, RegexSet};
use std::error::Error;

pub trait Matcher<B> {
    fn matches(
        &self,
        req: &Request<B>,
    ) -> bool;
}

struct Any {}

impl<B> Matcher<B> for Any {
    fn matches(
        &self,
        _: &Request<B>,
    ) -> bool {
        true
    }
}

impl<B> Matcher<B> for Method {
    fn matches(
        &self,
        req: &Request<B>,
    ) -> bool {
        req.method() == self
    }
}

pub trait Handler {
    type Body;
    type Output;
    fn call(
        &self,
        input: Request<Self::Body>,
        caps: Option<Captures>,
    ) -> Self::Output;
}

pub struct Router<H>
where
    H: Handler,
{
    routes: RegexSet,
    patterns: Vec<Regex>,
    handlers: Vec<(Box<dyn Matcher<H::Body>>, H)>,
}

impl<H> Router<H>
where
    H: Handler,
{
    pub fn route(
        &self,
        req: Request<H::Body>,
    ) -> Option<H::Output> {
        let uri = req.uri().clone();
        let uri = uri.path();
        for index in self.routes.matches(uri) {
            let (ref matcher, ref handler) = self.handlers[index];
            if !matcher.matches(&req) {
                continue;
            }
            let regex = &self.patterns[index];
            return Some(handler.call(req, regex.captures(uri)));
        }
        None
    }
}

pub struct Builder<H>
where
    H: Handler,
{
    routes: Vec<String>,
    handlers: Vec<(Box<dyn Matcher<H::Body>>, H)>,
}

impl<H> Builder<H>
where
    H: Handler,
{
    pub fn new() -> Self {
        Builder {
            routes: vec![],
            handlers: vec![],
        }
    }

    pub fn any(
        self,
        route: &str,
        handler: H,
    ) -> Self {
        self.route(Any {}, route, handler)
    }

    pub fn get(
        self,
        route: &str,
        handler: H,
    ) -> Self {
        self.route(Method::GET, route, handler)
    }

    pub fn post(
        self,
        route: &str,
        handler: H,
    ) -> Self {
        self.route(Method::POST, route, handler)
    }

    pub fn delete(
        self,
        route: &str,
        handler: H,
    ) -> Self {
        self.route(Method::DELETE, route, handler)
    }
    pub fn patch(
        self,
        route: &str,
        handler: H,
    ) -> Self {
        self.route(Method::PATCH, route, handler)
    }

    pub fn route<M>(
        mut self,
        matcher: M,
        route: &str,
        handler: H,
    ) -> Self
    where
        M: Matcher<H::Body> + 'static,
    {
        // \A (begin) ... \Z (end)
        self.routes.push([r"\A", route, r"\z"].join(""));
        self.handlers.push((Box::new(matcher), handler));
        self
    }

    pub fn build(self) -> Result<Router<H>, Box<dyn Error>> {
        Ok(Router {
            routes: RegexSet::new(self.routes.iter())?,
            patterns: self
                .routes
                .iter()
                .map(|r| Regex::new(r))
                .collect::<Result<_, _>>()?,
            handlers: self.handlers,
        })
    }
}

pub fn routes<H>() -> Builder<H>
where
    H: Handler,
{
    Builder::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Test {
        value: u32,
    }
    impl Handler for Test {
        type Body = ();
        type Output = u32;
        fn call(
            &self,
            _: Request<Self::Body>,
            _: Option<Captures>,
        ) -> Self::Output {
            self.value
        }
    }

    #[test]
    fn it_works() -> Result<(), Box<dyn Error>> {
        let routes = routes()
            .get("/", Test { value: 1 })
            .get("/foo/(?P<id>\\d+)", Test { value: 2 })
            .build()?;
        assert_eq!(
            routes.route(
                http::Request::get("/")
                    .body(())
                    .expect("failed to build request")
            ),
            Some(1)
        );
        assert_eq!(
            routes.route(
                http::Request::get("/foo/str")
                    .body(())
                    .expect("failed to build request")
            ),
            None
        );
        assert_eq!(
            routes.route(
                http::Request::get("/foo/1")
                    .body(())
                    .expect("failed to build request")
            ),
            Some(2)
        );
        assert_eq!(
            routes.route(
                http::Request::get("/nope")
                    .body(())
                    .expect("failed to build request")
            ),
            None
        );
        Ok(())
    }
}

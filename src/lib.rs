//! A parsimonious, framework and IO agnostic HTTP request router.
//!
//! enroute is opinionated about being simple with a focus fast and flexible.
//!
//! ## Simple
//!
//! There are very few components to enroute. There is no IO component and there
//! are no framework specific coupling points.
//!
//! ## Fast
//!
//! enroute leverage the regex crates [`RegexSet`](https://docs.rs/regex/1.3.7/regex/struct.RegexSet.html) feature to filter requests
//! on all patterns in a single pass. This means there is less work for enroute
//! to do.
//!
//! ## Flexible
//!
//! Regexes are universal language for pattern recognition. As such there
//! is incentive to not reinvent a router specific language for http path matching
//! Many routing systems provide a means of extracting information from paths
//! Regex capture groups also do this with built in way of expressing restrictions
//! on the desired type within those capture groups. enroute exposes these captures
//! directly to handlers.
//! Because regular expressions are a universal language it lends it self to more
//! problem domains than can be preemptively designed by an invented
//! syntax.
//!
//! enroute takes a wholistic view of requests with a notion of a `Matcher`
//! which is an extension point for matching additional parts of requests.
//! Perhaps on http methods, or header values or query string parameters.
use http::{header::HeaderName, HeaderValue, Method, Request};
use regex::{Captures, Regex, RegexSet};
use std::error::Error;

/// A secondary means of routing requests
/// via request introspection
///
/// A handful of Matcher impls are provided for http type values.
/// These also serve as an extension point
pub trait Matcher<B> {
    fn matches(
        &self,
        req: &Request<B>,
    ) -> bool;
}

struct Any;

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

impl<B> Matcher<B> for HeaderName {
    fn matches(
        &self,
        req: &Request<B>,
    ) -> bool {
        req.headers().get(self).is_some()
    }
}

impl<B> Matcher<B> for (HeaderName, HeaderValue) {
    fn matches(
        &self,
        req: &Request<B>,
    ) -> bool {
        let (k, v) = self;
        req.headers().get(k).filter(|value| value == v).is_some()
    }
}

/// An abstract request handler
///
/// The only concrete details assumed as that a handler
/// is interested in an http request and optionally information
/// extracted from a request path template
///
/// All details about what a handler responds with is up to its implementation,
/// maximizing the flexibility adapting to different runtimes
pub trait Handler {
    type Body;
    type Output;
    fn call(
        &self,
        input: Request<Self::Body>,
        caps: Option<Captures>,
    ) -> Self::Output;
}

/// A compiled set of routes
pub struct Router<H>
where
    H: Handler,
{
    routes: RegexSet,
    handlers: Vec<(Regex, Box<dyn Matcher<H::Body>>, H)>,
}

impl<H> Router<H>
where
    H: Handler,
{
    /// The primary means of resolving a route
    pub fn route(
        &self,
        req: Request<H::Body>,
    ) -> Option<H::Output> {
        let uri = req.uri().clone();
        let uri = uri.path();
        for index in self.routes.matches(uri) {
            let (ref regex, ref matcher, ref handler) = self.handlers[index];
            if !matcher.matches(&req) {
                continue;
            }
            return Some(handler.call(req, regex.captures(uri)));
        }
        None
    }
}

/// Collects a list of route definitions
pub struct Builder<H>
where
    H: Handler,
{
    #[allow(clippy::type_complexity)]
    routes: Vec<(String, Box<dyn Matcher<H::Body>>, H)>,
}

impl<H> Default for Builder<H>
where
    H: Handler,
{
    fn default() -> Self {
        Builder { routes: vec![] }
    }
}
impl<H> Builder<H>
where
    H: Handler,
{
    /// Routes requests by path
    pub fn any(
        self,
        route: &str,
        handler: H,
    ) -> Self {
        self.route(Any, route, handler)
    }

    /// Routes requests by path and HTTP GET method
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
        self.routes
            .push(([r"\A", route, r"\z"].join(""), Box::new(matcher), handler));
        self
    }

    pub fn build(self) -> Result<Router<H>, Box<dyn Error>> {
        Ok(Router {
            routes: RegexSet::new(self.routes.iter().map(|(pat, _, _)| pat))?,
            handlers: self
                .routes
                .into_iter()
                .map(|(pat, matcher, handler)| Regex::new(&pat).map(|r| (r, matcher, handler)))
                .collect::<Result<_, _>>()?,
        })
    }
}

/// Creates a new `Builder`
pub fn routes<H>() -> Builder<H>
where
    H: Handler,
{
    Builder::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{future::Future, pin::Pin};

    impl Handler for u32 {
        type Body = ();
        type Output = u32;
        fn call(
            &self,
            _: Request<Self::Body>,
            _: Option<Captures>,
        ) -> Self::Output {
            *self
        }
    }

    #[test]
    fn it_works() -> Result<(), Box<dyn Error>> {
        let routes = routes::<u32>()
            .get("/", 1)
            .get("/foo/(?P<id>\\d+)", 2)
            .route(HeaderName::from_static("x-canary-test"), "/test", 3)
            .build()?;
        assert_eq!(routes.route(http::Request::get("/").body(())?), Some(1));
        assert_eq!(routes.route(http::Request::get("/foo/str").body(())?), None);
        assert_eq!(
            routes.route(http::Request::get("/foo/1").body(())?),
            Some(2)
        );
        assert_eq!(routes.route(http::Request::get("/nope").body(())?), None);
        Ok(())
    }

    // insert async framework here
    type Func = fn(Request<()>) -> Pin<Box<dyn Future<Output = http::Response<u32>>>>;

    impl Handler for Func {
        type Body = ();
        type Output = Pin<Box<dyn Future<Output = http::Response<u32>>>>;
        fn call(
            &self,
            req: Request<Self::Body>,
            _: Option<Captures>,
        ) -> Self::Output {
            self(req)
        }
    }

    #[tokio::test]
    async fn it_works_with_futures() -> Result<(), Box<dyn Error>> {
        let routes = routes::<Func>()
            .get("/", |_| {
                Box::pin(async { http::Response::builder().body(1).unwrap() })
            })
            .get("/foo/(?P<id>\\d+)", |_| {
                Box::pin(async { http::Response::builder().body(2).unwrap() })
            })
            .build()?;

        assert_eq!(
            routes
                .route(http::Request::get("/").body(())?)
                .expect("expected match")
                .await
                .body(),
            &1
        );
        assert!(routes
            .route(http::Request::get("/foo/str").body(())?)
            .is_none());
        assert_eq!(
            routes
                .route(http::Request::get("/foo/1").body(())?)
                .expect("expected match")
                .await
                .body(),
            &2
        );
        assert!(routes
            .route(http::Request::get("/nope").body(())?)
            .is_none());
        Ok(())
    }
}

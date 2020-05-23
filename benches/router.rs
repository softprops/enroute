#![feature(test)]
extern crate test;

use enroute::{self, Captures, Handler, Router};
use http::Request;

use test::Bencher;

struct Const(u32);
impl Handler for Const {
    type Body = ();
    type Output = u32;
    fn call(
        &self,
        _: Request<Self::Body>,
        _: Option<Captures>,
    ) -> Self::Output {
        self.0
    }
}

#[bench]
fn bench_routeless_router(b: &mut Bencher) -> Result<(), Box<dyn std::error::Error>> {
    let routes: Router<Const> = enroute::routes().build()?;
    b.iter(|| {
        routes.route(
            Request::builder()
                .uri("/")
                .body(())
                .expect("failed to build request"),
        )
    });
    Ok(())
}

#[bench]
fn bench_static_routes_router(b: &mut Bencher) -> Result<(), Box<dyn std::error::Error>> {
    let routes: Router<Const> = enroute::routes()
        .get("/foo/bar", Const(1))
        .get("/foo/baz", Const(2))
        .get("/foo/boom", Const(3))
        .build()?;
    b.iter(|| {
        routes.route(
            Request::builder()
                .uri("/foo/boom")
                .body(())
                .expect("failed to build request"),
        )
    });
    Ok(())
}

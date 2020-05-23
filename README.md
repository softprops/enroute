<h1 align="center">
  🛩️
  <br/>
  enroute
</h1>

<p align="center">
   wasm-ready parsimonious http request router
</p>

<div align="center">
  <a href="https://github.com/softprops/again/enroute">
		<img src="https://github.com/softprops/enroute/workflows/Main/badge.svg"/>
	</a>
  <a href="https://crates.io/crates/enroute">
		<img src="http://meritbadge.herokuapp.com/enroute"/>
	</a>
  <a href="http://docs.rs/enroute">
		<img src="https://docs.rs/enroute/badge.svg"/>
	</a>  
  <a href="https://softprops.github.io/enroute">
		<img src="https://img.shields.io/badge/docs-master-green.svg"/>
	</a>
</div>

<br />

## Where does this fit in?

Almost anywhere!

There are many web server frameworks in Rust, each packaged with its own request routing api along with many other things typically packaged with servers and frameworks.

Sometimes you need just request routing. Sometimes you you don't need a server. This is `enroute's` target space. 

Enroute is embeddable.

You could actually embed enroute in your framework if you're framework is using the standard `http` types available in the Rust ecosystem.

## Install

Add the following in your `Cargo.toml` file

```toml
[dependencies]
enroute = "0.1"
```

 Doug Tangren (softprops) 2020
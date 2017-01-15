//! Json String Literal Generator
//!
//! Write json with Rust syntax instead of hard to read inline strings.
//! This crate doesn't evaluate any expressions, it just converts a Rust _token tree_ into a
//! minified json string.
//!
//! # Usage
//!
//! This crate is on [crates.io](https://crates.io/crates/json_str).
//!
//! There are two ways to reference `json_str` in your projects, depending on whether you're on
//! the `stable`/`beta` or `nightly` channels.
//!
//! ## Stable
//!
//! To get started, add `json_str` to your `Cargo.toml`:
//!
//! ```ignore
//! [dependencies]
//! json_str = "*"
//! ```
//!
//! And reference it in your crate root:
//!
//! ```ignore
//! #[macro_use]
//! extern crate json_str;
//! ```
//!
//! ## Nightly
//!
//! To get started, add `json_str` to your `Cargo.toml`:
//!
//! ```ignore
//! [dependencies]
//! json_str = { version = "*", features = "nightly" }
//! ```
//!
//! And reference it in your crate root:
//!
//! ```ignore
//! #![feature(plugin)]
//! #![plugin(json_str)]
//! ```
//!
//! If you're on the `nightly` channel, it's better to use the above `plugin` version with the `nightly`
//! feature because the conversion and sanitisation takes place at compile-time instead of runtime,
//! saving precious runtime cycles.
//!
//! ## Examples
//!
//! ### Literals
//! 
//! The `json_str!` macro will take an inline token tree and return a sanitised json `String`:
//!
//! ```ignore
//! let json = json_str!({
//!     "query": {
//!         "filtered": {
//!             "query": {
//!                 "match_all": {}
//!             },
//!             "filter": {
//!                 "geo_distance": {
//!                     "distance": "20km",
//!                     "location": {
//!                         "lat": 37.776,
//!                         "lon": -122.41
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! });
//! ```
//!
//! This will also work for unquoted keys for something a bit more `rusty`:
//!
//! ```ignore
//! let json = json_str!({
//!     query: {
//!         filtered: {
//!             query: {
//!                 match_all: {}
//!             },
//!             filter: {
//!                 geo_distance: {
//!                     distance: "20km",
//!                     location: {
//!                         lat: 37.776,
//!                         lon: -122.41
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! });
//! ```
//!
//! On `nightly`, there's an additional plugin called `json_lit` that returns a `&'static str`
//! instead of a `String`, so you can avoid allocating each time. The syntax is otherwise the same
//! as `json_str`:
//!
//! ```ignore
//! let json = json_lit!({
//!     query: {
//!         filtered: {
//!             query: {
//!                 match_all: {}
//!             },
//!             filter: {
//!                 geo_distance: {
//!                     distance: "20km",
//!                     location: {
//!                         lat: 37.776,
//!                         lon: -122.41
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! });
//! ```
//! 
//! ### Replacement values
//! 
//! The `json_fn` macro will convert a set of replacement tokens and token tree
//! and returns a lambda function that substitutes them:
//! 
//! ```ignore
//! // Declares an inline Fn(&str, &str, &str) -> String
//! let f = json_fn!(|dst, lat, lon| {
//!     query: {
//!         filtered: {
//!             query: {
//!                 match_all: {}
//!             },
//!             filter: {
//!                 geo_distance: {
//!                     distance: $dst,
//!                     location: {
//!                         lat: $lat,
//!                         lon: $lon
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! });
//! 
//! // Call the lambda and return the substituted json
//! let json = f("\"20km\"", "37.776", "-122.41");
//! ```
//! 
//! All input arguments are a `&str`, and the output is a `String`.
//! Only simple variable substitution is supported, no repeating or
//! sanitisation of the replacement values.

#![doc(html_root_url = "http://kodraus.github.io/rustdoc/json_str/")]
#![cfg_attr(feature = "nightly", crate_type="dylib")]
#![cfg_attr(feature = "nightly", feature(plugin_registrar, rustc_private, quote, plugin, stmt_expr_attributes))]

/// Raw parsers for sanitising a stream of json.
pub mod parse;

#[cfg(feature = "nightly")]
include!("lib.rs.in");

#[cfg_attr(not(feature = "nightly"), macro_export)]
#[cfg(not(feature = "nightly"))]
macro_rules! json_str {
    ($j:tt) => ({
        let json_raw = stringify!($j);
        let mut json = String::with_capacity(json_raw.len());

        $crate::parse::parse_literal(json_raw.as_bytes(), &mut json);

        json
    })
}

#[cfg_attr(not(feature = "nightly"), macro_export)]
#[cfg(not(feature = "nightly"))]
macro_rules! json_fn {
    (|$($repl:ident),*| $j:tt) => (|$($repl),*| {
        let repls = {
            let mut repls = ::std::collections::BTreeMap::<&'static str, &str>::new();

            $(repls.insert(stringify!($repl), $repl);)*

            repls
        };

        let json_raw = stringify!($j);

        let mut fragments = Vec::new();
        let mut result = String::new();
        
        $crate::parse::parse_fragments(json_raw.as_bytes(), &mut fragments);

        for f in fragments {
            match f {
                $crate::parse::JsonFragment::Literal(ref l) => result.push_str(l),
                $crate::parse::JsonFragment::Repl(ref r) => {
                    let val = repls
                        .get(r)
                        .expect(&format!("replacement '{}' is not in the list of fn args", r));

                    result.push_str(val);
                }
            }
        }

        result
    })
}
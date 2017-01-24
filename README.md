# json_str

[![Build Status](https://travis-ci.org/KodrAus/json_str.svg?branch=master)](https://travis-ci.org/KodrAus/json_str) 
[![Latest Version](https://img.shields.io/crates/v/json_str.svg)](https://crates.io/crates/json_str)

## [Docs and samples](https://docs.rs/json_str/)

### Use `json_str` for standardised json literals:

```rust
let json = json_str!({
    query: {
        query_string: {
            query: "*"
    }
});
```

### Use `json_fn` for standardised json that supports variable substitutions:

```rust
let get_json = json_fn!(|qry| {
    query: {
        query_string: {
            query: $qry
    }
});

let json = get_json("\"some value\"");
```

Also see the `json` macro from [`serde_json`](https://github.com/serde-rs/json). If you're building complex or dynamic structures, especially on `stable`, it'll be a better approach.

### Details

This crate is an ergonomic way to build json strings in Rust on the `stable` and `nightly` channels. Rust has a json-like syntax for defining structures, so it's easy to convert some valid Rust token trees into json. This crate will also minify whitespace and standardise quotes while it's building the `String`. 

On `stable`, conversion is provided by a simple macro. On `nightly`, conversion is provided by a compiler plugin that sanitises the input at compile time instead of runtime. The `nightly` channel also provides an alternative plugin for creating `&str` literals instead of a `String`, to avoid that allocation.

### Don't trust user input!

`json_fn` is not intended to be used with raw user-input. Values are spliced in as-is, with no sanitisation or escaping done. The parsing (which is also not designed to be secure, it expects trusted input since it come directly from the app binary) runs before any values are substituted. Make sure you verify your inputs appropriately!

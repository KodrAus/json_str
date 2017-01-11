# json_str

[![Build Status](https://travis-ci.org/KodrAus/json_str.svg?branch=master)](https://travis-ci.org/KodrAus/json_str) 
[![Latest Version](https://img.shields.io/crates/v/json_str.svg)](https://crates.io/crates/json_str)

[Docs and samples](http://kodraus.github.io/rustdoc/json_str/)

Provides an easy way to build json strings in Rust without having to use ungainly strings on `stable` and `nightly` channels. Rust has a json-like syntax for defining structures, so it's easy to convert some valid Rust token trees into json. This crate will also minify whitespace and standardise quotes while it's building the `String`. It's an otherwise very simple sanitiser that doesn't do any escaping within values or support interpolation.

On `stable`, conversion is provided by a simple macro. On `nightly`, conversion is provided by a compiler plugin that sanitises the input at compile time instead of runtime. The `nightly` channel also provides an alternative plugin for creating `&str` literals instead of `String`s, to avoid that allocation.

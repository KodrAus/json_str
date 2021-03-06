#![cfg_attr(feature = "nightly", feature(plugin, custom_derive))]
#![cfg_attr(feature = "nightly", plugin(json_str))]

#[cfg_attr(feature = "nightly", allow(plugin_as_library))]
#[macro_use]
extern crate json_str;

use json_str::parse::*;

#[test]
fn can_generate_json() {
    let j = json_str!({
        "a": 7,
        "b": { "c": "some stuff" },
        "data": [
            { "id": 1, "name": "stuff" },
            { "id": 2, "name": "stuff" }
        ]
    });

    assert_eq!("{\"a\":7,\"b\":{\"c\":\"some stuff\"},\"data\":[{\"id\":1,\"name\":\"stuff\"},{\"id\":2,\"name\":\"stuff\"}]}", j);
}

#[test]
fn can_generate_quasi_json() {
    let j = json_str!({
        a: 7,
        b: { c: "some stuff" },
        data: [
            { id: 1, name: "stuff" },
            { id: 2, name: "stuff" }
        ]
    });

    assert_eq!("{\"a\":7,\"b\":{\"c\":\"some stuff\"},\"data\":[{\"id\":1,\"name\":\"stuff\"},{\"id\":2,\"name\":\"stuff\"}]}", j);
}

#[test]
fn can_generate_replacement_json() {
    let f = json_fn!(|qry, fields| {
        "a": {
            "b": {
                "c": $ qry,
                "d": $fields
            },
            "e": $qry
        }
    });

    let j = f("\"*\"", "[1, 2, 3]");

    assert_eq!("{\"a\":{\"b\":{\"c\":\"*\",\"d\":[1, 2, 3]},\"e\":\"*\"}}", j);
}

#[test]
fn sanitisation_removes_whitespace() {
    let j = "\n{ \"a\" : \"stuff\", \"b\":{  \"c\":[ 0, \r\n1 ] }       ,\"d\":14 }";

    let mut sanitised = String::new();
    parse_literal(j.as_bytes(), &mut sanitised);

    assert_eq!("{\"a\":\"stuff\",\"b\":{\"c\":[0,1]},\"d\":14}", &sanitised);
}

#[test]
fn sanitisation_does_not_affect_strings() {
    let j = "\n{ \"a\" : \"stuff and data.\n	More.\", \"b\":\"色は匂へど 散りぬるを\"}";

    let mut sanitised = String::new();
    parse_literal(j.as_bytes(), &mut sanitised);

    assert_eq!("{\"a\":\"stuff and data.\n	More.\",\"b\":\"色は匂へど 散りぬるを\"}", &sanitised);
}

#[test]
fn sanitisation_recognises_escaped_strings() {
    let j = r#"{"a":"a \"quoted'\" string'. \"\\"}"#;

    let mut sanitised = String::new();
    parse_literal(j.as_bytes(), &mut sanitised);

    assert_eq!(r#"{"a":"a \"quoted'\" string'. \"\\"}"#, &sanitised);
}

#[test]
fn sanitisation_standardises_quotes() {
    let j = "{ 'a' : \"stuff\", \"b\":{  \"c\":[ '0', 1 ] },\"d\":14 }";

    let mut sanitised = String::new();
    parse_literal(j.as_bytes(), &mut sanitised);

    assert_eq!("{\"a\":\"stuff\",\"b\":{\"c\":[\"0\",1]},\"d\":14}", &sanitised);
}

#[test]
fn sanitisation_quotes_unquoted_keys() {
    let j = "{ a : \"stuff\", \"b\":{  c:[ 0, 1 ] },d:14 }";

    let mut sanitised = String::new();
    parse_literal(j.as_bytes(), &mut sanitised);

    assert_eq!("{\"a\":\"stuff\",\"b\":{\"c\":[0,1]},\"d\":14}", &sanitised);
}

#[test]
fn sanitisation_does_not_quote_special_values() {
    let j = "{ \"a\": \"stuff\", \"b\": true, \"c\": false, \"d\": null, \"e\": 3.14e+11 }";

    let mut sanitised = String::new();
    parse_literal(j.as_bytes(), &mut sanitised);

    assert_eq!("{\"a\":\"stuff\",\"b\":true,\"c\":false,\"d\":null,\"e\":3.14e+11}", &sanitised);
}

#[test]
fn sanitisation_works_on_empty_string_values() {
    let j = "{ \"a\": \"\", \"b\": 1 }";

    let mut sanitised = String::new();
    parse_literal(j.as_bytes(), &mut sanitised);

    assert_eq!("{\"a\":\"\",\"b\":1}", &sanitised);
}
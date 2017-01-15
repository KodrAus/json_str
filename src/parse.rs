use std::collections::BTreeMap;
use std::str;

// TODO: sanitise_literal parse until '$' -> JsonToken::Literal(value)
// TODO: sanitise_repl parse ident after '$' -> JsonToken::Repl(value)
// TODO: if calling json_str, don't recognise '$'

pub enum JsonFragment<'a> {
    Literal(String),
    Repl(&'a str)
}

pub struct JsonFragments<'a> {
    pub repls: BTreeMap<&'static str, &'a str>,
    pub fragments: Vec<JsonFragment<'a>>
}

impl<'a> ToString for JsonFragments<'a> {
    fn to_string(&self) -> String {
        let mut result = String::new();

        for f in &self.fragments {
            match *f {
                JsonFragment::Literal(ref l) => result.push_str(l),
                JsonFragment::Repl(ref r) => {
                    let val = self.repls.get(r).unwrap();
                    result.push_str(val);
                }
            }
        }

        result
    }
}

pub fn parse<'a>(remainder: &'a [u8], fragments: &mut Vec<JsonFragment<'a>>) {
    let (remainder, l) = literal(remainder, String::new(), true);
    if l.len() > 0 {
        fragments.push(JsonFragment::Literal(l));
    }

    let (remainder, r) = repl(remainder);
    if r.len() > 0 {
        fragments.push(JsonFragment::Repl(r));
    }

    if remainder.len() > 0 {
        parse(remainder, fragments);
    }
}

pub fn repl(remainder: &[u8]) -> (&[u8], &str) {
    if remainder.len() == 0 {
        return (&[], "");
    }

    take_while(&remainder, (), |_, c| {
        let more = (c as char).is_alphabetic() ||
                    c == b'_';

        ((), more)
    })
}

pub fn literal(remainder: &[u8], mut sanitised: String, break_on_repl: bool) -> (&[u8], String) {
    if remainder.len() == 0 {
        return (&[], sanitised);
    }

    let current = remainder[0];

    match current {
        //Replacement
        b'$' if break_on_repl => {
            (&remainder[1..], sanitised)
        },
        //Key
        b'"'|b'\'' => {
            enum StringState {
                Unescaped,
                Escaped
            }

            let (rest, key) = take_while(&remainder[1..], StringState::Unescaped, 
                |s, c| {
                    match (s, c) {
                        //Escape char
                        (StringState::Unescaped, b'\\') => {
                            (StringState::Escaped, true)
                        },
                        //Ignore char after escape
                        (StringState::Escaped, _) => {
                            (StringState::Unescaped, true)
                        },
                        //Unescaped quote
                        (StringState::Unescaped, c) if c == current => {
                            (StringState::Unescaped, false)
                        },
                        //Anything else
                        _ => {
                            (StringState::Unescaped, true)
                        }
                    }
                }
            );

            sanitised.push('"');
            sanitised.push_str(key);
            sanitised.push('"');

            literal(&rest[1..], sanitised, break_on_repl)
        },
        //Start of item
        b'{'|b'['|b':' => {
            sanitised.push(current as char);

            literal(&remainder[1..], sanitised, break_on_repl)
        },
        //Unquoted strings
        b if (b as char).is_alphabetic() => {
            let (rest, key) = take_while(&remainder, (), |_, c| {
                let more = (c as char).is_alphabetic() ||
                            c == b'_' ||
                            c == b'.';

                ((), more)
            });

            //Check if the string is a special value; true, false or null
            //For special values, push them as straight unquoted values. Otherwise, quote them
            match key {
                "true"|"false"|"null" =>
                    sanitised.push_str(key),
                _ => {
                    sanitised.push('"');
                    sanitised.push_str(key);
                    sanitised.push('"');
                }
            }

            literal(rest, sanitised, break_on_repl)
        },
        //Trim whitespace
        b' '|b'\r'|b'\n'|b'\t' => {
            literal(&remainder[1..], sanitised, break_on_repl)
        },
        //Other chars
        _ => {
            sanitised.push(current as char);

            literal(&remainder[1..], sanitised, break_on_repl)
        }
    }
}

pub fn take_while<F, S>(i: &[u8], mut s: S, f: F) -> (&[u8], &str) 
    where F: Fn(S, u8) -> (S, bool) 
{
    let mut ctr = 0;

    for c in i {
        let (new_state, more) = f(s, *c);
        if more {
            s = new_state;
            ctr += 1;
        }
        else {
            break;
        }
    }

    (&i[ctr..], unsafe { str::from_utf8_unchecked(&i[0..ctr]) })
}

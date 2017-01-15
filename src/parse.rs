use std::collections::BTreeMap;
use std::str;

#[derive(Debug)]
pub enum JsonFragment<'a> {
    Literal(String),
    Repl(&'a str)
}

#[derive(Debug)]
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

pub fn parse_literal(remainder: &[u8], mut json: String) -> String {
    let (_, json) = literal(remainder, json, false);

    json
}

pub fn parse_fragments<'a>(remainder: &'a [u8], mut fragments: Vec<JsonFragment<'a>>) -> Vec<JsonFragment<'a>> {
    // Parse a literal
    let (remainder, l) = literal(remainder, String::new(), true);
    if l.len() > 0 {
        fragments.push(JsonFragment::Literal(l));
    }

    // Parse a repl
    let (remainder, r) = repl(remainder);
    if r.len() > 0 {
        fragments.push(JsonFragment::Repl(r));
    }

    // If there's anything left, run again
    if remainder.len() > 0 {
        parse_fragments(remainder, fragments)
    }
    else {
        fragments
    }
}

// Parse a replacement ident.
fn repl(remainder: &[u8]) -> (&[u8], &str) {
    if remainder.len() == 0 {
        return (&[], "");
    }

    take_while(&remainder, (), |_, c| {
        let more = is_ident(c as char);

        ((), more)
    })
}

// Parse a literal and maybe break on a replacement token.
fn literal(remainder: &[u8], mut sanitised: String, break_on_repl: bool) -> (&[u8], String) {
    if remainder.len() == 0 {
        return (&[], sanitised);
    }

    let current = remainder[0];

    match current {
        //Replacement
        b'$' if break_on_repl => {
            //Strip trailing whitespace
            let remainder = shift_while(&remainder[1..], |c| c == b' ');

            (remainder, sanitised)
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
                let more = is_ident(c as char);

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

#[inline]
fn is_ident(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn shift_while<F>(i: &[u8], f: F) -> &[u8]
    where F: Fn(u8) -> bool 
{
    let mut ctr = 0;

    for c in i {
        if f(*c) {
            ctr += 1;
        }
        else {
            break;
        }
    }

    &i[ctr..]
}

fn take_while<F, S>(i: &[u8], mut s: S, f: F) -> (&[u8], &str) 
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

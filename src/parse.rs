use std::str;

/// A fragment of json.
#[derive(Debug)]
pub enum JsonFragment<'a> {
    Literal(String),
    Repl(&'a str)
}

/// Parse and sanitise the complete sequence as a literal.
pub fn parse_literal(remainder: &[u8], json: &mut String) {
    let _ = literal(remainder, json, false);
}

/// Parse and sanitise the complete sequence as literals and replacements.
pub fn parse_fragments<'a>(remainder: &'a [u8], fragments: &mut Vec<JsonFragment<'a>>) {
    // Parse a literal
    let mut l = String::new();
    let remainder = literal(remainder, &mut l, true);
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
        parse_fragments(remainder, fragments);
    }
}

// Parse a replacement ident.
fn repl(remainder: &[u8]) -> (&[u8], &str) {
    if remainder.len() == 0 {
        return (&[], "");
    }

    take_while(&remainder, (), |_, c| {
        ((), is_ident(c as char))
    })
}

// Parse a literal and maybe break on a replacement token.
fn literal<'a>(remainder: &'a [u8], sanitised: &mut String, break_on_repl: bool) -> &'a [u8] {
    if remainder.len() == 0 {
        return &[];
    }

    let current = remainder[0];

    match current {
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
        //Trim whitespace
        b' '|b'\r'|b'\n'|b'\t' => {
            literal(&remainder[1..], sanitised, break_on_repl)
        },
        //Unquoted strings
        b if (b as char).is_alphabetic() => {
            let (rest, key) = take_while(&remainder, (), |_, c| {
                ((), is_ident(c as char))
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
        //Replacement
        b'$' if break_on_repl => {
            //Strip trailing whitespace
            let remainder = shift_while(&remainder[1..], |c| c == b' ');

            remainder
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

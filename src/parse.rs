use std::str;

//TODO: Should take json state. Don't check for special values if parsing key
pub fn sanitise(remainder: &[u8], sanitised: &mut String) {
    if remainder.len() == 0 {
        return;
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

            sanitise(&rest[1..], sanitised)
        },
        //Start of item
        b'{'|b'['|b':' => {
            sanitised.push(current as char);

            sanitise(&remainder[1..], sanitised)
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

            sanitise(rest, sanitised)
        },
        //Trim whitespace
        b' '|b'\r'|b'\n'|b'\t' => {
            sanitise(&remainder[1..], sanitised)
        },
        //Other chars
        _ => {
            sanitised.push(current as char);

            sanitise(&remainder[1..], sanitised)
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

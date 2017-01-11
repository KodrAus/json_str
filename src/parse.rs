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
			let (rest, key) = take_while(&remainder[1..], |c| c != current);

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
			let (rest, key) = take_while1(&remainder, |c|
				(c as char).is_alphabetic() ||
				c == b'_' ||
				c == b'.'
			);

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

pub fn shift_while<F>(i: &[u8], f: F) -> &[u8] where F: Fn(u8) -> bool {
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

pub fn take_while<F>(i: &[u8], f: F) -> (&[u8], &str) where F: Fn(u8) -> bool {
	let mut ctr = 0;

	for c in i {
		if f(*c) {
			ctr += 1;
		}
		else {
			break;
		}
	}

	(&i[ctr..], unsafe { str::from_utf8_unchecked(&i[0..ctr]) })
}

pub fn take_while1<F>(i: &[u8], f: F) -> (&[u8], &str) where F: Fn(u8) -> bool {
	let mut ctr = 0;

	for c in i {
		if f(*c) || ctr == 0 {
			ctr += 1;
		}
		else {
			break;
		}
	}

	(&i[ctr..], unsafe { str::from_utf8_unchecked(&i[0..ctr]) })
}

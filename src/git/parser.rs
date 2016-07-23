use super::*;
use std::str;

#[macro_export]
macro_rules! parsers {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec: Vec<ParserFn> = Vec::new();
            $(
                temp_vec.push($x);
            )*
                temp_vec
        }
    };
}

pub type ParserFn<'a> = fn(&'a str, &mut Status) -> Option<&'a str>;

pub fn parse<'a, F>(sstr: &'a str, parsers: Vec<ParserFn<'a>> ) ->
    Result<Vec<Box<Status>>,&'a str>
{
    let mut s: Vec<Box<Status>> = Vec::new();
    let mut rest: &'a str = sstr;
    while rest.len() > 1 {
        let mut status: Box<Status> =
            Box::new(Status{index: '\0',
                            tree: '\0',
                            from_file: "".to_string(),
                            to_file: "".to_string()});
        for p in &parsers {
            rest = match p(rest, status.as_mut()) {
                Some(r) => r,
                None => return Err("")
            }
        }
        s.push(status);
    }
    Ok(s)
}

pub fn parse_index<'a>(s: &'a str, status: &mut Status) -> Option<&'a str> {
    match parse_utf8_char(s, "MADRU? ") {
        Some((c, cs)) => {status.index = c;
                          Some(cs)} ,
        None => None,
    }
}

pub fn parse_tree<'a>(s: &'a str, status: &mut Status) -> Option<&'a str> {
    match parse_utf8_char(s, "MADU? ") {
        Some((c, cs)) => {status.tree = c;
                          Some(cs)} ,
        None => None,
    }
}

pub fn parse_from<'a>(s: &'a str, status: &mut Status) -> Option<&'a str> {
    match parse_c_string(s) {
        Some((file, rest)) => {status.from_file = file.to_string();
                               Some(rest)},
        None => None,
    }
}

pub fn parse_to<'a>(s: &'a str, status: &mut Status) -> Option<&'a str> {
    if status.index == 'R' {
        let (f, rest) = parse_filename(s);
        match f {
            Some(file) => status.to_file = file.to_string(),
            None => {}
        };
        rest
    } else {
        Some(s)
    }
}

pub fn parse_utf8_char<'a>(s: &'a str, charset: &'static str) -> Option<(char, &'a str)> {
    let mut cs = s.chars();

    match cs.next() {
        Some(c) => if charset.chars().any(|x| x == c) {
            Some((c, cs.as_str()))
            } else {
                None
        },
        _ => None
    }
}

const TERMINATOR: char = '\u{0}';

/// take any character until it meets a zero-byte or the end and
/// returns the found string and the rest of the string
pub fn parse_c_string(stream: &str) -> Option<(&str,&str)> {

    let pos = match stream.chars().position(|c| c == TERMINATOR) {
        Some(pos) => pos,
        None => return None,
    };

    Some((&stream[0..pos], &stream[(pos+1)..]))
}

fn parse_filename<'a>(s: &'a str) -> (Option<&'a str>, Option<&'a str>) {
    let  v: Vec<&str> = s.splitn(2, "\u{0}").collect();
    match v.len() {
        1 => (Some(v[0]), None),
        2 => (Some(v[0]), Some(v[1])),
        _ => (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_utf8_char() {
        let input = " A demo\u{0}";

        match parse_utf8_char(input, "MADRU ") {
            Some((c, rest)) => {assert!(rest == "A demo\u{0}"); assert!(c == ' ');},
            None => assert!(false)
        };

        match parse_utf8_char(input, "MADRU") {
            Some(_) => assert!(false),
            None => assert!(true)
        };
    }

    #[test]
    fn test_match_til_zero() {
        let input = "demo\u{0}second\u{0}";

        let (f, rest) = match parse_c_string(input){
            Some((f, rest)) => (f,rest),
            None => ("",""),
        };
        println!("{:?} {:?}", f, rest);
        assert!(f == "demo");
        assert!(rest == "second\u{0}");
    }

    #[test]
    fn test_parse_file_single() {
        let input = "demo\u{0}";

        let (f, rest) = match parse_c_string(input){
            Some((f, rest)) => (f,rest),
            None => ("",""),
        };
        println!("{:?} {:?}", f, rest);
        assert!(f == "demo");
        assert!(rest == "");
    }

    #[test]
    fn test_parse_file_fail() {
        let input = "demo";

        let (_, _) = match parse_c_string(input){
            Some((f, rest)) => { assert!(false); (f,rest)},
            None => ("", ""),
        };
    }
}

#![feature(macro_rules)]
use std::borrow::Cow;

use super::*;

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

pub type ParserFn<'a> = fn(&'a str, &mut Status<'a>) -> Option<&'a str>;

pub fn parse<'a, F>(sstr: &'a str, parsers: Vec<ParserFn<'a>> ) -> Vec<Status<'a>> {
    let mut s: Vec<Status> = Vec::new();
    let mut rest: &'a str = sstr;
    while rest.len() > 1 {
        let mut status: Status = Status{index: '\0', tree: '\0', from_file: Cow::Owned("".to_string()), to_file: Cow::Owned("".to_string())};
        for p in &parsers {
            rest = match p(rest, &mut status) {
                Some(r) => r,
                None => break
            }
        }
        s.push(status);
    }
    s
}

pub fn parse_index<'a>(s: &'a str, status: &mut Status<'a>) -> Option<&'a str> {
    match parse_utf8_char(s, "MADRU ") {
        Some((c, cs)) => {status.index = c;
                          Some(cs)} ,
        None => None,
    }
}

pub fn parse_tree<'a>(s: &'a str, status: &mut Status<'a>) -> Option<&'a str> {
    match parse_utf8_char(s, "MADU ") {
        Some((c, cs)) => {status.tree = c;
                          Some(cs)} ,
        None => None,
    }
}

pub fn parse_from<'a>(s: &'a str, status: &mut Status<'a>) -> Option<&'a str> {
    let (f, rest) = parse_filename(s);
    match f {
        Some(file) => status.from_file = Cow::Borrowed(file),
        None => {}
    };
    rest
}

pub fn parse_to<'a>(s: &'a str, status: &mut Status<'a>) -> Option<&'a str> {
    if status.index == 'R' {
        let (f, rest) = parse_filename(s);
        match f {
            Some(file) => status.to_file = Cow::Borrowed(file),
            None => {}
        };
        rest
    } else {
        Some(s)
    }
}

fn parse_utf8_char<'a>(s: &'a str, charset: &'static str) -> Option<(char, &'a str)> {
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

pub fn parse_filename<'a>(s: &'a str) -> (Option<&'a str>, Option<&'a str>) {
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
    fn test_parse_file() {
        let input = "demo\u{0}second\u{0}";

        let (f, rest) = parse_filename(input);
        println!("{} {}", f, rest);
        assert!(f.unwrap() == "demo");
        assert!(rest.unwrap() == "second\u{0}");
    }

    #[test]
    fn test_parse_file_single() {
        let input = "demo\u{0}";

        let (f, rest) = parse_filename(input);
        println!("{} {}", f, rest);
        assert!(f.unwrap() == "demo");
        assert!(rest.unwrap() == "");
    }

    #[test]
    fn test_parse_file_fail() {
        let input = "demo";

        let (f, rest) = parse_filename(input);
        println!("{} {}", f, rest);
        assert!(f.unwrap() == "demo");
        assert!(rest.unwrap() == "");
    }
}

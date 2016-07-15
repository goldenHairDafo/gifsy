use std::borrow::Cow;

pub mod parser;

#[derive(Debug)]
pub struct Status<'a> {
    index: char,
    tree: char,
    from_file: Cow<'a, str>,
    to_file: Cow<'a, str>,
}

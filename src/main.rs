extern crate gifsy;

use gifsy::git;
use std::env;

fn main() {
    let status = match env::var_os("GIFSY_REPO") {
        Some(repo) => git::status(repo.to_string_lossy().as_ref()),
        None => panic!("GIFSY_REPO not set!")
    };

    match status {
        Ok(s) => {
            println!("{}", git::create_commit_message(s).unwrap() )
        },
        Err(e) => panic!("no status! {}", e)
    }
}

extern crate gifsy;

use std::process::Command;
use std::fs;
use std::path;
use std::borrow::Cow;

use gifsy::git::parser::*;
use gifsy::git::*;

fn main() {
    match option_env!("GIFSY_REPO") {
        Some(repo) => status(&repo),
        None => panic!("GIFSY_REPO not set!")
    }
}

fn status<'a >(repo: &'a str) {
    if check_repos(repo) {
        let output = Command::new("git")
            .current_dir(repo)
            .arg("status")
            .arg("--porcelain")
            .arg("-z")
            .output()
            .expect("can't execute git status");

        let outstr = String::from_utf8_lossy(&output.stdout);
        let rest: &str = &outstr;
        let p: Vec<ParserFn> = vec![parse_index, parse_tree, parse_from, parse_to];
        let s = parse::<Vec<Status>>(&rest, p);
        println!("status: {:?}", s);
        println!("stderr {:?}", String::from_utf8_lossy(&output.stderr));
    } else {
        println!("{} is not a git repository", repo);
    };
}

/// Check if the repository path is actual a git repository with
/// a working tree
fn check_repos(repo: &str) -> bool {
    match fs::metadata(repo) {
        Ok(meta) => if meta.is_dir() == true {
            let mut gitdir = path::PathBuf::from(repo);
            gitdir.push(".git");
            match fs::metadata(&gitdir) {
                Ok(m) => m.is_dir(),
                Err(_) => return false
            }
        } else {
            false
        },
        Err(_) => return false
    }
}

#[macro_use]
extern crate clap;
extern crate gifsy;

use gifsy::git;
use clap::App;
use std::env;
use std::path;

fn main() {

    let clap_yamp = load_yaml!("clap.yaml");
    let matches = App::from_yaml(clap_yamp).get_matches();

    let repo = &match  matches.value_of("repo") {
        Some(repo) => repo.to_string(),
        None => {
            let home = env::var("HOME").ok().expect("HOME not found");
            let mut defaultpath = Box::new( path::PathBuf::from(home));
            defaultpath.push("Shared");
            defaultpath.push("sync");
            defaultpath.to_string_lossy().into_owned()
        }
    };

    match matches.subcommand_name() {
        Some(subcmd) => match subcmd {
            "status" => status(repo),
            "sync" => sync(repo),
            _ => panic!("no subcommand found")
        },
        None => return
    };
}

fn status(repo: &str) -> i32 {
    let status = git::status(repo);

    match status {
        Ok(s) => {
            println!("{}", git::create_commit_message(s).unwrap() );
            0
        },
        Err(e) => panic!("no status! {}", e)
    }
}

fn sync(repo: &str) -> i32 {
    match git::pull(repo) {
        Ok(i) => i,
        Err(e) => { println!("pull error {}", e); return 1}
    };
    let mut status = match git::status(repo) {
        Ok(status) => status,
        Err(e) => { println!("status error {}", e); return 2}
    };
    if status.len() > 0 {
        status = match git::add(repo, status) {
            Ok(status) => status,
            Err(e) => { println!("add error {}", e); return 3}
        };
        match git::commit(repo, status) {
            Ok(i) => i,
            Err(e) => { println!("commit {}", e); return 4}
        };
        match git::push(repo) {
            Ok(i) => i,
            Err(e) => { println!("push {}", e); return 5}
        }
    } else {
        0
    }
}

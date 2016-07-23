#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate gifsy;

use std::env;
use std::path;

use gifsy::git;
use clap::{App,Arg,AppSettings,ArgMatches,SubCommand};

const RC_OK: i32 = 0;
const RC_ERR_SUBCMD_UNKNOWN: i32 = 1001;
const RC_ERR_SUBCMD_NOT_FOUND: i32 = 1002;
const RC_ERR_SYNC_PULL: i32 = 1003;
const RC_ERR_SYNC_STATUS: i32 = 1004;
const RC_ERR_SYNC_ADD: i32 = 1005;
const RC_ERR_SYNC_COMMIT: i32 = 1006;
const RC_ERR_SYNC_PUSH: i32 = 1007;

fn main() {

    env_logger::init().unwrap();

    debug!("GIt FileSYncronization startet");

    // Work the command line arguments
    let matches = arguments();

    let repo = &match  matches.value_of("repo") {
        Some(repo) => repo.to_string(),
        None => {
            info!("use default repository path");
            let home = env::var("HOME")
                .ok()
                .expect("HOME environemnt variable not found");
            let mut defaultpath = Box::new( path::PathBuf::from(home));
            defaultpath.push("Shared");
            defaultpath.push("sync");
            defaultpath.to_string_lossy().into_owned()
        }
    };
    debug!("use repository {}", repo);

    let r = match git::Repository::from(repo) {
        Ok(r) => r,
        Err(e) => {error!("can't create repository{}", e);
                   std::process::exit(1)}
    };

    let ecode = match matches.subcommand_name() {
        Some(subcmd) => match subcmd {
            "status" => status(&r),
            "sync" => sync(&r),
            n => {error!("unknown subcommand {} found", n);
                  std::process::exit(RC_ERR_SUBCMD_UNKNOWN) }
        },
        None =>{error!("no subcommand found");
                println!("{}", matches.usage());
                std::process::exit(RC_ERR_SUBCMD_NOT_FOUND)}
    };
    debug!("GIt FileSYncronization done");
    std::process::exit(ecode);
}

fn status(repo: &git::Repository) -> i32 {

    debug!("check status");

    let status = repo.status();

    match status {
        Ok(s) => {
            println!("{}", git::create_commit_message(s).unwrap() );
            RC_OK
        },
        Err(e) => panic!("no status! {}", e)
    }
}

fn sync(repo: &git::Repository) -> i32 {

    debug!("syncronize repository");

    debug!("pull changes");
    match repo.pull() {
        Ok(_) => 0,
        Err(e) => { error!("pull error {}", e);
                    return RC_ERR_SYNC_PULL}
    };
    let mut status = match repo.status() {
        Ok(status) => status,
        Err(e) => { error!("status error {}", e);
                    return RC_ERR_SYNC_STATUS}
    };
    if status.len() > 0 {
        debug!("add and push local changes");
        status = match repo.add(status) {
            Ok(status) => status,
            Err(e) => { error!("add error {}", e);
                        return RC_ERR_SYNC_ADD}
        };
        match repo.commit(status) {
            Ok(_) => 0,
            Err(e) => { error!("commit {}", e);
                        return RC_ERR_SYNC_COMMIT}
        };
        match repo.push()  {
            Ok(_) => 0,
            Err(e) => { error!("push {}", e);
                        return RC_ERR_SYNC_PUSH}
        }
    } else {
        debug!("no local changes");
        RC_OK
    }
}
fn arguments<'a>() -> ArgMatches<'a> {
    App::new("gifsy")
        .author("Dafo with the golden Hair <dafo@e6z9r.net>")
        .version("0.9")
        .about("GIT based file syncronasitaion for dot files")
        .setting(AppSettings::SubcommandRequired)
        .arg(Arg::with_name("repo")
              .short("-r")
              .long("repo")
              .value_name("PATH")
              .takes_value(true)
              .help("Sets the path to the repository"))
        .subcommand(SubCommand::with_name("sync")
                    .about("Synchronize the repository"))
        .subcommand(SubCommand::with_name("status")
                    .about("Status of the repository"))
        .get_matches()
}

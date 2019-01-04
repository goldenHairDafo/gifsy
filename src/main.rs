extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate gifsy;

use std::env;
use std::path;
use std::error::Error;

use gifsy::git;
use gifsy::git::GifsyError;
use clap::{App,Arg,AppSettings,ArgMatches,SubCommand};

#[derive(Debug, Clone)]
enum MainError {
    SubcomamndUnknown,
    SubcommandNotFound,
    NoRepository,
    GitFailed(i32, String)
}

impl MainError {
    fn code(&self) -> i32 {
        match *self {
            MainError::SubcomamndUnknown => 1001,
            MainError::SubcommandNotFound => 1002,
            MainError::NoRepository => 1008,
            MainError::GitFailed(c, _) => c,
        }
    }
}

impl From<GifsyError> for MainError{
    fn from(e: GifsyError) -> Self {
        match e {
            git::GifsyError::CmdFail(c, m) => MainError::GitFailed(c, m),
            e => MainError::GitFailed(1008, e.description().to_owned())
        }
    }
}


fn main() {

    env_logger::init().unwrap();

    debug!("GIt FileSYncronization startet");

    let host_env = match env::var("HOST") {
        Ok(h) => h,
        Err(_) => String::from("Unknown Host")
    };
    let name_env = match env::var("GIFSY_NAME") {
        Ok(h) => h,
        Err(_) => host_env
    };
    let repo_path_env = match env::var("GIFSY_REPO") {
        Ok(h) => h,
        Err(_) => {
            debug!("use default repository path");
            let home = env::var("HOME")
                .ok()
                .expect("HOME environemnt variable not found");
            let mut defaultpath = Box::new( path::PathBuf::from(home));
            defaultpath.push("Shared");
            defaultpath.push("sync");
            defaultpath.to_string_lossy().into_owned()
        }
    };


    // Work the command line arguments
    let matches = arguments();

    let repo = &match  matches.value_of("repo") {
        Some(repo) => repo.to_string(),
        None => repo_path_env
    };
    let name = &match  matches.value_of("name") {
        Some(repo) => repo.to_string(),
        None => name_env
    };
    debug!("use repository {}", repo);

    let r = match git::Repository::from(repo, &name) {
        Ok(r) => r,
        Err(e) => {error!("can't create repository{}", e);
                   std::process::exit(MainError::NoRepository.code())}
    };

    let ecode = match matches.subcommand_name() {
        Some(subcmd) => match subcmd {
            "status" => status(&r),
            "sync" => sync(&r),
            n => {error!("unknown subcommand {} found", n);
                  std::process::exit(MainError::SubcomamndUnknown.code()) }
        },
        None =>{error!("no subcommand found");
                println!("{}", matches.usage());
                std::process::exit(MainError::SubcommandNotFound.code())}
    };
    debug!("ecode: {:?}", ecode);
    let rc = match ecode {
        Ok(()) => { debug!("GIt FileSYncronization done"); 0},
        Err(rc) => { println!("GIt FileSYncronization done with error {:?}", rc); rc.code()},
    };
    std::process::exit(rc);
}

fn status(repo: &git::Repository) -> Result<(), MainError> {

    debug!("check status");

    let status = try!(repo.status());

    println!("{}", git::create_commit_message(status, &repo.name()).unwrap() );
    Ok(())
}

fn sync(repo: &git::Repository) -> Result<(), MainError> {

    debug!("synchronize repository");

    let mut status = try!(repo.status());
    if status.len() > 0 {
        debug!("add local changes");
        try!(repo.add(status));
        debug!("update local status");
        status = try!(repo.status());
        debug!("comit local changes");
        try!(repo.commit(status));
    } else {
        debug!("no local changes");
    }
    debug!("pull changes");
    try!(repo.pull());
    debug!("handle submodules");
    try!(repo.submodules_init());
    try!(repo.submodules_update());
    debug!("push changes");
    try!(repo.push());
    Ok(())
}

fn arguments<'a>() -> ArgMatches<'a> {
    App::new("gifsy")
        .author("Dafo with the golden Hair <dafo@e6z9r.net>")
        .version("0.9.1")
        .about("GIT based file synchronization for dot files")
        .setting(AppSettings::SubcommandRequired)
        .arg(Arg::with_name("repo")
             .short("-r")
             .long("repo")
             .value_name("PATH")
             .takes_value(true)
             .help("Sets the path to the repository"))
        .arg(Arg::with_name("name")
              .short("-n")
              .long("name")
              .value_name("NAME")
              .takes_value(true)
              .help("Sets the name to identify the host"))
        .subcommand(SubCommand::with_name("sync")
                    .about("Synchronize the repository"))
        .subcommand(SubCommand::with_name("status")
                    .about("Status of the repository"))
        .get_matches()
}

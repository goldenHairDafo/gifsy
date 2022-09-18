extern crate clap;
extern crate flexi_logger;
extern crate notify_rust;
#[macro_use]
extern crate log;

extern crate gifsy;

use std::env;
use std::path;

use clap::{App, AppSettings, Command, Arg, SubCommand};
use flexi_logger::FileSpec;
use flexi_logger::{Duplicate, Logger, opt_format};
use gifsy::git;
use gifsy::git::GifsyError;
use gifsy::notify;

#[derive(Debug, Clone)]
enum MainError {
    SubcomamndUnknown,
    SubcommandNotFound,
    NoRepository,
    GitFailed(i32, String),
}

impl MainError {
    fn code(&self) -> i32 {
        match *self
        {
            MainError::SubcomamndUnknown => 1001,
            MainError::SubcommandNotFound => 1002,
            MainError::NoRepository => 1008,
            MainError::GitFailed(c, _) => c,
        }
    }
}

impl From<GifsyError> for MainError {
    fn from(e: GifsyError) -> Self {
        match e
        {
            git::GifsyError::CmdFail(c, m) => MainError::GitFailed(c, m),
            e => MainError::GitFailed(1008, e.to_string()),
        }
    }
}
fn main() {
    let home = env::var("HOME")
        .ok()
        .expect("HOME environemnt variable not found");
    let host_env = match env::var("HOST")
    {
        Ok(h) => h,
        Err(_) => String::from("Unknown Host"),
    };
    let name_env = match env::var("GIFSY_NAME")
    {
        Ok(h) => h,
        Err(_) => host_env,
    };
    let repo_path_env = match env::var("GIFSY_REPO")
    {
        Ok(h) => h,
        Err(_) =>
        {
            debug!("use default repository path");
            let mut defaultpath = Box::new(path::PathBuf::from(home.clone()));
            defaultpath.push("Shared");
            defaultpath.push("sync");
            defaultpath.to_string_lossy().into_owned()
        }
    };
    // Work the command line arguments
    let mut app = arguments();
    let matches = app.clone().get_matches();

    let repo = &match matches.value_of("repo")
    {
        Some(repo) => repo.to_string(),
        None => repo_path_env,
    };
    let name = &match matches.value_of("name")
    {
        Some(repo) => repo.to_string(),
        None => name_env,
    };
    let logdir: &str = &match matches.value_of("logdir")
    {
        Some(repo) => repo.to_string(),
        None => format!("{}/.local/log", home),
    };
    if matches.is_present("notify")
    {
        notify::enable();
    }
    /* Setting up logging */
    let fspec = FileSpec::default()
      .directory(logdir)
      .suppress_timestamp();
    Logger::try_with_env_or_str("warn").expect("")
        .log_to_file(fspec)
        .append()
        .duplicate_to_stderr(Duplicate::Error)
        .format(opt_format)
        .start()
        .unwrap();

    debug!("GIt FileSYncronization startet");

    debug!("use repository {}", repo);
    let r = match git::Repository::from(repo, &name)
    {
        Ok(r) => r,
        Err(e) =>
        {
            notify::send(
                "GIt FileSYncronization needs attension",
                "gifsy sync needs some love",
            );
            error!("can't create repository{}", e);
            std::process::exit(MainError::NoRepository.code())
        }
    };

    let ecode = match matches.subcommand_name()
    {
        Some(subcmd) => match subcmd
        {
            "status" => status(&r),
            "sync" => sync(&r),
            n =>
            {
                error!("unknown subcommand {} found", n);
                std::process::exit(MainError::SubcomamndUnknown.code())
            }
        },
        None =>
        {
            error!("no subcommand found\n{}", app.render_usage());
            std::process::exit(MainError::SubcommandNotFound.code())
        }
    };
    debug!("command return code: {:?}", ecode);
    let rc = match ecode
    {
        Ok(()) =>
        {
            debug!("GIt FileSYncronization done");
            0
        }
        Err(rc) =>
        {
            notify::send(
                "GIt FileSYncronization needs attension",
                "gifsy sync needs some love",
            );
            error!("GIt FileSYncronization done with error {:?}", rc);
            rc.code()
        }
    };
    std::process::exit(rc);
}

fn status(repo: &git::Repository) -> Result<(), MainError> {
    debug!("check status");

    let status = repo.status()?;

    println!(
        "{}",
        git::create_commit_message(&status, &repo.name()).unwrap()
    );
    Ok(())
}

fn sync(repo: &git::Repository) -> Result<(), MainError> {
    debug!("synchronize repository");

    let mut status = repo.status()?;
    if status.len() > 0
    {
        debug!("add local changes");
        repo.add(status)?;
        debug!("update local status");
        status = repo.status()?;
        debug!("comit local changes");
        repo.commit(status)?;
    }
    else
    {
        debug!("no local changes");
    }
    debug!("pull changes");
    repo.pull()?;
    debug!("handle submodules");
    repo.submodules_init()?;
    repo.submodules_update()?;
    debug!("push changes");
    repo.push()?;
    Ok(())
}

fn arguments<'a>() -> App<'a> {
    Command::new("gifsy")
        .author("Dafo with the golden Hair <dafo@e6z9r.net>")
        .version("0.9.7")
        .about("GIT based file synchronization for dot files")
        .setting(AppSettings::SubcommandRequired)
        .arg(
            Arg::with_name("repo")
                .short('r')
                .long("repo")
                .value_name("PATH")
                .takes_value(true)
                .help("Sets the path to the repository"),
        )
        .arg(
            Arg::with_name("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .takes_value(true)
                .help("Sets the name to identify the host"),
        )
        .arg(
            Arg::with_name("logdir")
                .short('l')
                .long("logdir")
                .value_name("LOGDIR")
                .takes_value(true)
                .help("Sets the directory to write log file to"),
        )
        .arg(
            Arg::with_name("notify")
                .long("notify")
                .takes_value(false)
                .help("enables desktop notification"),
        )
        .subcommand(SubCommand::with_name("sync").about("Synchronize the repository"))
        .subcommand(SubCommand::with_name("status").about("Status of the repository"))
}

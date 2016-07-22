use std::process::{Command,Stdio};
use std::fmt;
use std::fs;
use std::path;
use std::str;
use std::io::{Write};
use std::string::*;

use chrono::Local;

use self::parser::*;

#[macro_use]
pub mod parser;

#[derive(Clone)]
pub struct Status {
    index: char,
    tree: char,
    from_file: String,
    to_file: String
}

impl Status {
    pub fn is_unmerged(&self) -> bool {
        self.index == 'U' || self.tree == 'U'
    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.to_file == "" {
            write!(f, "{}{} {}", self.index, self.tree, self.from_file)
        } else {
            write!(f, "{}{} {} -> {}", self.index, self.tree, self.from_file, self.to_file)
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.to_file == "" {
            write!(f, "  {} {}", encode_status_flag(self.index), self.from_file)
        } else {
            write!(f, "  {} {} -> {}", encode_status_flag(self.index), self.from_file, self.to_file)
        }
    }
}


/// Check if the repository path is actual a git repository with
/// a working tree
pub fn is_repos(repo: &str) -> bool {
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

pub type CmdFn<'a, F> = fn(&'a str) -> Result<F, String>;

pub fn with_repo<'a, F, C>(repo: &'a str, cmd: C) -> Result<F, String>
    where C: Fn(&'a str) -> (F, String)
{
    if is_repos(repo) {
        let (rc, err) = cmd(repo);
        if "" == err {
            Ok(rc)
        } else {
            Err(err)
        }
    } else {
        Err(format!("{} is not a repository", repo))
    }
}

pub fn add<'a>(repo: &str,status: Vec<Box<Status>>) -> Result<Vec<Box<Status>>,String> {
    with_repo(repo, move |r| {
        let mut rc = Vec::new();
        for s in &status {
            if s.is_unmerged() {
                warn!("unmerged file {}", s);
                continue;
            }
            let to_file = s.to_file.clone();
            let output = Command::new("git")
                .current_dir(r)
                .arg("add")
                .arg(&to_file)
                .output()
                .expect("can't execute git status");
            if !output.status.success() {
                return (rc, format!("can't add {} ({})", &to_file, String::from_utf8_lossy(&output.stdout)))
            }
            rc.push(s.clone());
        }
        (rc, "".to_string())
    })
}

pub fn commit<'a>(repo: &str,status: Vec<Box<Status>>) -> Result<i32,String> {
    if is_repos(repo) {
        let process = match Command::new("git")
            .current_dir(repo)
            .arg("commit")
            .arg("-f")
            .arg("-")
            .stdin(Stdio::piped())
            .spawn() {
                Err(e) => return Err(format!("can't start git commit {}", e)),
                Ok(process) => process,
            };
        let msg = create_commit_message(status).unwrap();
        match process.stdin.unwrap().write_all(msg.as_bytes()){
            Err(e) => return Err(format!("can't wirte commit message {}", e)),
            Ok(_) => {},
        };
        Ok(0)
    } else {
        Err(format!("{} is not a git repository", repo))
    }
}

pub fn status<'a >(repo: &'a str) -> Result<Vec<Box<Status>>, String> {
    if is_repos(repo) {
        match Command::new("git")
            .current_dir(repo)
            .arg("status")
            .arg("--porcelain")
            .arg("-z").output() {
                Err(e) => Err(format!("can't execute git sstatus ({})", e)),
                Ok(output) => if output.status.success() {
                        let rest = String::from_utf8_lossy(&output.stdout);
                        let p = parsers![parse_index, parse_tree, parse_from, parse_to];
                        match parse::<Vec<&Status>>(&rest, p) {
                            Err(e) => return Err(format!("can't parse git sstatus ({})", e)),
                            Ok(status) => Ok(status),
                        }
                    } else {
                        Err(format!("can't execute git sstatus ({})", output.status.code().unwrap()))
                    }
            }
    } else {
        Err(format!("{} is not a git repository", repo))
    }
}

pub fn pull<'a >(repo: &'a str) -> Result<i32,String> {
    if is_repos(repo) {
        let output = Command::new("git")
            .current_dir(repo)
            .arg("pull")
            .arg("origin")
            .output()
            .expect("can't execute git status");

        if output.status.success() {
            Ok(0)
        } else {
            Err(format!("{}", String::from_utf8_lossy(&output.stderr)).to_string())
        }
    } else {
        Err(format!("{} not a repository",repo).to_string())
    }
}

pub fn push<'a >(repo: &'a str) -> Result<i32,String> {
    if is_repos(repo) {
        let output = Command::new("git")
            .current_dir(repo)
            .arg("push")
            .arg("origin")
            .output()
            .expect("can't execute git status");

        if output.status.success() {
            Ok(0)
        } else {
            Err(format!("{}", String::from_utf8_lossy(&output.stderr)).to_string())
        }
    } else {
        Err(format!("{} not a repository",repo).to_string())
    }
}

pub fn create_commit_message(status: Vec<Box<Status>>) -> Result<String,FromUtf8Error> {
    let mut commitmsg = Vec::new();
    writeln!(&mut commitmsg, "changes from {}\n", Local::now().to_rfc2822()).unwrap();
    for s in status {
        writeln!(&mut commitmsg, "{}", s).unwrap();
    }
    String::from_utf8(commitmsg)
}

fn encode_status_flag(flag: char) -> char {
    match flag {
        'M' => '~',
        'A' => '+',
        'D' => '-',
        'R' => '>',
        'U' => '!',
        '?' => '?',
        ' ' => ' ',
        _ => '•'
    }
}

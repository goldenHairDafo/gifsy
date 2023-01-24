use std::error;
use std::fmt;
use std::io::{Error, Write};
use std::path;
use std::process::{Command, Stdio};
use std::str;
use std::string::*;

use chrono::prelude::*;

use self::parser::*;
use super::notify;

#[macro_use]
pub mod parser;

#[derive(Debug)]
pub enum GifsyError {
    NoRepoitory,
    IoError(Error),
    ParserError(String),
    CmdFail(i32, String),
}
impl fmt::Display for GifsyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self
        {
            GifsyError::CmdFail(code, ref out) =>
            {
                write!(f, "git command failed {} ({})", out, code)
            }
            GifsyError::NoRepoitory => write!(f, "the path is not a git repository"),
            GifsyError::IoError(ref e) => write!(f, "io error {}", e),
            GifsyError::ParserError(..) => write!(f, "parser error"),
        }
    }
}
impl error::Error for GifsyError {
}

pub struct Repository {
    path: String,
    name: String,
}

impl Repository {
    pub fn from(path: &str, name: &str) -> Result<Repository, GifsyError> {
        let repository_path = path::PathBuf::from(path);
        if repository_path.as_path().is_dir()
        {
            Ok(Repository {
                path: path.to_owned(),
                name: name.to_owned(),
            })
        }
        else
        {
            Err(GifsyError::NoRepoitory)
        }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn status(&self) -> Result<Vec<Box<Status>>, GifsyError> {
        match Command::new("git")
            .current_dir(&self.path)
            .arg("status")
            .arg("--porcelain")
            .arg("-z")
            .output()
        {
            Err(e) => Err(GifsyError::IoError(e)),
            Ok(output) =>
            {
                if output.status.success()
                {
                    let rest = String::from_utf8_lossy(&output.stdout);
                    let p = parsers![parse_index, parse_tree, parse_from, parse_to];
                    match parse::<Vec<&Status>>(&rest, p)
                    {
                        Err(e) => Err(GifsyError::ParserError(e.to_string())),
                        Ok(status) => Ok(status),
                    }
                }
                else
                {
                    Err(GifsyError::CmdFail(
                        output.status.code().unwrap(),
                        String::from_utf8_lossy(&output.stderr).to_string(),
                    ))
                }
            }
        }
    }
    pub fn add(&self, status: Vec<Box<Status>>) -> Result<Vec<Box<Status>>, GifsyError> {
        let mut rc = Vec::new();
        for s in &status
        {
            if s.is_unmerged()
            {
                warn!("unmerged file {}", s);
                let msg = format!("File {} need to be manually merged", s.file());
                notify::send("GIt FileSYncronization needs attension", &msg);
                continue;
            }
            let to_file = s.file();
            debug!("Status: {:?}", s);
            if s.index == 'D'
            {
                continue;
            };
            //let msg = format!("{} modified", to_file);
            //notify::send("gifsy sync", &msg);
            let output = Command::new("git")
                .current_dir(&self.path)
                .arg("add")
                .arg(&to_file)
                .output()
                .expect("can't execute git add");

            if !output.status.success()
            {
                return Err(GifsyError::CmdFail(
                    output.status.code().unwrap_or(-5),
                    format!(
                        "can't add {} ({})",
                        &to_file,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                ));
            }
            rc.push(s.clone());
        }
        Ok(rc)
    }
    pub fn commit(&self, status: Vec<Box<Status>>) -> Result<(), GifsyError> {
        let process = match Command::new("git")
            .current_dir(&self.path)
            .arg("commit")
            .arg("--file")
            .arg("-")
            .stdin(Stdio::piped())
            .spawn()
        {
            Err(e) => return Err(GifsyError::IoError(e)),
            Ok(process) => process,
        };
        let msg = create_commit_message(&status, &self.name).unwrap();
        match process.stdin.unwrap().write_all(msg.as_bytes())
        {
            Err(e) => Err(GifsyError::IoError(e)),
            Ok(_) =>
            {
                let mut msg = String::from("the following files have been changed:\n\n");
                if !status.is_empty()
                {
                    for s in &status
                    {
                        let f = format!("  {}\n", &s.file());
                        msg += &f;
                    }
                    notify::send("GIt FileSYncronization Files Modified", &msg);
                }
                Ok(())
            }
        }
    }
    pub fn pull(&self) -> Result<(), GifsyError> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .arg("pull")
            .arg("origin")
            .arg("--rebase")
            .arg("--autostash")
            .output()
            .expect("can't execute git pull origin");

        debug!(
            "pull output stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        debug!(
            "pull output stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        debug!("pull status: {}", output.status);
        if output.status.success()
        {
            match output.status.code()
            {
                Some(rc) =>
                {
                    if rc != 0
                    {
                      dbg!(&output);
                        Err(GifsyError::CmdFail(
                            rc,
                            format!(
                                "pull failed: {} err: {}",
                                String::from_utf8_lossy(&output.stdout),
                                String::from_utf8_lossy(&output.stderr)
                            ),
                        ))
                    }
                    else
                    {
                        Ok(())
                    }
                }
                None => Ok(()),
            }
        }
        else
        {
            Err(GifsyError::CmdFail(
                output.status.code().unwrap_or(-1),
                format!(
                    "couldn't call git pull: {} err: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ),
            ))
        }
    }
    pub fn push(&self) -> Result<(), GifsyError> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .arg("push")
            .arg("origin")
            .output()
            .expect("can't execute git push");

        debug!(
            "push output stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        debug!(
            "push output stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        debug!("push status: {}", output.status);
        if output.status.success()
        {
            match output.status.code()
            {
                Some(rc) =>
                {
                    if rc != 0
                    {
                        Err(GifsyError::CmdFail(
                            rc,
                            format!(
                                "can't push: {} err: {}",
                                String::from_utf8_lossy(&output.stdout),
                                String::from_utf8_lossy(&output.stderr)
                            ),
                        ))
                    }
                    else
                    {
                        Ok(())
                    }
                }
                None => Ok(()),
            }
        }
        else
        {
            Err(GifsyError::CmdFail(
                output.status.code().unwrap_or(-2),
                format!(
                    "couldn't call git push: {} err: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ),
            ))
        }
    }
    pub fn submodules_init(&self) -> Result<(), GifsyError> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .arg("submodule")
            .arg("init")
            .output()
            .expect("can't execute git submodule init");

        if output.status.success()
        {
            match output.status.code()
            {
                Some(rc) =>
                {
                    if rc != 0
                    {
                        Err(GifsyError::CmdFail(
                            rc,
                            format!(
                                "can't init submodules: {} err: {}",
                                String::from_utf8_lossy(&output.stdout),
                                String::from_utf8_lossy(&output.stderr)
                            ),
                        ))
                    }
                    else
                    {
                        Ok(())
                    }
                }
                None => Ok(()),
            }
        }
        else
        {
            Err(GifsyError::CmdFail(
                output.status.code().unwrap_or(-3),
                format!(
                    "couldn't call init submodules: {} err: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ),
            ))
        }
    }
    pub fn submodules_update(&self) -> Result<(), GifsyError> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .arg("submodule")
            .arg("update")
            .output()
            .expect("can't execute git submodule update");

        if output.status.success()
        {
            match output.status.code()
            {
                Some(rc) =>
                {
                    if rc != 0
                    {
                        Err(GifsyError::CmdFail(
                            rc,
                            format!(
                                "can't update submodules: {} err: {}",
                                String::from_utf8_lossy(&output.stdout),
                                String::from_utf8_lossy(&output.stderr)
                            ),
                        ))
                    }
                    else
                    {
                        Ok(())
                    }
                }
                None => Ok(()),
            }
        }
        else
        {
            Err(GifsyError::CmdFail(
                output.status.code().unwrap_or(-4),
                format!(
                    "couldn't call update submodules: {} err: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ),
            ))
        }
    }
}

#[derive(Clone)]
pub struct Status {
    index: char,
    tree: char,
    from_file: String,
    to_file: String,
}

impl Status {
    pub fn is_unmerged(&self) -> bool {
        self.index == 'U' || self.tree == 'U'
    }
    pub fn file(&self) -> String {
        if self.to_file.is_empty()
        {
            debug!("form file ({})", self.to_file.len());
            self.from_file.clone()
        }
        else
        {
            debug!("from file");
            self.to_file.clone()
        }
    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.to_file.is_empty()
        {
            write!(f, "{}{} {}", self.index, self.tree, self.from_file)
        }
        else
        {
            write!(
                f,
                "{}{} {} -> {}",
                self.index, self.tree, self.from_file, self.to_file
            )
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.to_file.is_empty()
        {
            write!(f, "  {} {}", encode_status_flag(self.index), self.from_file)
        }
        else
        {
            write!(
                f,
                "  {} {} -> {}",
                encode_status_flag(self.index),
                self.from_file,
                self.to_file
            )
        }
    }
}

pub fn create_commit_message(
    status: &Vec<Box<Status>>,
    name: &str,
) -> Result<String, FromUtf8Error> {
    let mut commitmsg = Vec::new();
    writeln!(
        &mut commitmsg,
        "changes on {} at {}\n",
        name,
        Local::now().to_rfc2822()
    )
    .unwrap();
    for s in status
    {
        writeln!(&mut commitmsg, "{}", s).unwrap();
    }
    String::from_utf8(commitmsg)
}

fn encode_status_flag(flag: char) -> char {
    match flag
    {
        'M' => '~',
        'A' => '+',
        'D' => '-',
        'R' => '>',
        'U' => '!',
        '?' => '?',
        ' ' => ' ',
        _ => '•',
    }
}

extern crate git2;
extern crate colored;
use colored::*;
use std::io::{self, Write};

enum Error {
    Git2Error(git2::Error),
    IoError(io::Error)
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Self {
        Error::Git2Error(e)
    }
}

fn main() {
    if let Ok(repo) = git2::Repository::discover(".") {
        if repo.is_bare(){
            return
        }
        let mut stdout = io::stdout();
        if let Err(e) = state(&repo, &mut stdout){
            match e {
                Error::Git2Error(e) => write!(&mut io::stderr(), "Error: {}", e.message()).expect("Could not write to stderr"),
                Error::IoError(e) => write!(&mut io::stderr(), "Error: {}", e).expect("Could not write to stderr"),
            }
        }
        if let Err(e) = files(&repo, &mut stdout){
            match e {
                Error::Git2Error(e) => write!(&mut io::stderr(), "Error: {}", e.message()).expect("Could not write to stderr"),
                Error::IoError(e) => write!(&mut io::stderr(), "Error: {}", e).expect("Could not write to stderr"),
            }
        }
    }
}
/// Print state information about a repo
/// Gives...
/// - The current branch
/// - Ahead/Behind stats for remote tracking branches (if any)
/// - Sexy Powerline-compatible symbols
/// - Colours
/// 
/// All operations based on HEAD
fn state<W: Write>(repo: &git2::Repository, writer: &mut W) -> Result<(), Error> {
    if let Some(name) = repo.head()?.shorthand() {
        write!(writer, "on {} {} ", "".bright_purple().bold(), name.white().bold())?;
    }

    let branch = git2::Branch::wrap(repo.head()?);
    if let (Some(local_oid), Some(remote_oid)) = (repo.head()?.target_peel(), branch.upstream()?.get().target_peel()){
        if let Ok((ahead, behind)) = repo.graph_ahead_behind(local_oid, remote_oid) {
            write!(writer, " ↑{} ↓{}", ahead, behind)?;
        }
    }
    Ok(())
}
/// Print file statuses for a repo
/// Gives...
/// - Files added (Green +)
/// - Files modified (Yellow *)
/// - Files deleted (Red -)
/// - Files not yet tracked (Cyan +)
/// - Files with merge conflicts (Red !)
/// 
/// All operations based on HEAD
fn files<W: Write>(repo: &git2::Repository, writer: &mut W) -> Result<(), Error> {
    use git2::Status;
    let new_status = Status::INDEX_NEW | Status::INDEX_MODIFIED | Status::INDEX_RENAMED | Status::INDEX_TYPECHANGE;
    let mod_status = Status::WT_MODIFIED | Status::WT_TYPECHANGE | Status::WT_RENAMED;
    let del_status = Status::INDEX_DELETED | Status::WT_DELETED;
    let untr_status = Status::WT_NEW;
    let confl_status = Status::CONFLICTED;

    let mut opts = git2::StatusOptions::default();
    let opts = opts.include_untracked(true);
    let (n_new, n_mod, n_del, n_untr, n_confl) = repo.statuses(Some(opts))?.iter()
        .map(|s| s.status())
        .fold((0, 0, 0, 0, 0), |(mut n_new, mut n_mod, mut n_del, mut n_untr, mut n_confl), s| {
            match s {
                _ if s.intersects(new_status) => n_new += 1,
                _ if s.intersects(mod_status) => n_mod += 1,
                _ if s.intersects(del_status) => n_del += 1,
                _ if s.intersects(untr_status) => n_untr += 1,
                _ if s.intersects(confl_status) => n_confl += 1,
                _ => {}
            }
            (n_new, n_mod, n_del, n_untr, n_confl)
        });

    if n_new + n_mod + n_del + n_untr == 0 {
        write!(writer, "{} ", "✓".green().bold())?;
    } else {
        if n_new > 0 {
            write!(writer, "{}{} ", "+".green().bold(), n_new)?;
        }
        if n_mod > 0{
            write!(writer, "{}{} ", "*".yellow().bold(), n_mod)?;
        }
        if n_del > 0{
            write!(writer, "{}{} ", "-".red().bold(), n_del)?;
        }
        if n_untr > 0{
            write!(writer, "{}{} ", "+".cyan().bold(), n_untr)?;
        }
        if n_confl > 0{
            write!(writer, "{}{} ", "!".red().bold(), n_confl)?;
        }
    }
    Ok(())
}

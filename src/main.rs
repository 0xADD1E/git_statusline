extern crate colored;
extern crate failure;
extern crate git2;
use colored::*;
use failure::Error;
use std::env::var;
use std::io::{self, Write};

fn main() -> Result<(), Error> {
    let repo = git2::Repository::discover(".")?;
    if !repo.is_bare() {
        let mut stdout = io::stdout();
        let (name, graph_res) = state(&repo)?;
        if let Some(name) = name {
            write!(
                stdout,
                "on {} {} ",
                "".bright_purple().bold(),
                name.white().bold()
            )?;
        }
        if let Some((ahead, behind)) = graph_res {
            write!(stdout, " ↑{} ↓{}", ahead, behind)?;
        }

        if let Ok(disable) = var("STATUSLINE_DISABLE") {
            if !disable.is_empty() {
                return Ok(());
            }
        }
        let (n_new, n_mod, n_del, n_untr, n_confl) = files(&repo)?;
        if n_new + n_mod + n_del + n_untr == 0 {
            write!(stdout, "{} ", "✓".green().bold())?;
        } else {
            if n_new > 0 {
                write!(stdout, "{}{} ", "+".green().bold(), n_new)?;
            }
            if n_mod > 0 {
                write!(stdout, "{}{} ", "*".yellow().bold(), n_mod)?;
            }
            if n_del > 0 {
                write!(stdout, "{}{} ", "-".red().bold(), n_del)?;
            }
            if n_untr > 0 {
                write!(stdout, "{}{} ", "+".cyan().bold(), n_untr)?;
            }
            if n_confl > 0 {
                write!(stdout, "{}{} ", "!".red().bold(), n_confl)?;
            }
        }
    }
    Ok(())
}
/// Print state information about a repo
/// Gives...
/// - The current branch
/// - Ahead/Behind stats for remote tracking branches (if any)
/// - Sexy Powerline-compatible symbols
/// - Colours
///
/// All operations based on HEAD
fn state(repo: &git2::Repository) -> Result<(Option<String>, Option<(usize, usize)>), git2::Error> {
    let name = match repo.head()?.shorthand() {
        Some(name) => Some(String::from(name)),
        None => None,
    };

    let branch = git2::Branch::wrap(repo.head()?);
    let graph = match (
        repo.head()?.target_peel(),
        branch.upstream()?.get().target_peel(),
    ) {
        (Some(local), Some(remote)) => Some(repo.graph_ahead_behind(local, remote)?),
        _ => None,
    };
    Ok((name, graph))
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
fn files(repo: &git2::Repository) -> Result<(u32, u32, u32, u32, u32), git2::Error> {
    use git2::Status;
    let new_status = Status::INDEX_NEW
        | Status::INDEX_MODIFIED
        | Status::INDEX_RENAMED
        | Status::INDEX_TYPECHANGE;
    let mod_status = Status::WT_MODIFIED | Status::WT_TYPECHANGE | Status::WT_RENAMED;
    let del_status = Status::INDEX_DELETED | Status::WT_DELETED;
    let untr_status = Status::WT_NEW;
    let confl_status = Status::CONFLICTED;

    let mut opts = git2::StatusOptions::default();
    let opts = opts.include_untracked(true);
    let statuses = repo.statuses(Some(opts))?.iter().map(|s| s.status()).fold(
        (0, 0, 0, 0, 0),
        |(mut n_new, mut n_mod, mut n_del, mut n_untr, mut n_confl), s| {
            match s {
                _ if s.intersects(new_status) => n_new += 1,
                _ if s.intersects(mod_status) => n_mod += 1,
                _ if s.intersects(del_status) => n_del += 1,
                _ if s.intersects(untr_status) => n_untr += 1,
                _ if s.intersects(confl_status) => n_confl += 1,
                _ => {}
            }
            (n_new, n_mod, n_del, n_untr, n_confl)
        },
    );
    Ok(statuses)
}

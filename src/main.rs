extern crate git2;
extern crate colored;
use colored::*;

fn main() {
    if let Ok(repo) = git2::Repository::open(".") {
        if repo.is_bare(){
            return
        }
        if let Ok(state) = state(&repo){
            print!("{}", state);
        }
        if let Ok(files) = files(&repo){
            print!("{}", files);
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
fn state(repo: &git2::Repository) -> Result<String, git2::Error> {
    let mut state = String::new();
    if let Some(name) = repo.head()?.shorthand() {
        state += &format!("on {} {} ", "".bright_purple().bold(), name.white().bold());
    }

    let branch = git2::Branch::wrap(repo.head()?);
    if let (Some(local_oid), Some(remote_oid)) = (repo.head()?.target_peel(), branch.upstream()?.get().target_peel()){
        if let Ok((ahead, behind)) = repo.graph_ahead_behind(local_oid, remote_oid) {
            state += &format!(" ↑{} ↓{}", ahead, behind);
        }
    }
    Ok(state)
}
/// Print file statuses for a repo
/// Gives...
/// - Files added (Green +)
/// - Files modified (Yellow *)
/// - Files deleted (Red -)
/// - Files not yet tracked (Cyan +)
/// 
/// All operations based on HEAD
fn files(repo: &git2::Repository) -> Result<String, git2::Error> {
    let mut files = String::new();
    let mut opts = git2::StatusOptions::default();
    let opts = opts.include_untracked(true);
    let (mut n_new, mut n_mod, mut n_del, mut n_untr) = (0, 0, 0, 0);
    for status in repo.statuses(Some(opts))?.iter() {
        let s = status.status();
        use git2::Status;

        if s.intersects(Status::INDEX_NEW) {
            n_new += 1;
        }
        if s.intersects(Status::INDEX_MODIFIED | Status::INDEX_RENAMED | Status::INDEX_TYPECHANGE | Status::WT_MODIFIED | Status::WT_TYPECHANGE | Status::WT_RENAMED | Status::CONFLICTED){
            n_mod += 1;
        }
        if s.intersects(Status::INDEX_DELETED | Status::WT_DELETED){
            n_del += 1;
        }
        if s.intersects(Status::WT_NEW){
            n_untr += 1;
        }
    }

    if n_new + n_mod + n_del + n_untr == 0 {
        files += &format!("{} ", "✓".green().bold());
    } else {
        if n_new > 0 {
            files += &format!("{}{} ", "+".green().bold(), n_new);
        }
        if n_mod > 0{
            files += &format!("{}{} ", "*".yellow().bold(), n_mod);
        }
        if n_del > 0{
            files += &format!("{}{} ", "-".red().bold(), n_del);
        }
        if n_untr > 0{
            files += &format!("{}{} ", "+".cyan().bold(), n_untr);
        }
    }
    Ok(files)
}

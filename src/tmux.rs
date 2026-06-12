//! tmux process interaction: data models and command wrappers.
//!
//! Everything that shells out to `tmux` lives here, so the rest of the crate
//! deals in plain data and `Result`s rather than raw `Command`s.

use std::io;
use std::process::Command;

/// A pane as reported by tmux, with its cell geometry within its window.
#[derive(Clone)]
pub struct Pane {
    pub id: String,
    pub index: String,
    pub left: u16,
    pub top: u16,
    pub width: u16,
    pub height: u16,
    pub active: bool,
    pub cmd: String,
}

/// A window and the panes it contains.
#[derive(Clone)]
pub struct Win {
    pub id: String,
    pub index: String,
    pub name: String,
    pub active: bool,
    pub w: u16,
    pub h: u16,
    pub panes: Vec<Pane>,
}

fn out(args: &[&str]) -> io::Result<String> {
    let o = Command::new("tmux").args(args).output()?;
    Ok(String::from_utf8_lossy(&o.stdout).to_string())
}

fn fire(args: &[&str]) -> io::Result<()> {
    Command::new("tmux").args(args).status()?;
    Ok(())
}

/// Run a tmux command, returning `Ok(stdout)` on success or `Err(stderr)` on
/// a non-zero exit (the outer `io::Result` is for spawn failures).
fn capture(args: &[&str]) -> io::Result<Result<String, String>> {
    let o = Command::new("tmux").args(args).output()?;
    Ok(if o.status.success() {
        Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&o.stderr).trim().to_string())
    })
}

fn parse_pane(f: &[&str]) -> Pane {
    Pane {
        id: f[0].to_string(),
        index: f[1].to_string(),
        left: f[2].parse().unwrap_or(0),
        top: f[3].parse().unwrap_or(0),
        width: f[4].parse().unwrap_or(1),
        height: f[5].parse().unwrap_or(1),
        active: f[6].trim() == "1",
        cmd: f[7].to_string(),
    }
}

const PANE_FMT: &str = "#{pane_id}\t#{pane_index}\t#{pane_left}\t#{pane_top}\t#{pane_width}\t#{pane_height}\t#{pane_active}\t#{pane_current_command}";

/// Current window dimensions and the geometry of each of its panes.
pub fn query() -> io::Result<(u16, u16, Vec<Pane>)> {
    let dims = out(&["display-message", "-p", "#{window_width} #{window_height}"])?;
    let mut it = dims.split_whitespace();
    let win_w = it.next().and_then(|s| s.parse().ok()).unwrap_or(80u16);
    let win_h = it.next().and_then(|s| s.parse().ok()).unwrap_or(24u16);
    let raw = out(&["list-panes", "-F", PANE_FMT])?;
    let mut panes = Vec::new();
    for line in raw.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() >= 8 {
            panes.push(parse_pane(&f));
        }
    }
    Ok((win_w, win_h, panes))
}

/// Every window in the session, each with its panes, ordered by index.
pub fn query_windows() -> io::Result<Vec<Win>> {
    let fmt = "#{window_id}\t#{window_index}\t#{window_name}\t#{window_active}\t#{window_width}\t#{window_height}\t#{pane_id}\t#{pane_index}\t#{pane_left}\t#{pane_top}\t#{pane_width}\t#{pane_height}\t#{pane_active}\t#{pane_current_command}";
    let raw = out(&["list-panes", "-s", "-F", fmt])?;
    let mut wins: Vec<Win> = Vec::new();
    for line in raw.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 14 {
            continue;
        }
        let pane = parse_pane(&f[6..14]);
        let wid = f[0].to_string();
        if let Some(w) = wins.iter_mut().find(|w| w.id == wid) {
            w.panes.push(pane);
        } else {
            wins.push(Win {
                id: wid,
                index: f[1].to_string(),
                name: f[2].to_string(),
                active: f[3].trim() == "1",
                w: f[4].parse().unwrap_or(80),
                h: f[5].parse().unwrap_or(24),
                panes: vec![pane],
            });
        }
    }
    wins.sort_by_key(|w| w.index.parse::<i64>().unwrap_or(0));
    Ok(wins)
}

/// Split `target` and return the new pane id. `flag` is "-h"/"-v"; `before`
/// puts the new pane left/above instead of right/below.
pub fn split(target: &str, flag: &str, before: bool) -> io::Result<Result<String, String>> {
    let mut args = vec!["split-window", flag];
    if before {
        args.push("-b");
    }
    args.extend(["-t", target, "-P", "-F", "#{pane_id}"]);
    capture(&args)
}

pub fn select_layout(name: &str) -> io::Result<Result<String, String>> {
    capture(&["select-layout", name])
}

pub fn join_pane(pane: &str, win: &str) -> io::Result<Result<String, String>> {
    capture(&["join-pane", "-s", pane, "-t", win])
}

pub fn swap_pane(s: &str, t: &str) -> io::Result<()> {
    fire(&["swap-pane", "-s", s, "-t", t])
}

pub fn kill_pane(id: &str) -> io::Result<()> {
    fire(&["kill-pane", "-t", id])
}

pub fn kill_window(id: &str) -> io::Result<()> {
    fire(&["kill-window", "-t", id])
}

pub fn next_layout() -> io::Result<()> {
    fire(&["next-layout"])
}

pub fn select_window(id: &str) -> io::Result<()> {
    fire(&["select-window", "-t", id])
}

pub fn swap_window(a: &str, b: &str) -> io::Result<()> {
    fire(&["swap-window", "-d", "-s", a, "-t", b])
}

pub fn rename_window(id: &str, name: &str) -> io::Result<()> {
    fire(&["rename-window", "-t", id, name])
}

pub fn new_window() -> io::Result<()> {
    fire(&["new-window", "-a"])
}

/// Focus a specific pane by id.
pub fn select_pane(id: &str) -> io::Result<()> {
    fire(&["select-pane", "-t", id])
}

/// Move focus to the neighbouring pane; `dir` is "-L"/"-R"/"-U"/"-D".
pub fn select_pane_dir(dir: &str) -> io::Result<()> {
    fire(&["select-pane", dir])
}

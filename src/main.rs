//! tdrag — a mouse-driven drag-and-drop layout manager for tmux.
//!
//! Two views, toggled with Tab (or the `[ windows ]` / `[ panes ]` button):
//!
//! * Pane mode — the current window's panes as a scaled schematic. Drag to
//!   swap, click to select, `|`/`-` to split, `x` to kill, `1`..`5`/space to
//!   change layout, right-click for a context menu.
//! * Window mode (mission control) — every window as a tile drawn with its own
//!   mini pane-layout. Drag a window onto another to reorder, drag a pane into
//!   another window to relocate it, click to switch, right-click to
//!   rename / create / close.
//!
//! You drag schematic boxes, not the live pane contents — true live-pane
//! dragging is the thing tmux core itself can't do yet (tmux#3503).

mod app;
mod geometry;
mod menu;
mod tmux;
mod ui;

use std::io::{self, stdout, Stdout};

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::Terminal;

use app::App;

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    let mut app = App::new()?;
    loop {
        terminal.draw(|f| app.draw(f))?;

        match event::read()? {
            Event::Key(k) => {
                // rename prompt intercepts keys (text input)
                if app.prompt.is_some() {
                    match k.code {
                        KeyCode::Enter => app.confirm_rename()?,
                        KeyCode::Esc => {
                            app.prompt = None;
                            app.status = "rename cancelled".into();
                        }
                        KeyCode::Backspace => {
                            if let Some(p) = &mut app.prompt {
                                p.buf.pop();
                            }
                        }
                        KeyCode::Char(c) => {
                            if let Some(p) = &mut app.prompt {
                                p.buf.push(c);
                            }
                        }
                        _ => {}
                    }
                    continue;
                }
                // confirm-kill intercepts keys
                if app.pending_kill.is_some() {
                    match k.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                            app.confirm_kill()?
                        }
                        _ => {
                            app.pending_kill = None;
                            app.status = "cancelled".into();
                        }
                    }
                    continue;
                }
                if app.show_help {
                    app.show_help = false;
                    continue;
                }
                if app.menu.is_some() {
                    match k.code {
                        KeyCode::Esc | KeyCode::Char('q') => app.menu = None,
                        KeyCode::Up => {
                            if let Some(menu) = &mut app.menu {
                                menu.highlight = menu.highlight.saturating_sub(1);
                            }
                        }
                        KeyCode::Down => {
                            if let Some(menu) = &mut app.menu {
                                if menu.highlight + 1 < menu.items.len() {
                                    menu.highlight += 1;
                                }
                            }
                        }
                        KeyCode::Enter => {
                            let h = app.menu.as_ref().map(|m| m.highlight).unwrap_or(0);
                            app.activate_menu(h)?;
                        }
                        _ => {}
                    }
                    continue;
                }
                // Global keys, then per-mode dispatch.
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('?') => app.show_help = true,
                    KeyCode::Tab => app.set_window_mode(!app.window_mode)?,
                    KeyCode::Char('w') => app.set_window_mode(true)?,
                    KeyCode::Char('p') => app.set_window_mode(false)?,
                    KeyCode::Char('r') => {
                        app.refresh()?;
                        app.status = "refreshed".into();
                    }
                    KeyCode::Char('R') => {
                        if let Some(id) = app.active_window_id() {
                            app.open_rename(id);
                        }
                    }
                    _ if app.window_mode => app.window_key(k)?,
                    _ => app.pane_key(k)?,
                }
            }
            Event::Mouse(m) => app.on_mouse(m)?,
            _ => {}
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn main() -> io::Result<()> {
    if std::env::var_os("TMUX").is_none() {
        eprintln!("tdrag: not inside a tmux session ($TMUX unset). Run it from tmux.");
        std::process::exit(1);
    }

    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let res = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    res
}

//! Application state and behavior: what the manager knows and what each
//! gesture does. Rendering lives in [`crate::ui`].

use std::cell::{Cell, RefCell};
use std::io;

use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Rect;

use crate::geometry::{contains, hit, mpane_at, rects_for, tile_at};
use crate::menu::{build_menu_items, Action, Menu};
use crate::tmux::{self, Pane, Win};

/// What is being dragged in window mode.
pub enum WDrag {
    /// A whole window (drop on another to reorder).
    Win(String),
    /// A pane being moved (drop on another window to relocate it).
    Pane { win: String, pane: String },
}

/// A modal text-input prompt (currently used only for renaming a window).
pub struct Prompt {
    pub win: String,
    pub buf: String,
}

pub struct App {
    // pane mode (current window)
    pub win_w: u16,
    pub win_h: u16,
    pub panes: Vec<Pane>,
    pub drag_src: Option<usize>,
    pub selected: Option<String>, // pane id
    // window mode
    pub windows: Vec<Win>,
    pub window_mode: bool,
    pub wdrag: Option<WDrag>,
    pub sel_win: Option<String>,                      // window id
    pub tiles: RefCell<Vec<(String, Rect)>>,          // window id -> tile rect (last draw)
    pub mpanes: RefCell<Vec<(String, String, Rect)>>, // (win, pane) -> mini rect
    // shared / modal
    pub pending_kill: Option<(String, bool)>, // (id, is_window) awaiting confirm
    pub prompt: Option<Prompt>,
    pub menu: Option<Menu>,
    pub show_help: bool,
    pub should_quit: bool,
    pub cursor: (u16, u16),
    pub status: String,
    pub last_canvas: Cell<Rect>,
    pub last_full: Cell<Rect>,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let (win_w, win_h, panes) = tmux::query()?;
        Ok(Self {
            win_w,
            win_h,
            panes,
            drag_src: None,
            selected: None,
            windows: tmux::query_windows().unwrap_or_default(),
            window_mode: false,
            wdrag: None,
            sel_win: None,
            tiles: RefCell::new(Vec::new()),
            mpanes: RefCell::new(Vec::new()),
            pending_kill: None,
            prompt: None,
            menu: None,
            show_help: false,
            should_quit: false,
            cursor: (0, 0),
            status: "ready".into(),
            last_canvas: Cell::new(Rect::default()),
            last_full: Cell::new(Rect::default()),
        })
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        let (w, h, p) = tmux::query()?;
        self.win_w = w;
        self.win_h = h;
        self.panes = p;
        self.windows = tmux::query_windows().unwrap_or_default();
        self.drag_src = None;
        self.wdrag = None;
        if let Some(sel) = &self.selected {
            if !self.panes.iter().any(|p| &p.id == sel) {
                self.selected = None;
            }
        }
        if let Some(sw) = &self.sel_win {
            if !self.windows.iter().any(|w| &w.id == sw) {
                self.sel_win = None;
            }
        }
        Ok(())
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected.as_ref().and_then(|id| self.panes.iter().position(|p| &p.id == id))
    }

    /// Target pane for create/kill: the selected pane, else the active one.
    fn target(&self) -> Option<String> {
        self.selected.clone().or_else(|| self.panes.iter().find(|p| p.active).map(|p| p.id.clone()))
    }

    /// Id of the currently active (focused) window.
    pub fn active_window_id(&self) -> Option<String> {
        self.windows.iter().find(|w| w.active).map(|w| w.id.clone())
    }

    /// ` index:name ` of the active window, shown in the top bar.
    pub fn active_window_label(&self) -> Option<String> {
        self.windows.iter().find(|w| w.active).map(|w| format!(" {}:{} ", w.index, w.name))
    }

    // --- pane operations ---

    pub fn split(&mut self, horizontal: bool) -> io::Result<()> {
        let dir = if horizontal { "-h" } else { "-v" };
        self.new_pane(dir, false)
    }

    /// Create a pane next to the target. `flag` is "-h"/"-v"; `before` puts the
    /// new pane left/above instead of right/below.
    pub fn new_pane(&mut self, flag: &str, before: bool) -> io::Result<()> {
        let Some(t) = self.target() else {
            self.status = "no pane to split from".into();
            return Ok(());
        };
        match tmux::split(&t, flag, before)? {
            Ok(new_id) => {
                self.refresh()?;
                if !new_id.is_empty() {
                    self.selected = Some(new_id);
                }
                self.status = "new pane".into();
            }
            Err(e) => self.status = format!("split failed: {e}"),
        }
        Ok(())
    }

    pub fn request_kill(&mut self) {
        if self.panes.len() <= 1 {
            self.status = "only one pane — refusing to kill the last one".into();
            return;
        }
        match self.target() {
            Some(id) => {
                self.status = format!("kill {id}?  Yes / No");
                self.pending_kill = Some((id, false));
            }
            None => self.status = "no pane selected".into(),
        }
    }

    pub fn set_layout(&mut self, name: &str, label: &str) -> io::Result<()> {
        match tmux::select_layout(name)? {
            Ok(_) => {
                self.refresh()?;
                self.status = format!("layout: {label}");
            }
            Err(e) => self.status = format!("layout failed: {e}"),
        }
        Ok(())
    }

    pub fn cycle_layout(&mut self) -> io::Result<()> {
        tmux::next_layout()?;
        self.refresh()?;
        self.status = "cycled layout".into();
        Ok(())
    }

    // --- window operations ---

    pub fn select_window(&mut self, id: &str) -> io::Result<()> {
        tmux::select_window(id)?;
        self.refresh()?;
        self.sel_win = Some(id.to_string());
        self.status = format!("window {id}");
        Ok(())
    }

    pub fn swap_windows(&mut self, a: &str, b: &str) -> io::Result<()> {
        tmux::swap_window(a, b)?;
        self.refresh()?;
        self.status = format!("reordered {a} ⇄ {b}");
        Ok(())
    }

    pub fn move_pane_to_window(&mut self, pane: &str, win: &str) -> io::Result<()> {
        match tmux::join_pane(pane, win)? {
            Ok(_) => {
                self.refresh()?;
                self.status = format!("moved {pane} → window {win}");
            }
            Err(e) => self.status = format!("move failed: {e}"),
        }
        Ok(())
    }

    pub fn new_window(&mut self) -> io::Result<()> {
        tmux::new_window()?;
        self.refresh()?;
        self.status = "new window".into();
        Ok(())
    }

    pub fn request_kill_window(&mut self, id: String) {
        if self.windows.len() <= 1 {
            self.status = "only one window — refusing to close the last one".into();
            return;
        }
        self.status = format!("close window {id}?  Yes / No");
        self.pending_kill = Some((id, true));
    }

    pub fn open_rename(&mut self, id: String) {
        let name =
            self.windows.iter().find(|w| w.id == id).map(|w| w.name.clone()).unwrap_or_default();
        self.status = "rename: type a name, Enter to confirm".into();
        self.prompt = Some(Prompt { win: id, buf: name });
    }

    pub fn confirm_rename(&mut self) -> io::Result<()> {
        if let Some(p) = self.prompt.take() {
            if !p.buf.is_empty() {
                tmux::rename_window(&p.win, &p.buf)?;
                self.status = format!("renamed → {}", p.buf);
            } else {
                self.status = "rename cancelled (empty)".into();
            }
            self.refresh()?;
        }
        Ok(())
    }

    // --- kill confirmation (shared by panes and windows) ---

    pub fn confirm_kill(&mut self) -> io::Result<()> {
        if let Some((id, is_win)) = self.pending_kill.take() {
            if is_win {
                tmux::kill_window(&id)?;
                if self.sel_win.as_deref() == Some(id.as_str()) {
                    self.sel_win = None;
                }
                self.status = format!("closed window {id}");
            } else {
                tmux::kill_pane(&id)?;
                if self.selected.as_deref() == Some(id.as_str()) {
                    self.selected = None;
                }
                self.status = format!("killed {id}");
            }
            self.refresh()?;
        }
        Ok(())
    }

    // --- menus ---

    pub fn open_menu(&mut self, name: &str, x: u16, y: u16, target: Option<String>) {
        let (title, items) = build_menu_items(name, &target, self.panes.len(), self.windows.len());
        self.menu = Some(Menu { x, y, target, title, items, highlight: 0 });
    }

    pub fn menu_rect(&self, menu: &Menu) -> Rect {
        let full = self.last_full.get();
        let label_w = menu.items.iter().map(|(l, _)| l.chars().count()).max().unwrap_or(10) as u16;
        let w = (label_w + 4).min(full.width.max(1));
        let h = (menu.items.len() as u16 + 2).min(full.height.max(1));
        let x = menu.x.min(full.right().saturating_sub(w));
        let y = menu.y.min(full.bottom().saturating_sub(h));
        Rect { x, y, width: w, height: h }
    }

    pub fn menu_item_at(&self, col: u16, row: u16) -> Option<usize> {
        let menu = self.menu.as_ref()?;
        let area = self.menu_rect(menu);
        if !contains(area, col, row) {
            return None;
        }
        let i = row.checked_sub(area.y + 1)? as usize;
        (i < menu.items.len()).then_some(i)
    }

    pub fn activate_menu(&mut self, idx: usize) -> io::Result<()> {
        let Some(menu) = self.menu.as_ref() else {
            return Ok(());
        };
        let Some((_, action)) = menu.items.get(idx).cloned() else {
            return Ok(());
        };
        let target = menu.target.clone();

        // Submenu navigation keeps the menu open and swaps its contents.
        if let Action::Submenu(name) = action {
            let (title, items) =
                build_menu_items(name, &target, self.panes.len(), self.windows.len());
            if let Some(menu) = &mut self.menu {
                menu.title = title;
                menu.items = items;
                menu.highlight = 0;
            }
            return Ok(());
        }

        self.menu = None;
        match action {
            Action::Kill => {
                self.selected = target;
                self.request_kill();
            }
            Action::Layout(name, label) => self.set_layout(name, label)?,
            Action::NewPane(flag, before) => {
                self.selected = target;
                self.new_pane(flag, before)?;
            }
            Action::RenameWindow => {
                // In window mode the menu targets a tile; in pane mode the
                // menu targets a pane, so rename the active window instead.
                let id = if self.window_mode { target } else { self.active_window_id() };
                if let Some(id) = id {
                    self.open_rename(id);
                }
            }
            Action::NewWindow => self.new_window()?,
            Action::KillWindow => {
                if let Some(t) = target {
                    self.request_kill_window(t);
                }
            }
            Action::Submenu(_) => unreachable!(),
        }
        Ok(())
    }

    // --- chrome rectangles (shared by mouse hit-testing and drawing) ---

    pub fn quit_button_rect(&self) -> Rect {
        let full = self.last_full.get();
        let w = 8u16.min(full.width.max(1));
        Rect { x: full.x, y: full.y, width: w, height: 1 }
    }

    pub fn mode_button_rect(&self) -> Rect {
        let full = self.last_full.get();
        let w = 11u16.min(full.width.max(1));
        let x = (full.x + 9).min(full.right().saturating_sub(w));
        Rect { x, y: full.y, width: w, height: 1 }
    }

    pub fn help_button_rect(&self) -> Rect {
        let full = self.last_full.get();
        let w = 10u16.min(full.width.max(1));
        Rect { x: full.right().saturating_sub(w), y: full.y, width: w, height: 1 }
    }

    /// (dialog area, Yes button, No button) for the kill confirmation.
    pub fn confirm_rects(&self) -> (Rect, Rect, Rect) {
        let full = self.last_full.get();
        let w = 34u16.min(full.width.max(1));
        let h = 5u16.min(full.height.max(1));
        let area = Rect {
            x: full.x + full.width.saturating_sub(w) / 2,
            y: full.y + full.height.saturating_sub(h) / 2,
            width: w,
            height: h,
        };
        let by = area.y + area.height.saturating_sub(2);
        let yes = Rect { x: area.x + 4, y: by, width: 7, height: 1 };
        let no = Rect { x: area.x + area.width.saturating_sub(10), y: by, width: 6, height: 1 };
        (area, yes, no)
    }

    /// (dialog area, text field, OK button, Cancel button) for the rename prompt.
    pub fn prompt_rects(&self) -> (Rect, Rect, Rect, Rect) {
        let full = self.last_full.get();
        let w = 44u16.min(full.width.max(1));
        let h = 6u16.min(full.height.max(1));
        let area = Rect {
            x: full.x + full.width.saturating_sub(w) / 2,
            y: full.y + full.height.saturating_sub(h) / 2,
            width: w,
            height: h,
        };
        let field =
            Rect { x: area.x + 2, y: area.y + 2, width: area.width.saturating_sub(4), height: 1 };
        let by = area.y + area.height.saturating_sub(2);
        let ok = Rect { x: area.x + 4, y: by, width: 6, height: 1 };
        let cancel =
            Rect { x: area.x + area.width.saturating_sub(12), y: by, width: 10, height: 1 };
        (area, field, ok, cancel)
    }

    // --- mouse ---

    pub fn on_mouse(&mut self, m: MouseEvent) -> io::Result<()> {
        self.cursor = (m.column, m.row);
        let left_down = matches!(m.kind, MouseEventKind::Down(MouseButton::Left));

        // Rename prompt is modal — only its OK/Cancel buttons respond to the mouse.
        if self.prompt.is_some() {
            if left_down {
                let (_, _, ok, cancel) = self.prompt_rects();
                if contains(ok, m.column, m.row) {
                    self.confirm_rename()?;
                } else if contains(cancel, m.column, m.row) {
                    self.prompt = None;
                    self.status = "rename cancelled".into();
                }
            }
            return Ok(());
        }

        // Kill confirmation is modal.
        if self.pending_kill.is_some() {
            if left_down {
                let (_, yes, no) = self.confirm_rects();
                if contains(yes, m.column, m.row) {
                    self.confirm_kill()?;
                } else if contains(no, m.column, m.row) {
                    self.pending_kill = None;
                    self.status = "cancelled".into();
                }
            }
            return Ok(());
        }

        // Help overlay: any left click dismisses it.
        if self.show_help {
            if left_down {
                self.show_help = false;
            }
            return Ok(());
        }

        // Top-bar buttons.
        if left_down && contains(self.help_button_rect(), m.column, m.row) {
            self.show_help = true;
            self.menu = None;
            return Ok(());
        }
        if left_down && contains(self.quit_button_rect(), m.column, m.row) {
            self.should_quit = true;
            return Ok(());
        }
        if left_down && contains(self.mode_button_rect(), m.column, m.row) {
            self.set_window_mode(!self.window_mode)?;
            return Ok(());
        }

        // An open menu captures interaction first.
        if self.menu.is_some() {
            match m.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    match self.menu_item_at(m.column, m.row) {
                        Some(idx) => self.activate_menu(idx)?,
                        None => self.menu = None,
                    }
                    return Ok(());
                }
                MouseEventKind::Down(MouseButton::Right) => self.menu = None,
                _ => return Ok(()),
            }
        }

        if self.window_mode {
            self.on_mouse_windows(m)
        } else {
            self.on_mouse_panes(m)
        }
    }

    fn on_mouse_panes(&mut self, m: MouseEvent) -> io::Result<()> {
        let rects = rects_for(&self.panes, self.last_canvas.get(), self.win_w, self.win_h);
        match m.kind {
            MouseEventKind::Down(MouseButton::Right) => {
                let target = hit(&rects, m.column, m.row).map(|i| self.panes[i].id.clone());
                self.open_menu("root", m.column, m.row, target);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                self.drag_src = hit(&rects, m.column, m.row);
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if let Some(src) = self.drag_src {
                    match hit(&rects, m.column, m.row) {
                        Some(dst) if dst != src => {
                            let s = self.panes[src].id.clone();
                            let t = self.panes[dst].id.clone();
                            tmux::swap_pane(&s, &t)?;
                            self.refresh()?;
                            self.selected = Some(s.clone());
                            self.status = format!("swapped {s} ⇄ {t}");
                        }
                        Some(same) => {
                            let id = self.panes[same].id.clone();
                            self.status = format!("selected {id}");
                            self.selected = Some(id);
                        }
                        None => self.status = "dropped outside — cancelled".into(),
                    }
                }
                self.drag_src = None;
            }
            _ => {}
        }
        Ok(())
    }

    fn on_mouse_windows(&mut self, m: MouseEvent) -> io::Result<()> {
        let tiles = self.tiles.borrow().clone();
        let mpanes = self.mpanes.borrow().clone();
        match m.kind {
            MouseEventKind::Down(MouseButton::Right) => {
                let target = tile_at(&tiles, m.column, m.row);
                self.open_menu("win", m.column, m.row, target);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // grabbing a mini-pane moves the pane; grabbing the tile frame
                // moves the whole window.
                if let Some((win, pane)) = mpane_at(&mpanes, m.column, m.row) {
                    self.wdrag = Some(WDrag::Pane { win, pane });
                } else if let Some(win) = tile_at(&tiles, m.column, m.row) {
                    self.wdrag = Some(WDrag::Win(win));
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                let drop = tile_at(&tiles, m.column, m.row);
                match self.wdrag.take() {
                    Some(WDrag::Win(src)) => match drop {
                        Some(dst) if dst != src => self.swap_windows(&src, &dst)?,
                        Some(_) => self.select_window(&src)?,
                        None => {}
                    },
                    Some(WDrag::Pane { win, pane }) => match drop {
                        Some(dst) if dst != win => self.move_pane_to_window(&pane, &dst)?,
                        Some(same) => self.select_window(&same)?,
                        None => {}
                    },
                    None => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    // --- keyboard navigation ---

    /// Switch between pane and window mode, seeding the window-mode cursor.
    pub fn set_window_mode(&mut self, on: bool) -> io::Result<()> {
        self.window_mode = on;
        self.menu = None;
        self.refresh()?;
        if on && self.sel_win.is_none() {
            self.sel_win = self.active_window_id();
        }
        self.status = if on { "window mode" } else { "pane mode" }.into();
        Ok(())
    }

    /// Columns the mission-control grid uses (matches the renderer).
    fn win_cols(&self) -> i32 {
        ((self.windows.len() as f64).sqrt().ceil() as i32).max(1)
    }

    /// Index of the focused window (selected, else active, else first).
    fn focused_window_index(&self) -> i32 {
        self.sel_win
            .as_ref()
            .and_then(|id| self.windows.iter().position(|w| &w.id == id))
            .or_else(|| self.windows.iter().position(|w| w.active))
            .unwrap_or(0) as i32
    }

    /// Pane mode: move selection directionally, via tmux's spatial logic.
    pub fn select_dir(&mut self, dir: &str) -> io::Result<()> {
        tmux::select_pane_dir(dir)?;
        self.refresh()?;
        self.selected = self.panes.iter().find(|p| p.active).map(|p| p.id.clone());
        Ok(())
    }

    /// Pane mode: swap the selected pane with its neighbour in `dir`.
    pub fn swap_dir(&mut self, dir: &str) -> io::Result<()> {
        let Some(src) = self.target() else {
            return Ok(());
        };
        tmux::select_pane_dir(dir)?;
        let (_, _, panes) = tmux::query()?;
        if let Some(nbr) = panes.iter().find(|p| p.active).map(|p| p.id.clone()) {
            if nbr != src {
                tmux::swap_pane(&src, &nbr)?;
                tmux::select_pane(&src)?;
                self.status = format!("moved {src}");
            }
        }
        self.refresh()?;
        self.selected = Some(src);
        Ok(())
    }

    /// Window mode: move the focus across the grid AND switch to that window
    /// live, so arrowing around takes you through the session.
    pub fn win_nav(&mut self, dx: i32, dy: i32) -> io::Result<()> {
        if self.windows.is_empty() {
            return Ok(());
        }
        let n = self.windows.len() as i32;
        let idx = (self.focused_window_index() + dx + dy * self.win_cols()).clamp(0, n - 1);
        let id = self.windows[idx as usize].id.clone();
        self.select_window(&id)
    }

    /// Window mode: reorder — swap the focused window with the grid neighbour.
    pub fn win_swap(&mut self, dx: i32, dy: i32) -> io::Result<()> {
        if self.windows.is_empty() {
            return Ok(());
        }
        let n = self.windows.len() as i32;
        let cur = self.focused_window_index();
        let tgt = cur + dx + dy * self.win_cols();
        if tgt < 0 || tgt >= n || tgt == cur {
            return Ok(());
        }
        let a = self.windows[cur as usize].id.clone();
        let b = self.windows[tgt as usize].id.clone();
        self.swap_windows(&a, &b)?;
        self.sel_win = Some(a);
        Ok(())
    }

    /// Dispatch a key in pane mode (arrows navigate, Shift+arrows move).
    pub fn pane_key(&mut self, k: KeyEvent) -> io::Result<()> {
        let shift = k.modifiers.contains(KeyModifiers::SHIFT);
        match k.code {
            KeyCode::Left => self.move_or_select("-L", shift)?,
            KeyCode::Right => self.move_or_select("-R", shift)?,
            KeyCode::Up => self.move_or_select("-U", shift)?,
            KeyCode::Down => self.move_or_select("-D", shift)?,
            KeyCode::Char('|') => self.split(true)?,
            KeyCode::Char('-') => self.split(false)?,
            KeyCode::Char('x') | KeyCode::Char('d') => self.request_kill(),
            KeyCode::Char('1') => self.set_layout("even-horizontal", "side-by-side")?,
            KeyCode::Char('2') => self.set_layout("even-vertical", "stacked")?,
            KeyCode::Char('3') => self.set_layout("main-vertical", "main-left")?,
            KeyCode::Char('4') => self.set_layout("main-horizontal", "main-top")?,
            KeyCode::Char('5') => self.set_layout("tiled", "tiled")?,
            KeyCode::Char(' ') => self.cycle_layout()?,
            _ => {}
        }
        Ok(())
    }

    fn move_or_select(&mut self, dir: &str, shift: bool) -> io::Result<()> {
        if shift {
            self.swap_dir(dir)
        } else {
            self.select_dir(dir)
        }
    }

    /// Dispatch a key in window mode (arrows navigate, Shift+arrows reorder).
    pub fn window_key(&mut self, k: KeyEvent) -> io::Result<()> {
        let shift = k.modifiers.contains(KeyModifiers::SHIFT);
        let (dx, dy) = match k.code {
            KeyCode::Left => (-1, 0),
            KeyCode::Right => (1, 0),
            KeyCode::Up => (0, -1),
            KeyCode::Down => (0, 1),
            KeyCode::Enter => {
                // Land in the focused window and close the manager.
                if let Some(id) = self.sel_win.clone() {
                    self.select_window(&id)?;
                }
                self.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('n') => return self.new_window(),
            KeyCode::Char('x') | KeyCode::Char('d') => {
                if let Some(id) = self.sel_win.clone().or_else(|| self.active_window_id()) {
                    self.request_kill_window(id);
                }
                return Ok(());
            }
            _ => return Ok(()),
        };
        if shift {
            self.win_swap(dx, dy)?;
        } else {
            self.win_nav(dx, dy)?;
        }
        Ok(())
    }
}

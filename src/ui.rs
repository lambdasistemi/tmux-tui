//! Rendering. All drawing is implemented as methods on [`App`] so it can read
//! state directly; no mutation happens here.

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::{App, WDrag};
use crate::geometry::{contains, hit, map_rect, rects_for};

impl App {
    pub fn draw(&self, f: &mut Frame) {
        let full = f.area();
        self.last_full.set(full);
        // Reserve row 0 for the top bar and the bottom two rows for the
        // status + keys lines, so the canvas never sits under the chrome.
        let canvas = Rect {
            x: full.x,
            y: full.y + 1,
            width: full.width,
            height: full.height.saturating_sub(4),
        };
        self.last_canvas.set(canvas);

        f.render_widget(Block::default().style(Style::default().bg(Color::Black)), full);

        if self.window_mode {
            self.draw_windows(f, canvas);
        } else {
            self.draw_panes(f, canvas);
        }

        self.draw_chrome(f, full);
    }

    fn draw_panes(&self, f: &mut Frame, canvas: Rect) {
        let rects = rects_for(&self.panes, canvas, self.win_w, self.win_h);
        let hover = self.drag_src.and_then(|_| hit(&rects, self.cursor.0, self.cursor.1));
        let sel_idx = self.selected_index();

        for (i, r) in &rects {
            let p = &self.panes[*i];
            let is_src = Some(*i) == self.drag_src;
            let is_hover = Some(*i) == hover && Some(*i) != self.drag_src;
            let is_sel = Some(*i) == sel_idx;
            let is_kill =
                self.pending_kill.as_ref().is_some_and(|(id, is_win)| !*is_win && id == &p.id);

            let border_style = if is_kill {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else if is_src {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if is_hover {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if is_sel {
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
            } else if p.active {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(format!(" {}:{} ", p.index, p.cmd));
            if is_hover {
                block = block.style(Style::default().bg(Color::Rgb(20, 60, 20)));
            }
            f.render_widget(block, *r);

            if r.height >= 3 && r.width >= 6 {
                let label = if is_kill {
                    "kill?"
                } else if is_src {
                    "↕ moving"
                } else if is_hover {
                    "drop here"
                } else if is_sel {
                    "● selected"
                } else {
                    p.id.as_str()
                };
                let inner = Rect {
                    x: r.x + 1,
                    y: r.y + r.height / 2,
                    width: r.width.saturating_sub(2),
                    height: 1,
                };
                f.render_widget(Paragraph::new(label).alignment(Alignment::Center), inner);
            }
        }
    }

    fn draw_windows(&self, f: &mut Frame, canvas: Rect) {
        let mut tiles = self.tiles.borrow_mut();
        let mut mpanes = self.mpanes.borrow_mut();
        tiles.clear();
        mpanes.clear();

        let n = self.windows.len() as u16;
        if n == 0 {
            return;
        }
        let cols = ((n as f32).sqrt().ceil() as u16).max(1);
        let rows = n.div_ceil(cols);
        let cell_w = (canvas.width / cols).max(3);
        let cell_h = (canvas.height / rows.max(1)).max(3);

        // tile rectangles first, so the hover target can be computed.
        let mut tr: Vec<Rect> = Vec::with_capacity(self.windows.len());
        for i in 0..self.windows.len() {
            let ci = i as u16 % cols;
            let ri = i as u16 / cols;
            tr.push(Rect {
                x: canvas.x + ci * cell_w + 1,
                y: canvas.y + ri * cell_h,
                width: cell_w.saturating_sub(2).max(2),
                height: cell_h.saturating_sub(1).max(2),
            });
        }

        let drag_win = match &self.wdrag {
            Some(WDrag::Win(id)) => Some(id.clone()),
            _ => None,
        };
        let drag_pane = match &self.wdrag {
            Some(WDrag::Pane { pane, .. }) => Some(pane.clone()),
            _ => None,
        };
        let hover_win = if self.wdrag.is_some() {
            tr.iter()
                .position(|r| contains(*r, self.cursor.0, self.cursor.1))
                .map(|i| self.windows[i].id.clone())
        } else {
            None
        };

        for (i, tile) in tr.iter().enumerate() {
            let w = &self.windows[i];
            let is_drag = drag_win.as_deref() == Some(w.id.as_str());
            let is_hover = hover_win.as_deref() == Some(w.id.as_str()) && !is_drag;
            let is_sel = self.sel_win.as_deref() == Some(w.id.as_str());
            let border = if is_drag {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if is_hover {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if is_sel {
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
            } else if w.active {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_style(border)
                .title(format!(" {}:{} ", w.index, w.name));
            if is_hover {
                block = block.style(Style::default().bg(Color::Rgb(20, 60, 20)));
            }
            f.render_widget(block, *tile);
            tiles.push((w.id.clone(), *tile));

            let inner = Rect {
                x: tile.x + 1,
                y: tile.y + 1,
                width: tile.width.saturating_sub(2),
                height: tile.height.saturating_sub(2),
            };
            if inner.width < 2 || inner.height < 1 {
                continue;
            }
            for p in &w.panes {
                let mr = map_rect(p, inner, w.w, w.h);
                let pstyle = if drag_pane.as_deref() == Some(p.id.as_str()) {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if p.active {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                f.render_widget(Block::default().borders(Borders::ALL).border_style(pstyle), mr);
                mpanes.push((w.id.clone(), p.id.clone(), mr));
            }
        }
    }

    fn draw_chrome(&self, f: &mut Frame, full: Rect) {
        // status line
        f.render_widget(
            Paragraph::new(format!("  {}", self.status))
                .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Rect { x: full.x, y: full.bottom().saturating_sub(2), width: full.width, height: 1 },
        );

        // keys line (mode-dependent)
        let keys = if self.window_mode {
            "  arrows = navigate+switch · Enter = enter window · drag/⇧arrows = reorder · drag pane → window = move · right-click/R = rename · n · x   p panes · ?"
        } else {
            "  drag swap · arrows select · ⇧arrows move · |/- split · x kill · R rename   layout 1/2/3/4/5 · ␣ cycle   w windows · ? help · q quit"
        };
        f.render_widget(
            Paragraph::new(keys).style(Style::default().fg(Color::Gray)),
            Rect { x: full.x, y: full.bottom().saturating_sub(1), width: full.width, height: 1 },
        );

        // top bar: a reserved row carrying the buttons plus the current
        // window name (centered), so nothing overlaps the canvas below.
        let bar = Rect { x: full.x, y: full.y, width: full.width, height: 1 };
        f.render_widget(Block::default().style(Style::default().bg(Color::Rgb(30, 30, 30))), bar);
        if let Some(label) = self.active_window_label() {
            f.render_widget(
                Paragraph::new(label)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                bar,
            );
        }

        // top-bar buttons (drawn over the bar)
        f.render_widget(
            Paragraph::new("[ quit ]").style(
                Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            self.quit_button_rect(),
        );
        let mode_label = if self.window_mode { "[ panes ]  " } else { "[ windows ]" };
        f.render_widget(
            Paragraph::new(mode_label).style(
                Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD),
            ),
            self.mode_button_rect(),
        );
        f.render_widget(
            Paragraph::new("[ ? help ]").style(
                Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            self.help_button_rect(),
        );

        self.draw_menu(f);
        self.draw_help(f);
        self.draw_confirm(f);
        self.draw_prompt(f);
    }

    fn draw_menu(&self, f: &mut Frame) {
        let Some(menu) = &self.menu else { return };
        let area = self.menu_rect(menu);
        f.render_widget(Clear, area);
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title(format!(" {} ", menu.title)),
            area,
        );
        for (i, (label, _)) in menu.items.iter().enumerate() {
            if (i as u16) + 1 >= area.height.saturating_sub(1) {
                break;
            }
            let row = Rect {
                x: area.x + 1,
                y: area.y + 1 + i as u16,
                width: area.width.saturating_sub(2),
                height: 1,
            };
            let style = if i == menu.highlight {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };
            f.render_widget(Paragraph::new(format!(" {label}")).style(style), row);
        }
    }

    fn draw_help(&self, f: &mut Frame) {
        if !self.show_help {
            return;
        }
        let lines = [
            "tmux-tui — tmux layout manager        w panes/windows · Tab toggle",
            "",
            "Pane mode",
            "  drag a box / click          swap / select a pane",
            "  arrows / Shift+arrows       select neighbour / move pane",
            "  | -                         split left-right / top-bottom",
            "  x                           kill selected pane",
            "  1 2 3 4 5 / space           layout presets / cycle",
            "  R                           rename the current window",
            "  right-click                 New pane ▸ / Kill / Layout / Rename",
            "",
            "Window mode  (w, or [ windows ])",
            "  arrows                      navigate + switch window live",
            "  Enter                       enter the focused window (closes)",
            "  drag window / Shift+arrows  reorder windows",
            "  drag a pane into another    move pane across windows",
            "  n / x / R                   new / close / rename window",
            "  right-click                 Rename / New / Close window",
            "",
            "[ quit ] closes · drops are schematic, not live panes (tmux#3503)",
            "press any key or click to close",
        ];
        let full = self.last_full.get();
        let w = (lines.iter().map(|l| l.chars().count()).max().unwrap_or(40) as u16 + 4)
            .min(full.width.max(1));
        let h = (lines.len() as u16 + 2).min(full.height.max(1));
        let area = Rect {
            x: full.x + full.width.saturating_sub(w) / 2,
            y: full.y + full.height.saturating_sub(h) / 2,
            width: w,
            height: h,
        };
        f.render_widget(Clear, area);
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" help "),
            area,
        );
        for (i, line) in lines.iter().enumerate() {
            if (i as u16) + 1 >= area.height.saturating_sub(1) {
                break;
            }
            let row = Rect {
                x: area.x + 2,
                y: area.y + 1 + i as u16,
                width: area.width.saturating_sub(3),
                height: 1,
            };
            f.render_widget(Paragraph::new(*line).style(Style::default().fg(Color::Gray)), row);
        }
    }

    fn draw_confirm(&self, f: &mut Frame) {
        let Some((id, is_win)) = &self.pending_kill else {
            return;
        };
        let (area, yes, no) = self.confirm_rects();
        let what = if *is_win { "window" } else { "pane" };
        f.render_widget(Clear, area);
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title(" confirm "),
            area,
        );
        f.render_widget(
            Paragraph::new(format!("Close {what} {id} ?"))
                .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Rect { x: area.x + 2, y: area.y + 1, width: area.width.saturating_sub(3), height: 1 },
        );
        f.render_widget(
            Paragraph::new("[ Yes ]").style(
                Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            yes,
        );
        f.render_widget(
            Paragraph::new("[ No ]").style(
                Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD),
            ),
            no,
        );
    }

    fn draw_prompt(&self, f: &mut Frame) {
        let Some(p) = &self.prompt else {
            return;
        };
        let (area, field, ok, cancel) = self.prompt_rects();
        f.render_widget(Clear, area);
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" rename window "),
            area,
        );
        f.render_widget(
            Paragraph::new(format!("window {}", p.win)).style(Style::default().fg(Color::DarkGray)),
            Rect { x: area.x + 2, y: area.y + 1, width: area.width.saturating_sub(3), height: 1 },
        );
        f.render_widget(
            Paragraph::new(format!("{}▏", p.buf))
                .style(Style::default().fg(Color::White).bg(Color::Rgb(40, 40, 40))),
            field,
        );
        f.render_widget(
            Paragraph::new("[ OK ]").style(
                Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            ok,
        );
        f.render_widget(
            Paragraph::new("[Cancel]").style(
                Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD),
            ),
            cancel,
        );
    }
}

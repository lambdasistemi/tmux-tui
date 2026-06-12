//! Context-menu model and the menu-item builder.

/// What a menu entry does when activated. Pane actions act on the menu's
/// target pane; window actions on its target window.
#[derive(Clone)]
pub enum Action {
    Kill,
    Layout(&'static str, &'static str),
    NewPane(&'static str, bool), // (split flag "-h"/"-v", before = left/above)
    Submenu(&'static str),       // open a nested menu by name
    RenameWindow,
    NewWindow,
    KillWindow,
}

/// A right-click context menu anchored at (x, y), acting on `target`.
pub struct Menu {
    pub x: u16,
    pub y: u16,
    pub target: Option<String>,
    pub title: String,
    pub items: Vec<(String, Action)>,
    pub highlight: usize,
}

/// Build `(title, items)` for a menu screen by name. `n_panes` / `n_windows`
/// gate destructive entries so we never offer to kill the last one.
pub fn build_menu_items(
    name: &str,
    target: &Option<String>,
    n_panes: usize,
    n_windows: usize,
) -> (String, Vec<(String, Action)>) {
    match name {
        "newpane" => (
            "New pane".into(),
            vec![
                ("Right".into(), Action::NewPane("-h", false)),
                ("Left".into(), Action::NewPane("-h", true)),
                ("Above".into(), Action::NewPane("-v", true)),
                ("Below".into(), Action::NewPane("-v", false)),
                ("‹ back".into(), Action::Submenu("root")),
            ],
        ),
        "win" => {
            let mut items: Vec<(String, Action)> = Vec::new();
            if target.is_some() {
                items.push(("Rename window".into(), Action::RenameWindow));
            }
            items.push(("New window".into(), Action::NewWindow));
            if target.is_some() && n_windows > 1 {
                items.push(("Close window".into(), Action::KillWindow));
            }
            ("window".into(), items)
        }
        _ => {
            let mut items: Vec<(String, Action)> = Vec::new();
            items.push(("New pane          ▸".into(), Action::Submenu("newpane")));
            if target.is_some() && n_panes > 1 {
                items.push(("Kill pane".into(), Action::Kill));
            }
            items.push((
                "Layout: side-by-side".into(),
                Action::Layout("even-horizontal", "side-by-side"),
            ));
            items.push(("Layout: stacked".into(), Action::Layout("even-vertical", "stacked")));
            items.push(("Layout: tiled".into(), Action::Layout("tiled", "tiled")));
            items.push(("Rename window".into(), Action::RenameWindow));
            ("actions".into(), items)
        }
    }
}

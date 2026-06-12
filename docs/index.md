# tmux-tui

A mouse-driven, drag-and-drop layout manager for [tmux](https://github.com/tmux/tmux),
built with [ratatui](https://ratatui.rs). Pop it over your session, rearrange
panes and windows by dragging, and close it — your real tmux layout follows.

!!! note "You drag a schematic, not live panes"
    tmux-tui draws boxes that mirror your layout and translates each gesture
    into the matching tmux command (`swap-pane`, `select-layout`,
    `split-window`, `join-pane`, `swap-window`, `kill-pane`, `rename-window`).
    True dragging of *live* pane contents is something tmux core itself can't do
    yet — see [tmux#3503](https://github.com/tmux/tmux/issues/3503), where the
    maintainer tried it, couldn't make it work, and parked it on a todo list.

## Two views

- **Pane mode** — the current window's panes as a scaled schematic. Drag to
  swap, click to select, split/kill/relayout, all by mouse or keyboard.
- **Window mode** (mission control) — every window as a tile drawn with its own
  mini pane-layout. Drag windows to reorder, drag a pane into another window to
  relocate it, click to switch.

See **[Usage](usage.md)** for the full set of gestures and keybindings, and the
**[API Reference](api-reference.md)** for the generated rustdoc.

## Quick start

Bind it in your tmux config and launch it from a popup:

```tmux
# Ctrl + right-click on any pane, or prefix + g
bind-key -n C-MouseDown3Pane display-popup -E -w 30% -h 30% tmux-tui
bind-key g                   display-popup -E -w 30% -h 30% tmux-tui
```

Build the binary with Nix:

```sh
nix build github:lambdasistemi/tmux-tui   # ./result/bin/tmux-tui
```

# Changelog

## 0.1.0 (2026-06-12)


### Features

* mouse-driven tmux pane and window layout manager ([c6a79c0](https://github.com/lambdasistemi/tmux-tui/commit/c6a79c0dfd40b5b75ccabd9dbe2a9a0311723408))

## 0.1.0

Initial release.

- Pane mode: drag-to-swap, click-to-select, split (`|`/`-`), kill (with
  confirm), layout presets (`1`–`5`/space), right-click context menu.
- Window mode (mission control): every window drawn as a tile with its own
  mini pane-layout; drag windows to reorder, drag a pane into another
  window to relocate it, click to switch, right-click to rename / create /
  close.
- Mouse-only operation throughout (quit / help / confirm / rename are all
  clickable), launched from a `tmux display-popup`.

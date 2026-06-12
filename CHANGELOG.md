# Changelog

## [0.2.0](https://github.com/lambdasistemi/tmux-tui/compare/v0.1.0...v0.2.0) (2026-06-12)


### Features

* NixOS and Home Manager modules plus overlay ([#5](https://github.com/lambdasistemi/tmux-tui/issues/5)) ([a4fbfbf](https://github.com/lambdasistemi/tmux-tui/commit/a4fbfbf90f3b66b1f1a6826875911be74c1388f9))


### Bug Fixes

* **release:** unblock tag-publish (awk-free notes + tap-token guard) ([#7](https://github.com/lambdasistemi/tmux-tui/issues/7)) ([151de9a](https://github.com/lambdasistemi/tmux-tui/commit/151de9a8b6a1e5d15c4d93e56c45d2f459f68fc6))

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

# tmux-tui

A mouse-driven, drag-and-drop layout manager for [tmux](https://github.com/tmux/tmux),
built with [ratatui](https://ratatui.rs). Pop it over your session, rearrange
panes and windows by dragging, and close it — your real tmux layout follows.

> **You drag a schematic, not live panes.** tmux-tui draws boxes that mirror
> your layout and translates each gesture into the matching tmux command
> (`swap-pane`, `select-layout`, `split-window`, `join-pane`, `swap-window`,
> `kill-pane`, `rename-window`). True dragging of *live* pane contents is
> something tmux core itself can't do yet — see
> [tmux#3503](https://github.com/tmux/tmux/issues/3503), where the maintainer
> tried it, couldn't make it work, and parked it on a todo list.

📖 **Documentation:** <https://lambdasistemi.github.io/tmux-tui/>

## Two views

**Pane mode** — the current window's panes as a scaled schematic:

| Gesture / key | Action |
|---|---|
| drag a box onto another | swap the two panes |
| click a box | select it |
| `\|` / `-` | split the selected pane (left-right / top-bottom) |
| `x` | kill the selected pane (click **Yes/No** or `y`/`n`) |
| `1` `2` `3` `4` `5` | layout: side-by-side · stacked · main-left · main-top · tiled |
| `space` | cycle layouts |
| `R` | rename the current window |
| right-click | context menu (New pane ▸, Kill, Layout, Rename) |

**Window mode** (`Tab`, or the `[ windows ]` button) — mission control over every
window, each tile drawn with its own mini pane-layout:

| Gesture / key | Action |
|---|---|
| drag one window onto another | reorder (`swap-window`) |
| drag a pane into another window | move it there (`join-pane`) |
| click a window | switch to it |
| right-click | Rename / New / Close window |
| `n` / `x` / `R` | new / close / rename window |

The top-bar buttons — `[ quit ]`, `[ windows ]`/`[ panes ]`, `[ ? help ]` — and
the kill/rename dialogs are all clickable, so the whole tool is operable with
the mouse alone.

## Install & run

tmux-tui is launched from a tmux popup. Bind it in your tmux config:

```tmux
# Ctrl + right-click on any pane, or prefix + g
bind-key -n C-MouseDown3Pane display-popup -E -w 30% -h 30% tmux-tui
bind-key g                   display-popup -E -w 30% -h 30% tmux-tui
```

Build the binary with Nix (the flake provides the toolchain):

```sh
nix build github:lambdasistemi/tmux-tui     # ./result/bin/tmux-tui
# or, from a clone:
nix develop -c cargo build --release        # ./target/release/tmux-tui
```

## Development

`nix flake check` is the single gate — the same command passes locally and in
CI (clippy, rustfmt, nextest, cargo-deny, rustdoc), with no environment drift.

```sh
just            # = just ci = nix flake check
just build      # build the CLI
just test       # cargo-nextest
just clippy     # clippy -D warnings
just fmt        # format in place
```

## License

[Apache-2.0](LICENSE).

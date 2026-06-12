# Usage

tmux-tui runs inside a `tmux display-popup`, drawing a schematic of your layout
that you manipulate with the mouse or keyboard. Switch views with **`w`**
(windows) / **`p`** (panes), or **`Tab`** to toggle. The top bar shows the
current window and carries the `[ quit ]`, `[ windows ]`/`[ panes ]` and
`[ ? help ]` buttons; press **`?`** for an in-app cheat sheet.

## Launching

```tmux
# Ctrl + right-click on any pane, or prefix + g
bind-key -n C-MouseDown3Pane display-popup -E -w 30% -h 30% tmux-tui
bind-key g                   display-popup -E -w 30% -h 30% tmux-tui
```

Without a binding, from the tmux command prompt (`prefix :`):

```
display-popup -E -w 30% -h 30% tmux-tui
```

## Pane mode

| Gesture / key | Action |
|---|---|
| drag a box onto another | swap the two panes |
| click a box | select it |
| arrow keys | move the selection to the neighbouring pane |
| **Shift** + arrows | move the selected pane (swap with that neighbour) |
| `\|` / `-` | split the selected pane (left-right / top-bottom) |
| `x` | kill the selected pane (click **Yes/No** or `y`/`n`) |
| `1` `2` `3` `4` `5` | layout: side-by-side · stacked · main-left · main-top · tiled |
| `space` | cycle layouts |
| `R` | rename the current window |
| right-click | context menu: New pane ▸, Kill, Layout, Rename |

Layout keys re-arrange panes **without swapping their contents** — that's
`tmux select-layout`, distinct from the drag-to-swap gesture.

## Window mode (mission control)

Every window is drawn as a tile containing its own mini pane-layout.

| Gesture / key | Action |
|---|---|
| arrow keys | move the focus across the grid, switching window live |
| `Enter` | enter the focused window and close tmux-tui |
| click a window | switch to it |
| drag one window onto another | reorder (`swap-window`) |
| **Shift** + arrows | reorder the focused window across the grid |
| drag a pane out of a tile into another | move that pane across windows (`join-pane`) |
| `n` / `x` / `R` | new / close / rename window |
| right-click a tile | Rename / New / Close window |

## Notes

- **Mouse-only is fully supported:** the buttons and the kill/rename dialogs are
  all clickable, so you never need the keyboard if you don't want it.
- **Drops are schematic.** You drag boxes that represent panes/windows; tmux-tui
  issues the corresponding tmux command on drop. See the note on
  [the home page](index.md) about why live-pane dragging isn't possible.

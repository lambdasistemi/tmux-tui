#!/usr/bin/env bash
# Launch tmux-tui in a tmux popup over the current session.
set -euo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec tmux display-popup -E -w 30% -h 30% "$DIR/target/release/tmux-tui"

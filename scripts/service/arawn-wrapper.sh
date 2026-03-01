#!/usr/bin/env bash
# Wrapper script for macOS launchd.
# Sources the environment file then execs arawn.
# Installed to ~/.config/arawn/arawn-wrapper.sh by setup.sh.

set -euo pipefail

ENV_FILE="$HOME/.config/arawn/env"

if [ -f "$ENV_FILE" ]; then
    set -a
    # shellcheck source=/dev/null
    . "$ENV_FILE"
    set +a
fi

exec "$HOME/.local/bin/arawn" start

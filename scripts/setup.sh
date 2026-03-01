#!/usr/bin/env bash
# Arawn host provisioning script.
# Sets up directories, config, and service files for running arawn as a service.
# Idempotent — safe to re-run.
#
# Usage:
#   bash scripts/setup.sh              # Full setup
#   bash scripts/setup.sh --no-service # Skip service installation
#   bash scripts/setup.sh --no-config  # Skip config generation
#   bash scripts/setup.sh --uninstall  # Remove service and optionally data

set -euo pipefail

# --- Defaults ---
INSTALL_SERVICE=true
INSTALL_CONFIG=true
UNINSTALL=false

ARAWN_BIN="$HOME/.local/bin/arawn"
CONFIG_DIR="$HOME/.config/arawn"
CONFIG_FILE="$CONFIG_DIR/config.toml"
ENV_FILE="$CONFIG_DIR/env"
WRAPPER_FILE="$CONFIG_DIR/arawn-wrapper.sh"
LOG_DIR="$CONFIG_DIR/logs"
WORKFLOW_DIR="$CONFIG_DIR/workflows"
PLUGIN_DIR="$CONFIG_DIR/plugins"

PLIST_LABEL="io.colliery.arawn"
PLIST_DEST="$HOME/Library/LaunchAgents/$PLIST_LABEL.plist"
SYSTEMD_DIR="$HOME/.config/systemd/user"
SYSTEMD_DEST="$SYSTEMD_DIR/arawn.service"

# Resolve the directory where this script lives (for locating service templates)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE_DIR="$SCRIPT_DIR/service"

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

info()  { echo -e "${BLUE}==>${NC} ${BOLD}$*${NC}"; }
ok()    { echo -e "${GREEN}  ✓${NC} $*"; }
warn()  { echo -e "${YELLOW}  !${NC} $*"; }
err()   { echo -e "${RED}  ✗${NC} $*" >&2; }

# --- Parse arguments ---
for arg in "$@"; do
    case "$arg" in
        --no-service) INSTALL_SERVICE=false ;;
        --no-config)  INSTALL_CONFIG=false ;;
        --uninstall)  UNINSTALL=true ;;
        -h|--help)
            echo "Usage: bash scripts/setup.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --no-service   Skip service file installation"
            echo "  --no-config    Skip config generation"
            echo "  --uninstall    Stop service, remove service files, optionally remove data"
            echo "  -h, --help     Show this help message"
            exit 0
            ;;
        *)
            err "Unknown argument: $arg"
            exit 1
            ;;
    esac
done

# --- Detect platform ---
detect_platform() {
    OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
    ARCH="$(uname -m)"

    case "$OS" in
        linux)  OS="linux" ;;
        darwin) OS="darwin" ;;
        *)
            err "Unsupported OS: $OS"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *)
            err "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac
}

# --- Uninstall ---
do_uninstall() {
    info "Uninstalling arawn service..."

    if [ "$OS" = "darwin" ]; then
        if [ -f "$PLIST_DEST" ]; then
            launchctl bootout "gui/$(id -u)/$PLIST_LABEL" 2>/dev/null || true
            rm -f "$PLIST_DEST"
            ok "Removed launchd plist"
        else
            warn "No launchd plist found"
        fi
    else
        if [ -f "$SYSTEMD_DEST" ]; then
            systemctl --user stop arawn 2>/dev/null || true
            systemctl --user disable arawn 2>/dev/null || true
            rm -f "$SYSTEMD_DEST"
            systemctl --user daemon-reload 2>/dev/null || true
            ok "Removed systemd unit"
        else
            warn "No systemd unit found"
        fi
    fi

    if [ -f "$WRAPPER_FILE" ]; then
        rm -f "$WRAPPER_FILE"
        ok "Removed wrapper script"
    fi

    echo ""
    read -rp "Also remove config and data (~/.config/arawn/)? [y/N] " remove_data
    if [[ "$remove_data" =~ ^[Yy] ]]; then
        rm -rf "$CONFIG_DIR"
        ok "Removed $CONFIG_DIR"
    else
        warn "Kept $CONFIG_DIR"
    fi

    echo ""
    info "Uninstall complete."
    echo "  The arawn binary at $ARAWN_BIN was not removed."
    echo "  To remove it: rm $ARAWN_BIN"
    exit 0
}

# --- Main ---
detect_platform
echo ""
info "Arawn setup — $OS/$ARCH"
echo ""

if $UNINSTALL; then
    do_uninstall
fi

# Step 1: Verify binary
info "Checking for arawn binary..."
if [ -x "$ARAWN_BIN" ]; then
    ok "Found $ARAWN_BIN"
else
    warn "arawn binary not found at $ARAWN_BIN"
    if [ -f "$SCRIPT_DIR/install.sh" ]; then
        read -rp "  Run scripts/install.sh to download it? [Y/n] " run_install
        if [[ ! "$run_install" =~ ^[Nn] ]]; then
            bash "$SCRIPT_DIR/install.sh"
            if [ ! -x "$ARAWN_BIN" ]; then
                err "Installation did not produce $ARAWN_BIN"
                exit 1
            fi
            ok "arawn installed"
        else
            err "Cannot continue without arawn binary."
            exit 1
        fi
    else
        err "install.sh not found. Please install arawn first."
        exit 1
    fi
fi

# Step 2: Create directories
info "Creating data directories..."
for dir in "$CONFIG_DIR" "$LOG_DIR" "$WORKFLOW_DIR" "$PLUGIN_DIR"; do
    if [ -d "$dir" ]; then
        ok "$dir (exists)"
    else
        mkdir -p "$dir"
        ok "$dir"
    fi
done

# Step 3: Config file
if $INSTALL_CONFIG; then
    info "Setting up configuration..."
    if [ -f "$CONFIG_FILE" ]; then
        ok "$CONFIG_FILE already exists — skipping"
    else
        echo ""
        echo "  Choose your LLM backend:"
        echo "    1) groq (default)"
        echo "    2) anthropic"
        echo "    3) openai"
        echo ""
        read -rp "  Backend [1]: " backend_choice
        case "$backend_choice" in
            2) BACKEND="anthropic"; MODEL="claude-sonnet-4-20250514" ;;
            3) BACKEND="openai";    MODEL="gpt-4o" ;;
            *)  BACKEND="groq";     MODEL="openai/gpt-oss-20b" ;;
        esac

        read -rp "  Model name [$MODEL]: " custom_model
        MODEL="${custom_model:-$MODEL}"

        read -rp "  Server port [8080]: " custom_port
        PORT="${custom_port:-8080}"

        cat > "$CONFIG_FILE" <<EOF
[llm]
backend = "$BACKEND"
model = "$MODEL"
max_context_tokens = 131072

[agent.default]
max_tokens = 65536

[server]
port = $PORT
bind = "127.0.0.1"
EOF
        ok "Generated $CONFIG_FILE"
    fi
else
    warn "Skipping config generation (--no-config)"
fi

# Step 4: Environment file
info "Setting up environment file..."
if [ -f "$ENV_FILE" ]; then
    ok "$ENV_FILE already exists — skipping"
else
    cp "$SERVICE_DIR/arawn.env" "$ENV_FILE"
    ok "Created $ENV_FILE — edit this to add your API key"
fi

# Step 5: Service installation
if $INSTALL_SERVICE; then
    info "Installing service..."

    if [ "$OS" = "darwin" ]; then
        # Install wrapper script
        cp "$SERVICE_DIR/arawn-wrapper.sh" "$WRAPPER_FILE"
        chmod +x "$WRAPPER_FILE"
        ok "Installed wrapper script to $WRAPPER_FILE"

        # Update plist log paths to use config dir
        mkdir -p "$HOME/Library/LaunchAgents"
        sed \
            -e "s|/tmp/arawn-launchd-stdout.log|$LOG_DIR/launchd-stdout.log|g" \
            -e "s|/tmp/arawn-launchd-stderr.log|$LOG_DIR/launchd-stderr.log|g" \
            -e "s|<string>/tmp</string>|<string>$HOME</string>|g" \
            "$SERVICE_DIR/io.colliery.arawn.plist" > "$PLIST_DEST"
        ok "Installed plist to $PLIST_DEST"

        echo ""
        read -rp "  Load the service now with launchctl? [Y/n] " load_now
        if [[ ! "$load_now" =~ ^[Nn] ]]; then
            launchctl bootout "gui/$(id -u)/$PLIST_LABEL" 2>/dev/null || true
            launchctl bootstrap "gui/$(id -u)" "$PLIST_DEST"
            ok "Service loaded"
        else
            warn "Skipped — load later with: launchctl bootstrap gui/\$(id -u) $PLIST_DEST"
        fi
    else
        # Linux / systemd
        mkdir -p "$SYSTEMD_DIR"
        cp "$SERVICE_DIR/arawn.service" "$SYSTEMD_DEST"
        ok "Installed unit to $SYSTEMD_DEST"
        systemctl --user daemon-reload
        ok "Reloaded systemd user daemon"

        echo ""
        read -rp "  Enable and start the service now? [Y/n] " enable_now
        if [[ ! "$enable_now" =~ ^[Nn] ]]; then
            systemctl --user enable --now arawn
            ok "Service enabled and started"
        else
            warn "Skipped — start later with: systemctl --user enable --now arawn"
        fi
    fi
else
    warn "Skipping service installation (--no-service)"
fi

# Step 6: Summary
echo ""
info "Setup complete!"
echo ""
echo "  Config:  $CONFIG_FILE"
echo "  Env:     $ENV_FILE"
echo "  Logs:    $LOG_DIR/"
echo ""
echo "  ${BOLD}Next steps:${NC}"
echo "  1. Edit $ENV_FILE and add your API key"
if [ "$OS" = "darwin" ]; then
    echo "  2. Check service status: launchctl print gui/\$(id -u)/$PLIST_LABEL"
    echo "  3. View logs: tail -f $LOG_DIR/launchd-stdout.log"
    echo "  4. Restart: launchctl kickstart -k gui/\$(id -u)/$PLIST_LABEL"
else
    echo "  2. Check service status: systemctl --user status arawn"
    echo "  3. View logs: journalctl --user -u arawn -f"
    echo "  4. Restart: systemctl --user restart arawn"
fi
echo ""

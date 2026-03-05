#!/usr/bin/env bash
# Arawn installer
# Usage: curl -fsSL https://raw.githubusercontent.com/colliery-io/arawn/main/scripts/install.sh | bash
#
# Downloads the arawn binary, creates config directories, installs
# the system service, and tells you what to do next.

set -euo pipefail

REPO="colliery-io/arawn"
INSTALL_DIR="${HOME}/.local/bin"
VERSION="latest"

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

# ─────────────────────────────────────────────────────────────────────────────
# Output
# ─────────────────────────────────────────────────────────────────────────────

if [ -t 1 ]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'
    BLUE='\033[0;34m'; CYAN='\033[0;36m'; BOLD='\033[1m'
    DIM='\033[2m'; RESET='\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; BLUE=''; CYAN=''; BOLD=''; DIM=''; RESET=''
fi

banner()  { printf "\n${BOLD}${BLUE}━━━ %s ━━━${RESET}\n\n" "$1"; }
step()    { printf "${BOLD}${CYAN}[%s/%s]${RESET} ${BOLD}%s${RESET}\n" "$1" "$TOTAL_STEPS" "$2"; }
info()    { printf "     ${BLUE}info${RESET}  %s\n" "$1"; }
ok()      { printf "     ${GREEN} ok ${RESET}  %s\n" "$1"; }
warn()    { printf "     ${YELLOW}warn${RESET}  %s\n" "$1"; }
err()     { printf "     ${RED}fail${RESET}  %s\n" "$1" >&2; }
detail()  { printf "     ${DIM}      %s${RESET}\n" "$1"; }

# ─────────────────────────────────────────────────────────────────────────────
# Cleanup
# ─────────────────────────────────────────────────────────────────────────────

TMPDIR_INSTALL=""
cleanup() {
    if [ -n "${TMPDIR_INSTALL}" ] && [ -d "${TMPDIR_INSTALL}" ]; then
        rm -rf "${TMPDIR_INSTALL}"
    fi
    return 0
}
trap cleanup EXIT

# ─────────────────────────────────────────────────────────────────────────────
# Arguments
# ─────────────────────────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
    case "$1" in
        --help)
            cat <<EOF
${BOLD}Arawn Installer${RESET}

Downloads arawn, sets up directories and service files, then tells
you what to configure.

${BOLD}USAGE${RESET}
    curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | bash
    ./scripts/install.sh [OPTIONS]

${BOLD}OPTIONS${RESET}
    --help              Show this help message
    --install-dir DIR   Install directory (default: ${INSTALL_DIR})
    --version VER       Install a specific version tag (default: latest)
    --skip-deps         Skip sandbox dependency installation (Linux)
    --skip-service      Skip service file installation

EOF
            exit 0 ;;
        --install-dir)
            [ $# -lt 2 ] && { err "--install-dir requires an argument"; exit 1; }
            INSTALL_DIR="$2"; shift 2 ;;
        --version)
            [ $# -lt 2 ] && { err "--version requires an argument"; exit 1; }
            VERSION="$2"; shift 2 ;;
        --skip-deps)    SKIP_DEPS=true; shift ;;
        --skip-service) SKIP_SERVICE=true; shift ;;
        *) err "Unknown option: $1"; exit 1 ;;
    esac
done

SKIP_DEPS="${SKIP_DEPS:-false}"
SKIP_SERVICE="${SKIP_SERVICE:-false}"

# ─────────────────────────────────────────────────────────────────────────────
# Platform
# ─────────────────────────────────────────────────────────────────────────────

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    case "$OS" in
        Linux*)  OS="linux" ;;
        Darwin*) OS="darwin" ;;
        *)       err "Unsupported OS: $OS"; exit 1 ;;
    esac
    case "$ARCH" in
        x86_64|amd64)   ARCH="x86_64" ;;
        aarch64|arm64)  ARCH="aarch64" ;;
        *)              err "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
}

check_dep() { command -v "$1" >/dev/null 2>&1; }

# ─────────────────────────────────────────────────────────────────────────────
# 1. Sandbox dependencies (Linux only)
# ─────────────────────────────────────────────────────────────────────────────

install_sandbox_deps() {
    if [ "$OS" = "darwin" ]; then
        ok "macOS — sandbox support is built-in (sandbox-exec)"
        return 0
    fi
    if [ "$SKIP_DEPS" = true ]; then
        warn "Skipping sandbox dependencies (--skip-deps)"
        return 0
    fi

    local missing=""
    check_dep bwrap  || missing="bubblewrap"
    check_dep socat  || missing="${missing:+$missing }socat"

    if [ -z "$missing" ]; then
        ok "bubblewrap and socat already installed"
        return 0
    fi

    info "Installing: ${missing}"

    local pkg_mgr=""
    if check_dep apt-get; then pkg_mgr="apt-get"
    elif check_dep dnf; then pkg_mgr="dnf"
    elif check_dep pacman; then pkg_mgr="pacman"
    elif check_dep apk; then pkg_mgr="apk"
    elif check_dep zypper; then pkg_mgr="zypper"
    fi

    if [ -z "$pkg_mgr" ]; then
        err "Could not detect package manager."
        detail "Please install manually: ${missing}"
        return 1
    fi

    case "$pkg_mgr" in
        apt-get) sudo apt-get install -y ${missing} ;;
        dnf)     sudo dnf install -y ${missing} ;;
        pacman)  sudo pacman -S --noconfirm ${missing} ;;
        apk)     sudo apk add ${missing} ;;
        zypper)  sudo zypper install -y ${missing} ;;
    esac
    ok "Sandbox dependencies installed"
}

# ─────────────────────────────────────────────────────────────────────────────
# 2. Download binary
# ─────────────────────────────────────────────────────────────────────────────

download_binary() {
    # Resolve version
    local tag="$VERSION"
    if [ "$tag" = "latest" ]; then
        info "Fetching latest release..."
        if check_dep curl; then
            tag=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
                | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
        elif check_dep wget; then
            tag=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" \
                | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
        else
            err "Neither curl nor wget found."; return 1
        fi
        [ -z "$tag" ] && { err "Could not determine latest release."; return 1; }
    fi
    info "Version: ${tag}"

    local archive="arawn-${OS}-${ARCH}.tar.gz"
    local url="https://github.com/${REPO}/releases/download/${tag}/${archive}"

    TMPDIR_INSTALL="$(mktemp -d)"
    local tmpfile="${TMPDIR_INSTALL}/${archive}"

    info "Downloading ${archive}..."
    detail "${url}"
    if check_dep curl; then
        curl -fsSL -o "${tmpfile}" "${url}" || { err "Download failed."; detail "Check https://github.com/${REPO}/releases"; return 1; }
    else
        wget -qO "${tmpfile}" "${url}" || { err "Download failed."; detail "Check https://github.com/${REPO}/releases"; return 1; }
    fi

    mkdir -p "${INSTALL_DIR}"
    tar -xzf "${tmpfile}" -C "${TMPDIR_INSTALL}"

    local binary=""
    binary="$(find "${TMPDIR_INSTALL}" -name arawn -type f | head -1)"
    [ -z "$binary" ] && { err "Could not find arawn binary in archive"; return 1; }

    cp "${binary}" "${INSTALL_DIR}/arawn"
    chmod +x "${INSTALL_DIR}/arawn"
    ok "Installed ${INSTALL_DIR}/arawn"
}

# ─────────────────────────────────────────────────────────────────────────────
# 3. Create directories + skeleton files
# ─────────────────────────────────────────────────────────────────────────────

create_skeleton() {
    for dir in "$CONFIG_DIR" "$LOG_DIR" "$WORKFLOW_DIR" "$PLUGIN_DIR"; do
        if [ -d "$dir" ]; then
            ok "${dir}"
        else
            mkdir -p "$dir"
            ok "${dir} (created)"
        fi
    done

    # Env file template
    if [ ! -f "$ENV_FILE" ]; then
        cat > "$ENV_FILE" <<'ENVFILE'
# Arawn environment variables.
# The service sources this file on startup.
# Uncomment the line for your chosen backend.

# GROQ_API_KEY=
# ANTHROPIC_API_KEY=
# OPENAI_API_KEY=
ENVFILE
        chmod 600 "$ENV_FILE"
        ok "${ENV_FILE} (created)"
    else
        ok "${ENV_FILE}"
    fi

    # Default config
    if [ ! -f "$CONFIG_FILE" ]; then
        cat > "$CONFIG_FILE" <<'TOMLFILE'
# Arawn configuration.
# Docs: https://github.com/colliery-io/arawn

[llm]
# backend = "groq"            # groq | anthropic | openai | ollama
# model = "openai/gpt-oss-20b"
# max_context_tokens = 131072

# Uncomment for ollama:
# base_url = "http://localhost:11434/v1"

[agent.default]
max_tokens = 65536

[server]
port = 8080
bind = "127.0.0.1"
TOMLFILE
        ok "${CONFIG_FILE} (created)"
    else
        ok "${CONFIG_FILE}"
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# 4. Install service
# ─────────────────────────────────────────────────────────────────────────────

install_service() {
    if [ "$SKIP_SERVICE" = true ]; then
        warn "Skipping service installation (--skip-service)"
        return 0
    fi

    if [ "$OS" = "darwin" ]; then
        install_service_macos
    else
        install_service_linux
    fi
}

install_service_macos() {
    # Wrapper script (sources env file then execs arawn)
    cat > "$WRAPPER_FILE" <<'WRAPPER'
#!/usr/bin/env bash
set -euo pipefail
ENV_FILE="$HOME/.config/arawn/env"
[ -f "$ENV_FILE" ] && { set -a; . "$ENV_FILE"; set +a; }
exec "$HOME/.local/bin/arawn" start
WRAPPER
    chmod +x "$WRAPPER_FILE"
    ok "${WRAPPER_FILE}"

    # Plist
    mkdir -p "$HOME/Library/LaunchAgents"
    cat > "$PLIST_DEST" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${PLIST_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/bash</string>
        <string>-c</string>
        <string>exec "\$HOME/.config/arawn/arawn-wrapper.sh"</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>${LOG_DIR}/launchd-stdout.log</string>
    <key>StandardErrorPath</key>
    <string>${LOG_DIR}/launchd-stderr.log</string>
    <key>WorkingDirectory</key>
    <string>${HOME}</string>
</dict>
</plist>
PLIST
    ok "${PLIST_DEST}"

    # Load service
    launchctl bootout "gui/$(id -u)/${PLIST_LABEL}" 2>/dev/null || true
    launchctl bootstrap "gui/$(id -u)" "$PLIST_DEST"
    ok "Service loaded (launchd)"
}

install_service_linux() {
    mkdir -p "$SYSTEMD_DIR"
    cat > "$SYSTEMD_DEST" <<UNIT
[Unit]
Description=Arawn AI Agent Service
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=%h/.local/bin/arawn start
Restart=on-failure
RestartSec=5
EnvironmentFile=%h/.config/arawn/env
WorkingDirectory=%h
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
UNIT
    ok "${SYSTEMD_DEST}"

    systemctl --user daemon-reload
    systemctl --user enable --now arawn
    ok "Service enabled and started (systemd)"
}

# ─────────────────────────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────────────────────────

print_next_steps() {
    banner "What to do next"

    # PATH check
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            printf "  ${YELLOW}${BOLD}1. Add arawn to your PATH${RESET}\n\n"
            local shell_name
            shell_name="$(basename "${SHELL:-/bin/bash}")"
            case "$shell_name" in
                zsh)  printf "     echo 'export PATH=\"%s:\$PATH\"' >> ~/.zshrc\n" "${INSTALL_DIR}"
                      printf "     source ~/.zshrc\n" ;;
                fish) printf "     fish_add_path %s\n" "${INSTALL_DIR}" ;;
                *)    printf "     echo 'export PATH=\"%s:\$PATH\"' >> ~/.bashrc\n" "${INSTALL_DIR}"
                      printf "     source ~/.bashrc\n" ;;
            esac
            printf "\n"
            NEXT_STEP=2
            ;;
    esac
    NEXT_STEP="${NEXT_STEP:-1}"

    # Config
    printf "  ${YELLOW}${BOLD}${NEXT_STEP}. Edit the config file${RESET}\n\n"
    printf "     Uncomment and set your backend and model:\n\n"
    printf "     ${DIM}%s %s${RESET}\n\n" "${EDITOR:-vim}" "${CONFIG_FILE}"
    printf "     Example for Groq:\n\n"
    printf "     ${DIM}[llm]${RESET}\n"
    printf "     ${DIM}backend = \"groq\"${RESET}\n"
    printf "     ${DIM}model = \"openai/gpt-oss-20b\"${RESET}\n\n"
    NEXT_STEP=$((NEXT_STEP + 1))

    # API key
    printf "  ${YELLOW}${BOLD}${NEXT_STEP}. Set your API key${RESET}\n\n"
    printf "     Pick one method:\n\n"
    printf "     ${DIM}# Encrypted store (recommended)${RESET}\n"
    printf "     arawn secrets set groq\n\n"
    printf "     ${DIM}# Or set in the service env file${RESET}\n"
    printf "     echo 'GROQ_API_KEY=gsk_...' >> %s\n\n" "${ENV_FILE}"
    printf "     ${DIM}# Or export directly${RESET}\n"
    printf "     export GROQ_API_KEY=gsk_...\n\n"
    printf "     ${DIM}Keys by provider:${RESET}\n"
    printf "     ${DIM}  Groq:      https://console.groq.com/keys${RESET}\n"
    printf "     ${DIM}  Anthropic: https://console.anthropic.com/settings/keys${RESET}\n"
    printf "     ${DIM}  OpenAI:    https://platform.openai.com/api-keys${RESET}\n"
    printf "     ${DIM}  Ollama:    no key needed${RESET}\n\n"
    NEXT_STEP=$((NEXT_STEP + 1))

    # Restart service
    printf "  ${YELLOW}${BOLD}${NEXT_STEP}. Restart the service${RESET}\n\n"
    printf "     After editing config and setting your key, restart:\n\n"
    if [ "$OS" = "darwin" ]; then
        printf "     launchctl kickstart -k gui/\$(id -u)/%s\n\n" "${PLIST_LABEL}"
    else
        printf "     systemctl --user restart arawn\n\n"
    fi

    # Reference
    banner "Reference"

    printf "  ${BOLD}Files${RESET}\n"
    printf "  %-10s %s\n" "Binary" "${INSTALL_DIR}/arawn"
    printf "  %-10s %s\n" "Config" "${CONFIG_FILE}"
    printf "  %-10s %s\n" "Env" "${ENV_FILE}"
    printf "  %-10s %s/\n" "Logs" "${LOG_DIR}"
    printf "\n"

    printf "  ${BOLD}Service commands${RESET}\n"
    if [ "$OS" = "darwin" ]; then
        printf "  %-10s %s\n" "Status" "launchctl print gui/\$(id -u)/${PLIST_LABEL}"
        printf "  %-10s %s\n" "Logs" "tail -f ${LOG_DIR}/launchd-stdout.log"
        printf "  %-10s %s\n" "Restart" "launchctl kickstart -k gui/\$(id -u)/${PLIST_LABEL}"
        printf "  %-10s %s\n" "Stop" "launchctl bootout gui/\$(id -u)/${PLIST_LABEL}"
    else
        printf "  %-10s %s\n" "Status" "systemctl --user status arawn"
        printf "  %-10s %s\n" "Logs" "journalctl --user -u arawn -f"
        printf "  %-10s %s\n" "Restart" "systemctl --user restart arawn"
        printf "  %-10s %s\n" "Stop" "systemctl --user stop arawn"
    fi
    printf "\n"

    printf "  ${BOLD}Uninstall${RESET}\n"
    printf "  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/uninstall.sh | bash\n"
    printf "\n"
}

# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────

main() {
    detect_platform
    TOTAL_STEPS=4

    banner "Arawn Installer — ${OS}/${ARCH}"

    step 1 "Sandbox dependencies"
    install_sandbox_deps
    printf "\n"

    step 2 "Download arawn"
    download_binary
    printf "\n"

    step 3 "Create config skeleton"
    create_skeleton
    printf "\n"

    step 4 "Install service"
    install_service
    printf "\n"

    print_next_steps
}

main

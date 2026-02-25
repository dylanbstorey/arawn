#!/usr/bin/env bash
# Arawn installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/colliery-io/arawn/main/scripts/install.sh | sh
#
# Installs Arawn and its sandbox dependencies (Linux only).
# macOS sandbox support is built-in via sandbox-exec.

set -euo pipefail

REPO="colliery-io/arawn"
DEFAULT_INSTALL_DIR="${HOME}/.local/bin"

# Defaults
INSTALL_DIR="${DEFAULT_INSTALL_DIR}"
SKIP_DEPS=false
DRY_RUN=false
VERSION="latest"

# --- Color output ---

if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    RESET=''
fi

info()  { printf "${BLUE}info${RESET}  %s\n" "$1"; }
ok()    { printf "${GREEN}ok${RESET}    %s\n" "$1"; }
warn()  { printf "${YELLOW}warn${RESET}  %s\n" "$1"; }
err()   { printf "${RED}error${RESET} %s\n" "$1" >&2; }

# --- Cleanup ---

TMPDIR_INSTALL=""
cleanup() {
    if [ -n "${TMPDIR_INSTALL}" ] && [ -d "${TMPDIR_INSTALL}" ]; then
        rm -rf "${TMPDIR_INSTALL}"
    fi
}
trap cleanup EXIT

# --- Usage ---

usage() {
    cat <<EOF
${BOLD}Arawn Installer${RESET}

Install Arawn and its sandbox dependencies.

${BOLD}USAGE${RESET}
    install.sh [OPTIONS]

${BOLD}OPTIONS${RESET}
    --help              Show this help message
    --skip-deps         Skip sandbox dependency installation (bubblewrap, socat)
    --install-dir DIR   Install directory (default: ${DEFAULT_INSTALL_DIR})
    --dry-run           Show what would happen without making changes
    --version VER       Install a specific version tag (default: latest)

${BOLD}EXAMPLES${RESET}
    # Standard install (latest release)
    curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | sh

    # Install a specific version
    ./install.sh --version v0.2.0

    # Install to custom directory
    ./install.sh --install-dir /usr/local/bin

    # Preview what would happen
    ./install.sh --dry-run

EOF
}

# --- Argument parsing ---

while [ $# -gt 0 ]; do
    case "$1" in
        --help)
            usage
            exit 0
            ;;
        --skip-deps)
            SKIP_DEPS=true
            shift
            ;;
        --install-dir)
            if [ $# -lt 2 ]; then
                err "--install-dir requires an argument"
                exit 1
            fi
            INSTALL_DIR="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --version)
            if [ $# -lt 2 ]; then
                err "--version requires an argument"
                exit 1
            fi
            VERSION="$2"
            shift 2
            ;;
        *)
            err "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# --- Platform detection ---

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "darwin" ;;
        *)       echo "unsupported" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64" ;;
        aarch64|arm64)  echo "aarch64" ;;
        *)              echo "unsupported" ;;
    esac
}

detect_package_manager() {
    if command -v apt-get >/dev/null 2>&1; then
        echo "apt-get"
    elif command -v dnf >/dev/null 2>&1; then
        echo "dnf"
    elif command -v pacman >/dev/null 2>&1; then
        echo "pacman"
    elif command -v apk >/dev/null 2>&1; then
        echo "apk"
    elif command -v zypper >/dev/null 2>&1; then
        echo "zypper"
    else
        echo "unknown"
    fi
}

# --- Dependency installation (Linux only) ---

check_dep() {
    command -v "$1" >/dev/null 2>&1
}

install_deps_linux() {
    local pkg_mgr
    pkg_mgr="$(detect_package_manager)"

    local missing=""
    if ! check_dep bwrap; then
        missing="bubblewrap"
    fi
    if ! check_dep socat; then
        if [ -n "${missing}" ]; then
            missing="${missing} socat"
        else
            missing="socat"
        fi
    fi

    if [ -z "${missing}" ]; then
        ok "Sandbox dependencies already installed (bubblewrap, socat)"
        return 0
    fi

    info "Missing sandbox dependencies: ${missing}"

    if [ "${pkg_mgr}" = "unknown" ]; then
        err "Could not detect package manager. Please install manually: ${missing}"
        return 1
    fi

    local install_cmd=""
    case "${pkg_mgr}" in
        apt-get) install_cmd="sudo apt-get install -y ${missing}" ;;
        dnf)     install_cmd="sudo dnf install -y ${missing}" ;;
        pacman)  install_cmd="sudo pacman -S --noconfirm ${missing}" ;;
        apk)     install_cmd="sudo apk add ${missing}" ;;
        zypper)  install_cmd="sudo zypper install -y ${missing}" ;;
    esac

    if [ "${DRY_RUN}" = true ]; then
        info "[dry-run] Would run: ${install_cmd}"
        return 0
    fi

    info "Installing sandbox dependencies: ${install_cmd}"
    eval "${install_cmd}"
    ok "Sandbox dependencies installed"
}

# --- Binary download ---

resolve_version() {
    if [ "${VERSION}" != "latest" ]; then
        echo "${VERSION}"
        return 0
    fi

    info "Fetching latest release tag..."
    local tag=""
    if command -v curl >/dev/null 2>&1; then
        tag=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
    elif command -v wget >/dev/null 2>&1; then
        tag=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
    else
        err "Neither curl nor wget found. Cannot download release."
        return 1
    fi

    if [ -z "${tag}" ]; then
        err "Could not determine latest release tag. Check https://github.com/${REPO}/releases"
        return 1
    fi

    info "Latest release: ${tag}"
    echo "${tag}"
}

download_binary() {
    local os="$1"
    local arch="$2"
    local tag="$3"

    local archive="arawn-${os}-${arch}.tar.gz"
    local url="https://github.com/${REPO}/releases/download/${tag}/${archive}"

    if [ "${DRY_RUN}" = true ]; then
        info "[dry-run] Would download: ${url}"
        info "[dry-run] Would extract to: ${INSTALL_DIR}/arawn"
        return 0
    fi

    TMPDIR_INSTALL="$(mktemp -d)"
    local tmpfile="${TMPDIR_INSTALL}/${archive}"

    info "Downloading ${url}..."
    if command -v curl >/dev/null 2>&1; then
        if ! curl -fsSL -o "${tmpfile}" "${url}"; then
            err "Download failed. Check that release ${tag} exists at https://github.com/${REPO}/releases"
            return 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -qO "${tmpfile}" "${url}"; then
            err "Download failed. Check that release ${tag} exists at https://github.com/${REPO}/releases"
            return 1
        fi
    fi

    info "Extracting to ${INSTALL_DIR}..."
    mkdir -p "${INSTALL_DIR}"
    tar -xzf "${tmpfile}" -C "${TMPDIR_INSTALL}"

    # Look for the arawn binary in the extracted contents
    local binary=""
    if [ -f "${TMPDIR_INSTALL}/arawn" ]; then
        binary="${TMPDIR_INSTALL}/arawn"
    elif [ -f "${TMPDIR_INSTALL}/arawn/arawn" ]; then
        binary="${TMPDIR_INSTALL}/arawn/arawn"
    else
        binary="$(find "${TMPDIR_INSTALL}" -name arawn -type f | head -1)"
    fi

    if [ -z "${binary}" ]; then
        err "Could not find arawn binary in archive"
        return 1
    fi

    cp "${binary}" "${INSTALL_DIR}/arawn"
    chmod +x "${INSTALL_DIR}/arawn"
    ok "Installed arawn to ${INSTALL_DIR}/arawn"
}

# --- Verification ---

verify_install() {
    local os="$1"

    if [ "${DRY_RUN}" = true ]; then
        info "[dry-run] Would verify: arawn --version"
        if [ "${os}" = "linux" ]; then
            info "[dry-run] Would verify: bwrap --version"
            info "[dry-run] Would verify: socat -V"
        fi
        return 0
    fi

    printf "\n${BOLD}Verification${RESET}\n"

    if check_dep arawn; then
        ok "arawn $(arawn --version 2>/dev/null || echo '(version unknown)')"
    else
        warn "arawn not found in PATH (you may need to add ${INSTALL_DIR} to your PATH)"
    fi

    if [ "${os}" = "linux" ] && [ "${SKIP_DEPS}" = false ]; then
        if check_dep bwrap; then
            ok "bwrap $(bwrap --version 2>/dev/null || echo '(version unknown)')"
        else
            warn "bwrap not found"
        fi

        if check_dep socat; then
            ok "socat available"
        else
            warn "socat not found"
        fi
    fi
}

# --- Path instructions ---

print_path_instructions() {
    # Check if install dir is already in PATH
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) return 0 ;;
    esac

    printf "\n${YELLOW}Note${RESET}: ${INSTALL_DIR} is not in your PATH.\n"
    printf "Add it by appending this to your shell profile:\n\n"

    local shell_name
    shell_name="$(basename "${SHELL:-/bin/bash}")"

    case "${shell_name}" in
        zsh)
            printf "  echo 'export PATH=\"%s:\$PATH\"' >> ~/.zshrc\n" "${INSTALL_DIR}"
            printf "  source ~/.zshrc\n"
            ;;
        fish)
            printf "  fish_add_path %s\n" "${INSTALL_DIR}"
            ;;
        *)
            printf "  echo 'export PATH=\"%s:\$PATH\"' >> ~/.bashrc\n" "${INSTALL_DIR}"
            printf "  source ~/.bashrc\n"
            ;;
    esac
    printf "\n"
}

# --- Main ---

main() {
    printf "\n${BOLD}Arawn Installer${RESET}\n\n"

    local os arch
    os="$(detect_os)"
    arch="$(detect_arch)"

    if [ "${os}" = "unsupported" ]; then
        err "Unsupported operating system: $(uname -s)"
        err "Arawn supports Linux and macOS."
        exit 1
    fi

    if [ "${arch}" = "unsupported" ]; then
        err "Unsupported architecture: $(uname -m)"
        err "Arawn supports x86_64 and aarch64 (ARM64)."
        exit 1
    fi

    info "Detected platform: ${os}/${arch}"

    # Step 1: Install sandbox dependencies (Linux only)
    if [ "${os}" = "linux" ] && [ "${SKIP_DEPS}" = false ]; then
        info "Installing sandbox dependencies..."
        install_deps_linux
    elif [ "${os}" = "darwin" ]; then
        ok "macOS detected — sandbox support is built-in (sandbox-exec)"
    fi

    # Step 2: Download arawn binary from GitHub Releases
    local tag
    tag="$(resolve_version)"
    download_binary "${os}" "${arch}" "${tag}"

    # Step 3: Verify
    verify_install "${os}"

    # Step 4: Path instructions
    if [ "${DRY_RUN}" = false ]; then
        print_path_instructions
    fi

    printf "${GREEN}${BOLD}Done!${RESET}\n\n"
}

main

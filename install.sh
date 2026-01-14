#!/bin/bash
# Mole-RS Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/thirukguru/mole-rs/main/install.sh | bash
#        curl -fsSL https://raw.githubusercontent.com/thirukguru/mole-rs/main/install.sh | bash -s -- -v 0.1.0

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
REPO="thirukguru/mole-rs"
BINARY_NAME="mo"
INSTALL_DIR="${HOME}/.local/bin"

# Print colored output
info() { echo -e "${CYAN}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }

# Detect OS and architecture
detect_platform() {
    local os arch

    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        linux) os="linux" ;;
        darwin) os="darwin" ;;
        *)
            error "Unsupported OS: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

# Get latest release version from GitHub
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | \
        grep '"tag_name"' | \
        head -1 | \
        sed -E 's/.*"v?([^"]+)".*/\1/'
}

# Download and install binary
install_binary() {
    local version="${1:-}"
    local platform
    local download_url
    local tmp_dir

    platform=$(detect_platform)
    
    # Get version if not specified
    if [[ -z "$version" ]]; then
        info "Fetching latest version..."
        version=$(get_latest_version)
        if [[ -z "$version" ]]; then
            error "Could not determine latest version"
            error "Please specify a version with: -v VERSION"
            exit 1
        fi
    fi

    info "Installing Mole-RS v${version} for ${platform}..."

    # Create temp directory
    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    # Construct download URL
    # Format: mole-rs-{version}-{platform}.tar.gz
    download_url="https://github.com/${REPO}/releases/download/v${version}/mole-rs-${version}-${platform}.tar.gz"

    info "Downloading from: ${download_url}"

    # Download
    if ! curl -fsSL "$download_url" -o "${tmp_dir}/mole-rs.tar.gz"; then
        error "Failed to download release"
        error "URL: ${download_url}"
        error ""
        error "If no releases exist yet, build from source:"
        error "  git clone https://github.com/${REPO}.git"
        error "  cd mole-rs && cargo build --release"
        error "  cp target/release/mo ~/.local/bin/"
        exit 1
    fi

    # Extract
    info "Extracting..."
    tar -xzf "${tmp_dir}/mole-rs.tar.gz" -C "${tmp_dir}"

    # Create install directory if needed
    mkdir -p "$INSTALL_DIR"

    # Install binary
    if [[ -f "${tmp_dir}/${BINARY_NAME}" ]]; then
        cp "${tmp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    elif [[ -f "${tmp_dir}/mole-rs-${version}-${platform}/${BINARY_NAME}" ]]; then
        cp "${tmp_dir}/mole-rs-${version}-${platform}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    else
        # Try to find the binary
        local found_binary
        found_binary=$(find "${tmp_dir}" -name "${BINARY_NAME}" -type f | head -1)
        if [[ -n "$found_binary" ]]; then
            cp "$found_binary" "${INSTALL_DIR}/${BINARY_NAME}"
        else
            error "Could not find binary in archive"
            exit 1
        fi
    fi

    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    success "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
}

# Build from source (fallback)
build_from_source() {
    info "Building from source..."

    # Check for Rust
    if ! command -v cargo &>/dev/null; then
        error "Rust is not installed. Install it with:"
        error "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    info "Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "${tmp_dir}/mole-rs"

    info "Building release binary..."
    cd "${tmp_dir}/mole-rs"
    cargo build --release

    mkdir -p "$INSTALL_DIR"
    cp "target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    success "Built and installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
}

# Add to PATH if needed
setup_path() {
    local shell_config=""
    local path_line="export PATH=\"\$HOME/.local/bin:\$PATH\""

    # Detect shell config file
    if [[ -n "${BASH_VERSION:-}" ]]; then
        shell_config="${HOME}/.bashrc"
    elif [[ -n "${ZSH_VERSION:-}" ]]; then
        shell_config="${HOME}/.zshrc"
    elif [[ -f "${HOME}/.bashrc" ]]; then
        shell_config="${HOME}/.bashrc"
    elif [[ -f "${HOME}/.zshrc" ]]; then
        shell_config="${HOME}/.zshrc"
    fi

    # Check if already in PATH
    if echo "$PATH" | grep -q "${HOME}/.local/bin"; then
        return 0
    fi

    if [[ -n "$shell_config" ]]; then
        if ! grep -q "/.local/bin" "$shell_config" 2>/dev/null; then
            echo "" >> "$shell_config"
            echo "# Added by Mole-RS installer" >> "$shell_config"
            echo "$path_line" >> "$shell_config"
            warn "Added ${INSTALL_DIR} to PATH in ${shell_config}"
            warn "Run 'source ${shell_config}' or restart your terminal"
        fi
    else
        warn "Could not detect shell config file"
        warn "Add this to your shell config:"
        warn "  ${path_line}"
    fi
}

# Print usage
usage() {
    cat <<EOF
Mole-RS Installer

Usage:
    curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash
    curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash -s -- [OPTIONS]

Options:
    -v, --version VERSION   Install specific version (e.g., 0.1.0)
    -s, --source            Build from source instead of downloading binary
    -d, --dir DIR           Install to DIR (default: ~/.local/bin)
    -h, --help              Show this help message

Examples:
    # Install latest release
    curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash

    # Install specific version
    curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash -s -- -v 0.1.0

    # Build from source
    curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash -s -- -s

EOF
}

# Main
main() {
    local version=""
    local from_source=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -v|--version)
                version="$2"
                shift 2
                ;;
            -s|--source)
                from_source=true
                shift
                ;;
            -d|--dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    echo ""
    echo -e "${CYAN}🐹 Mole-RS Installer${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    if $from_source; then
        build_from_source
    else
        install_binary "$version"
    fi

    setup_path

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    success "Installation complete!"
    echo ""
    echo "Run 'mo' to start Mole-RS"
    echo ""
}

main "$@"

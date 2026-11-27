#!/usr/bin/env bash
# ==============================================================================
# clrinf CLI & MCP Linux Installation Script
# Supports: CachyOS, Arch Linux, Debian, Ubuntu, Fedora, openSUSE, etc.
# Compliant with XDG Base Directory & Linux FHS Standards
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEGEN_DIR="${SCRIPT_DIR}/tools/clrinf-codegen"

# Default install targets (FHS / XDG Compliant)
DEFAULT_USER_BIN="${XDG_BIN_HOME:-$HOME/.local/bin}"
DEFAULT_SYSTEM_BIN="/usr/local/bin"
INSTALL_DIR="${DEFAULT_USER_BIN}"
IS_SYSTEM=false
FORCE_BUILD=false
DO_UNINSTALL=false

show_help() {
    cat << EOF
${BOLD}clrinf CLI & MCP Linux/macOS Installer${NC}
Supports modern Linux distributions and macOS.

${BOLD}USAGE:${NC}
    ./install.sh [OPTIONS]

${BOLD}OPTIONS:${NC}
    --user               Install to user bin (~/.local/bin) [Default, no root required]
    --system             Install system-wide (/usr/local/bin) [Requires sudo]
    --dir <PATH>         Custom installation directory
    --build              Force build from source (cargo build --release)
    --uninstall          Remove installed binaries and symlinks
    -h, --help           Show this help message

EOF
}

# Parse CLI arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --system)
            INSTALL_DIR="${DEFAULT_SYSTEM_BIN}"
            IS_SYSTEM=true
            shift
            ;;
        --user)
            INSTALL_DIR="${DEFAULT_USER_BIN}"
            IS_SYSTEM=false
            shift
            ;;
        --dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --build)
            FORCE_BUILD=true
            shift
            ;;
        --uninstall)
            DO_UNINSTALL=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Check OS distribution info
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO_NAME="${NAME:-Linux}"
        DISTRO_ID="${ID:-linux}"
    else
        DISTRO_NAME="Linux"
        DISTRO_ID="linux"
    fi
}

detect_distro

# ==============================================================================
# UNINSTALL FLOW
# ==============================================================================
if [ "${DO_UNINSTALL}" = true ]; then
    echo -e "${YELLOW}🧹 Removing clrinf CLI and tools...${NC}"
    
    if [ "${IS_SYSTEM}" = true ] && [ "$EUID" -ne 0 ]; then
        SUDO="sudo"
    else
        SUDO=""
    fi

    ${SUDO} rm -f "${INSTALL_DIR}/clrinf-codegen"
    ${SUDO} rm -f "${INSTALL_DIR}/clrinf"
    
    # Also check ~/.cargo/bin
    if [ -d "${HOME}/.cargo/bin" ]; then
        rm -f "${HOME}/.cargo/bin/clrinf-codegen"
        rm -f "${HOME}/.cargo/bin/clrinf"
    fi

    echo -e "${GREEN}✅ clrinf has been uninstalled.${NC}"
    exit 0
fi

# ==============================================================================
# INSTALL FLOW
# ==============================================================================
echo -e "${CYAN}${BOLD}🚀 Starting clrinf Installer (${DISTRO_NAME} / ${DISTRO_ID})${NC}"

GITHUB_REPO="brkmustu/clrinf"
CLRINF_VERSION="${CLRINF_VERSION:-latest}"
TARGET_BIN=""

# Platform Detection (OS & Architecture)
detect_platform() {
    local os arch
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"

    case "${arch}" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            echo -e "${YELLOW}⚠️  Unsupported architecture: ${arch}${NC}"
            return 1
            ;;
    esac

    case "${os}" in
        linux)
            # Default to gnu for maximum compatibility
            echo "${arch}-unknown-linux-gnu"
            ;;
        darwin)
            echo "${arch}-apple-darwin"
            ;;
        *)
            echo -e "${YELLOW}⚠️  Unsupported OS: ${os}${NC}"
            return 1
            ;;
    esac
}

# 1. Binary Source Resolution: Try Pre-built Release Binary first (Fast & No Cargo required)
if [ "${FORCE_BUILD}" = false ]; then
    # First, check if already built locally in workspace
    POSSIBLE_BINS=(
        "${CODEGEN_DIR}/target/release/clrinf-codegen"
        "${SCRIPT_DIR}/target/release/clrinf-codegen"
    )
    for bin in "${POSSIBLE_BINS[@]}"; do
        if [ -f "${bin}" ] && [ -x "${bin}" ]; then
            TARGET_BIN="${bin}"
            echo -e "${GREEN}✓ Found locally built binary: ${TARGET_BIN}${NC}"
            break
        fi
    done

    # If no local binary, try downloading pre-built binary package from GitHub Releases
    if [ -z "${TARGET_BIN}" ]; then
        PLATFORM_TARGET="$(detect_platform || true)"
        if [ -n "${PLATFORM_TARGET}" ]; then
            echo -e "${BLUE}🌐 Checking GitHub Releases for pre-built binary (${PLATFORM_TARGET})...${NC}"
            
            if [ "${CLRINF_VERSION}" = "latest" ]; then
                RELEASE_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/clrinf-${PLATFORM_TARGET}.tar.gz"
            else
                RELEASE_URL="https://github.com/${GITHUB_REPO}/releases/download/${CLRINF_VERSION}/clrinf-${PLATFORM_TARGET}.tar.gz"
            fi

            TMP_DIR=$(mktemp -d)
            TAR_FILE="${TMP_DIR}/clrinf.tar.gz"

            DOWNLOADED=false
            if command -v curl &> /dev/null; then
                if curl -fsSL "${RELEASE_URL}" -o "${TAR_FILE}" 2>/dev/null; then
                    DOWNLOADED=true
                fi
            elif command -v wget &> /dev/null; then
                if wget -q "${RELEASE_URL}" -O "${TAR_FILE}" 2>/dev/null; then
                    DOWNLOADED=true
                fi
            fi

            if [ "${DOWNLOADED}" = true ] && [ -f "${TAR_FILE}" ] && [ -s "${TAR_FILE}" ]; then
                echo -e "${GREEN}✓ Downloaded pre-built archive. Extracting...${NC}"
                tar -xzf "${TAR_FILE}" -C "${TMP_DIR}"
                
                # Check for extracted binary
                if [ -f "${TMP_DIR}/clrinf-codegen" ]; then
                    TARGET_BIN="${TMP_DIR}/clrinf-codegen"
                elif [ -f "${TMP_DIR}/clrinf" ]; then
                    TARGET_BIN="${TMP_DIR}/clrinf"
                fi
            else
                echo -e "${YELLOW}ℹ️  Pre-built GitHub Release package not found (not published yet or offline).${NC}"
            fi
        fi
    fi
fi

# Fallback: Build from source if binary is still not found or --build is explicitly requested
if [ -z "${TARGET_BIN}" ] || [ "${FORCE_BUILD}" = true ]; then
    if [ -d "${CODEGEN_DIR}" ] && [ -f "${CODEGEN_DIR}/Cargo.toml" ]; then
        echo -e "${BLUE}📦 Building clrinf-codegen from local source (cargo build --release)...${NC}"
        
        if ! command -v cargo &> /dev/null; then
            echo -e "${RED}❌ 'cargo' not found! Rust is required to build from source.${NC}"
            if [[ "${DISTRO_ID}" == "cachyos" || "${DISTRO_ID}" == "arch" ]]; then
                echo -e "${YELLOW}👉 Install on Arch/CachyOS:${NC} sudo pacman -S rust"
            else
                echo -e "${YELLOW}👉 Install Rust via https://rustup.rs${NC}"
            fi
            exit 1
        fi

        cd "${CODEGEN_DIR}"
        cargo build --release
        TARGET_BIN="${CODEGEN_DIR}/target/release/clrinf-codegen"
    else
        echo -e "${RED}❌ No pre-built binary found and local source directory is unavailable.${NC}"
        echo -e "👉 Clone the repository and build from source:"
        echo -e "   git clone https://github.com/${GITHUB_REPO}.git"
        echo -e "   cd clrinf && ./install.sh --build"
        exit 1
    fi
fi

if [ ! -f "${TARGET_BIN}" ]; then
    echo -e "${RED}❌ Could not verify binary: ${TARGET_BIN}${NC}"
    exit 1
fi

echo -e "${BLUE}📁 Preparing install directory: ${INSTALL_DIR}${NC}"
if [ "${IS_SYSTEM}" = true ]; then
    if [ "$EUID" -ne 0 ]; then
        SUDO="sudo"
    else
        SUDO=""
    fi
    ${SUDO} mkdir -p "${INSTALL_DIR}"
    echo -e "${BLUE}⬇️  Installing system-wide (${INSTALL_DIR})...${NC}"
    ${SUDO} install -m 755 "${TARGET_BIN}" "${INSTALL_DIR}/clrinf-codegen"
    ${SUDO} ln -sf "clrinf-codegen" "${INSTALL_DIR}/clrinf"
else
    mkdir -p "${INSTALL_DIR}"
    echo -e "${BLUE}⬇️  Installing to user bin (${INSTALL_DIR})...${NC}"
    install -m 755 "${TARGET_BIN}" "${INSTALL_DIR}/clrinf-codegen"
    ln -sf "clrinf-codegen" "${INSTALL_DIR}/clrinf"

    # Also link to ~/.cargo/bin if it exists and is distinct
    if [ -d "${HOME}/.cargo/bin" ] && [ "${INSTALL_DIR}" != "${HOME}/.cargo/bin" ]; then
        ln -sf "${INSTALL_DIR}/clrinf-codegen" "${HOME}/.cargo/bin/clrinf-codegen" 2>/dev/null || true
        ln -sf "${INSTALL_DIR}/clrinf" "${HOME}/.cargo/bin/clrinf" 2>/dev/null || true
    fi
fi

# 2. PATH Verification
PATH_CHECK=false
if [[ ":$PATH:" == *":${INSTALL_DIR}:"* ]]; then
    PATH_CHECK=true
fi


echo ""
echo -e "${GREEN}${BOLD}🎉 clrinf CLI & MCP Server installed successfully!${NC}"
echo -e "  📍 Binary:  ${BOLD}${INSTALL_DIR}/clrinf-codegen${NC}"
echo -e "  🔗 Alias:   ${BOLD}${INSTALL_DIR}/clrinf${NC}"
echo ""

if [ "${PATH_CHECK}" = true ]; then
    echo -e "${GREEN}✅ '${INSTALL_DIR}' is already in your PATH.${NC}"
else
    echo -e "${YELLOW}⚠️  Note: '${INSTALL_DIR}' is not currently in your PATH.${NC}"
    echo -e "Add the following line to your shell profile:"
    echo -e "  • Bash: ${BOLD}echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc${NC}"
    echo -e "  • Zsh:  ${BOLD}echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc${NC}"
    echo -e "  • Fish: ${BOLD}fish_add_path ${INSTALL_DIR}${NC}"
fi

echo ""
echo -e "${BOLD}Quick Verification Commands:${NC}"
echo -e "  ${CYAN}clrinf --version${NC}"
echo -e "  ${CYAN}clrinf verify --help${NC}"
echo -e "  ${CYAN}clrinf catalog${NC}"
echo -e "  ${CYAN}clrinf mcp --profile lean${NC}"
echo ""

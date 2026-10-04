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
${BOLD}clrinf CLI & MCP Linux Kurulum Aracı${NC}
CachyOS, Arch Linux ve tüm modern Linux dağıtımlarıyla tam uyumludur.

${BOLD}KULLANIM:${NC}
    ./install.sh [SEÇENEKLER]

${BOLD}SEÇENEKLER:${NC}
    --user               Kullanıcı dizinine kur (~/.local/bin) [Varsayılan, root gerektirmez]
    --system             Sistem geneline kur (/usr/local/bin) [sudo gerektirir]
    --dir <PATH>         Özel bir hedef kurulum dizini belirle
    --build              Kaynak koddan yeniden derlemeyi zorunlu kıl (cargo build --release)
    --uninstall          Yüklü ikili dosyaları ve sembolik bağları kaldır
    -h, --help           Bu yardım mesajını göster

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
            echo -e "${RED}Bilinmeyen seçenek: $1${NC}"
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
    echo -e "${YELLOW}🧹 clrinf CLI ve araçları kaldırılıyor...${NC}"
    
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

    echo -e "${GREEN}✅ clrinf başarıyla kaldırıldı.${NC}"
    exit 0
fi

# ==============================================================================
# INSTALL FLOW
# ==============================================================================
echo -e "${CYAN}${BOLD}🚀 clrinf Kurulum Motoru başlatılıyor (${DISTRO_NAME} / ${DISTRO_ID})${NC}"

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
            echo -e "${YELLOW}⚠️  Desteklenmeyen mimari: ${arch}${NC}"
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
            echo -e "${YELLOW}⚠️  Desteklenmeyen işletim sistemi: ${os}${NC}"
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
            echo -e "${GREEN}✓ Yerel olarak derlenmiş ikili dosya bulundu: ${TARGET_BIN}${NC}"
            break
        fi
    done

    # If no local binary, try downloading pre-built binary package from GitHub Releases
    if [ -z "${TARGET_BIN}" ]; then
        PLATFORM_TARGET="$(detect_platform || true)"
        if [ -n "${PLATFORM_TARGET}" ]; then
            echo -e "${BLUE}🌐 GitHub Releases üzerinden önceden derlenmiş paket aranıyor (${PLATFORM_TARGET})...${NC}"
            
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
                echo -e "${GREEN}✓ Önceden derlenmiş paket indirildi. Arşiv açılıyor...${NC}"
                tar -xzf "${TAR_FILE}" -C "${TMP_DIR}"
                
                # Check for extracted binary
                if [ -f "${TMP_DIR}/clrinf-codegen" ]; then
                    TARGET_BIN="${TMP_DIR}/clrinf-codegen"
                elif [ -f "${TMP_DIR}/clrinf" ]; then
                    TARGET_BIN="${TMP_DIR}/clrinf"
                fi
            else
                echo -e "${YELLOW}ℹ️  GitHub Release paketi bulunamadı (henüz yayınlanmamış veya ağ erişilemiyor).${NC}"
            fi
        fi
    fi
fi

# Fallback: Build from source if binary is still not found or --build is explicitly requested
if [ -z "${TARGET_BIN}" ] || [ "${FORCE_BUILD}" = true ]; then
    if [ -d "${CODEGEN_DIR}" ] && [ -f "${CODEGEN_DIR}/Cargo.toml" ]; then
        echo -e "${BLUE}📦 clrinf-codegen yerel kaynak koddan derleniyor (cargo build --release)...${NC}"
        
        if ! command -v cargo &> /dev/null; then
            echo -e "${RED}❌ 'cargo' bulunamadı! Önceden derlenmiş ikili paket bulunamadığı için derleme için Rust gereklidir.${NC}"
            if [[ "${DISTRO_ID}" == "cachyos" || "${DISTRO_ID}" == "arch" ]]; then
                echo -e "${YELLOW}👉 CachyOS/Arch üzerinde yüklemek için:${NC} sudo pacman -S rust"
            else
                echo -e "${YELLOW}👉 https://rustup.rs üzerinden Rust kurabilirsiniz.${NC}"
            fi
            exit 1
        fi

        cd "${CODEGEN_DIR}"
        cargo build --release
        TARGET_BIN="${CODEGEN_DIR}/target/release/clrinf-codegen"
    else
        echo -e "${RED}❌ Kurulum için uygun bir ikili dosya bulunamadı ve yerel kaynak kod dizini mevcut değil.${NC}"
        echo -e "👉 Depoyu kaynak koddan klonlayarak kurmak için:"
        echo -e "   git clone --recurse-submodules https://github.com/${GITHUB_REPO}.git"
        echo -e "   cd clrinf && ./install.sh --build"
        exit 1
    fi
fi

if [ ! -f "${TARGET_BIN}" ]; then
    echo -e "${RED}❌ İkili dosya doğrulanamadı: ${TARGET_BIN}${NC}"
    exit 1
fi

echo -e "${BLUE}📁 Hedef dizin hazırlanıyor: ${INSTALL_DIR}${NC}"
if [ "${IS_SYSTEM}" = true ]; then
    if [ "$EUID" -ne 0 ]; then
        SUDO="sudo"
    else
        SUDO=""
    fi
    ${SUDO} mkdir -p "${INSTALL_DIR}"
    echo -e "${BLUE}⬇️  Sistem geneline kopyalanıyor (${INSTALL_DIR})...${NC}"
    ${SUDO} install -m 755 "${TARGET_BIN}" "${INSTALL_DIR}/clrinf-codegen"
    ${SUDO} ln -sf "clrinf-codegen" "${INSTALL_DIR}/clrinf"
else
    mkdir -p "${INSTALL_DIR}"
    echo -e "${BLUE}⬇️  Kullanıcı dizinine kopyalanıyor (${INSTALL_DIR})...${NC}"
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
echo -e "${GREEN}${BOLD}🎉 clrinf CLI ve MCP Sunucusu başarıyla kuruldu!${NC}"
echo -e "  📍 Binary:     ${BOLD}${INSTALL_DIR}/clrinf-codegen${NC}"
echo -e "  🔗 Kısayol:    ${BOLD}${INSTALL_DIR}/clrinf${NC}"
echo ""

if [ "${PATH_CHECK}" = true ]; then
    echo -e "${GREEN}✅ '${INSTALL_DIR}' dizini PATH ortam değişkeninizde mevcut.${NC}"
else
    echo -e "${YELLOW}⚠️  Dikkat: '${INSTALL_DIR}' dizini şu anda PATH ortam değişkeninizde görünmüyor.${NC}"
    echo -e "Kabuğunuza göre aşağıdaki satırı profil dosyanıza ekleyin:"
    echo -e "  • Bash: ${BOLD}echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc${NC}"
    echo -e "  • Zsh:  ${BOLD}echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc${NC}"
    echo -e "  • Fish: ${BOLD}fish_add_path ${INSTALL_DIR}${NC}"
fi

echo ""
echo -e "${BOLD}Hızlı Doğrulama Komutları:${NC}"
echo -e "  ${CYAN}clrinf --version${NC}"
echo -e "  ${CYAN}clrinf catalog${NC}"
echo -e "  ${CYAN}clrinf docs --lang csharp${NC}"
echo -e "  ${CYAN}clrinf mcp${NC}"
echo ""

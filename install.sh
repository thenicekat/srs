#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

REPO="thenicekat/srs"
BINARY_NAME="srs"
INSTALL_DIR="$HOME/.local/bin"
SHELL_RC=""

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

detect_platform() {
    local os=$(uname -s)
    local arch=$(uname -m)
    
    case $os in
        Darwin)
            case $arch in
                x86_64)
                    echo "macos-x86_64"
                    ;;
                arm64|aarch64)
                    echo "macos-aarch64"
                    ;;
                *)
                    print_error "Unsupported macOS architecture: $arch"
                    exit 1
                    ;;
            esac
            ;;
        Linux)
            case $arch in
                x86_64)
                    echo "linux-x86_64"
                    ;;
                aarch64|arm64)
                    echo "linux-aarch64"
                    ;;
                *)
                    print_error "Unsupported Linux architecture: $arch"
                    exit 1
                    ;;
            esac
            ;;
        *)
            print_error "Unsupported operating system: $os"
            exit 1
            ;;
    esac
}

detect_shell() {
    local shell_name=$(basename "$SHELL")
    case $shell_name in
        zsh)
            SHELL_RC="$HOME/.zshrc"
            ;;
        bash)
            SHELL_RC="$HOME/.bashrc"
            ;;
        fish)
            SHELL_RC="$HOME/.config/fish/config.fish"
            ;;
        *)
            print_warning "Unknown shell: $shell_name. Will try to use .zshrc"
            SHELL_RC="$HOME/.zshrc"
            ;;
    esac
    
    print_status "Detected shell: $shell_name"
    print_status "Using shell RC: $SHELL_RC"
}

get_latest_version() {
    local version=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
    if [ -z "$version" ]; then
        print_error "Failed to get latest version from GitHub API"
        exit 1
    fi
    echo "$version"
}

install_binary() {
    local version=$1
    local platform=$2
    local download_url="https://github.com/$REPO/releases/download/$version/srs-$platform-$version"
    local binary_path="$INSTALL_DIR/$BINARY_NAME"
    
    print_status "Downloading SRS $version for $platform..."
    
    mkdir -p "$INSTALL_DIR"
    
    if ! curl -L -o "$binary_path" "$download_url"; then
        print_error "Failed to download binary from $download_url"
        exit 1
    fi
    
    chmod +x "$binary_path"
    
    print_success "Binary installed to $binary_path"
}

setup_path() {
    local shell_rc=$1
    
    # Check if INSTALL_DIR is already in PATH
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        print_success "SRS directory already in PATH"
        return 0
    fi
    
    print_status "Adding SRS to PATH in $shell_rc..."
    
    # Add PATH export to shell RC file
    echo "" >> "$shell_rc"
    echo "# SRS - Secure Rust Storage" >> "$shell_rc"
    echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$shell_rc"
    
    print_success "PATH updated in $shell_rc"
    print_warning "Please run 'source $shell_rc' or restart your terminal to use SRS"
}

check_path() {
    if ! command -v "$BINARY_NAME" &> /dev/null; then
        print_warning "SRS binary not found in PATH"
        print_status "Please run 'source $SHELL_RC' or restart your terminal"
    else
        print_success "SRS is available in PATH"
    fi
}

check_dependencies() {
    if ! command -v curl &> /dev/null; then
        print_error "curl is required but not installed. Please install it first:"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            echo "brew install curl"
        else
            echo "sudo apt update && sudo apt install curl"
        fi
        exit 1
    fi
}

main() {
    print_status "Starting SRS installation..."
    
    check_dependencies
    
    local platform=$(detect_platform)
    detect_shell
    
    print_status "Detected platform: $platform"
    print_status "Detected shell RC: $SHELL_RC"
    
    local version=$(get_latest_version)
    print_status "Latest version: $version"
    
    install_binary "$version" "$platform"
    
    setup_path "$SHELL_RC"
    
    check_path
    
    print_success "Installation completed!"
    print_status "You can now use 'srs' command after sourcing your shell RC file"
    print_status "Run: source $SHELL_RC"
}

main "$@"

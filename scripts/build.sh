#!/bin/bash

# Output colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Directory where builds will be saved
BUILD_DIR="target/releases"

# Function to display script usage
show_usage() {
    echo -e "Usage: $0 [options]"
    echo -e "Options:"
    echo -e "  --all\t\tBuild for all supported platforms"
    echo -e "  --linux\tBuild only for Linux (x86_64, ARM64, ARMv7)"
    echo -e "  --macos\tBuild only for macOS"
    echo -e "  --windows\tBuild only for Windows"
    echo -e "  --help\t\tShow this help message"
}

# Check if Rust is installed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}Error: Rust is not installed. Please install Rust first.${NC}"
        echo "Visit: https://rustup.rs/"
        exit 1
    fi
}

# Function to install target if needed
install_target() {
    local target=$1
    if ! rustup target list | grep -q "$target installed"; then
        echo -e "${YELLOW}Installing target $target...${NC}"
        rustup target add $target
    fi
}

# Function to build for a specific platform
build_for_target() {
    local target=$1
    local name=$2
    
    echo -e "${BLUE}Building for $name...${NC}"
    install_target $target
    
    cargo build --release --target $target || {
        echo -e "${RED}Build failed for $target${NC}"
        return 1
    }
    
    # Create releases directory if it doesn't exist
    mkdir -p $BUILD_DIR
    
    # Copy binary to releases directory
    local binary_name="chain-db-v2"
    if [[ $target == *"windows"* ]]; then
        binary_name="chain-db-v2.exe"
    fi
    
    cp "target/$target/release/$binary_name" "$BUILD_DIR/chain-db-v2-$name"
    echo -e "${GREEN}✓ Build completed for $name${NC}"
}

# Main build function
main() {
    local build_all=false
    local build_linux=false
    local build_macos=false
    local build_windows=false

    # If no arguments provided, show help
    if [ $# -eq 0 ]; then
        show_usage
        exit 0
    fi

    # Process arguments
    for arg in "$@"; do
        case $arg in
            --all)
                build_all=true
                ;;
            --linux)
                build_linux=true
                ;;
            --macos)
                build_macos=true
                ;;
            --windows)
                build_windows=true
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                echo -e "${RED}Invalid option: $arg${NC}"
                show_usage
                exit 1
                ;;
        esac
    done

    # Check if Rust is installed
    check_rust

    # Create build directory if it doesn't exist
    mkdir -p $BUILD_DIR

    # Build for selected platforms
    if [ "$build_all" = true ] || [ "$build_linux" = true ]; then
        build_for_target "x86_64-unknown-linux-gnu" "linux-amd64"
        build_for_target "aarch64-unknown-linux-gnu" "linux-arm64"
        build_for_target "armv7-unknown-linux-gnueabihf" "linux-armv7"
    fi

    if [ "$build_all" = true ] || [ "$build_macos" = true ]; then
        build_for_target "x86_64-apple-darwin" "macos-amd64"
    fi

    if [ "$build_all" = true ] || [ "$build_windows" = true ]; then
        build_for_target "x86_64-pc-windows-msvc" "windows-amd64.exe"
    fi

    echo -e "\n${GREEN}✓ Build process completed!${NC}"
    echo -e "${YELLOW}Binaries are available in: $BUILD_DIR${NC}"
}

# Run main function with all arguments
main "$@" 
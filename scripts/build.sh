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

# Function to check if we're on macOS
is_macos() {
    [[ "$(uname)" == "Darwin" ]]
}

# Function to check if we're on Linux
is_linux() {
    [[ "$(uname)" == "Linux" ]]
}

# Function to check for cross-compilation dependencies
check_cross_dependencies() {
    local target=$1
    
    # Check for ARM Linux targets
    if [[ $target == "aarch64-unknown-linux-gnu" ]] || [[ $target == "armv7-unknown-linux-gnueabihf" ]]; then
        if is_macos; then
            echo -e "${YELLOW}Warning: Cross-compiling for ARM Linux from macOS requires additional setup.${NC}"
            echo -e "Consider using 'cross' tool: https://github.com/cross-rs/cross"
            echo -e "Or use Docker with a Linux environment for ARM compilation."
            echo -e "Alternatively, use GitHub Actions for automated builds."
            return 1
        elif is_linux; then
            if ! dpkg -l | grep -q "gcc-aarch64-linux-gnu\|gcc-arm-linux-gnueabihf"; then
                echo -e "${YELLOW}Installing cross-compilation tools for ARM...${NC}"
                sudo apt-get update
                sudo apt-get install -y gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf
                if [ $? -ne 0 ]; then
                    echo -e "${RED}Failed to install cross-compilation tools.${NC}"
                    echo -e "Try manually: sudo apt-get install gcc-aarch64-linux-gnu gcc-arm-linux-gnueabihf"
                    return 1
                fi
            fi
        fi
    fi
    
    # Check for Windows target on non-Windows systems
    if [[ $target == "x86_64-pc-windows-msvc" ]] && ! [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
        if is_macos; then
            echo -e "${YELLOW}Warning: Cross-compiling for Windows from macOS requires additional setup.${NC}"
            echo -e "Consider installing 'mingw-w64' via Homebrew:"
            echo -e "  brew install mingw-w64"
            echo -e "Or use GitHub Actions for automated builds."
            return 1
        elif is_linux; then
            echo -e "${YELLOW}Warning: Cross-compiling for Windows from Linux requires additional setup.${NC}"
            echo -e "Consider installing MinGW:"
            echo -e "  sudo apt-get install mingw-w64"
            echo -e "Or use GitHub Actions for automated builds."
            return 1
        fi
    fi
    
    return 0
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
    
    echo -e "${BLUE}Building for $target ($name)...${NC}"
    
    # Install target
    install_target $target
    
    # Check dependencies for cross-compilation
    check_cross_dependencies $target
    if [ $? -ne 0 ]; then
        echo -e "${YELLOW}Skipping build for $target due to missing dependencies.${NC}"
        return 0
    fi
    
    # Build
    echo -e "${BLUE}Compiling...${NC}"
    cargo build --release --target $target
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}Build failed for $target${NC}"
        return 1
    fi
    
    # Create releases directory if it doesn't exist
    mkdir -p $BUILD_DIR
    
    # Copy binary to releases directory
    local binary_name="chain-db"
    if [[ $target == *"windows"* ]]; then
        binary_name="chain-db.exe"
    fi
    
    cp "target/$target/release/$binary_name" "$BUILD_DIR/chain-db-$name"
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
    
    echo -e "${BLUE}Starting build process...${NC}"
    echo -e "${YELLOW}Note: Cross-compilation may require additional dependencies.${NC}"
    echo -e "${YELLOW}If builds fail, consider using GitHub Actions for automated builds.${NC}\n"

    # Build for selected platforms
    if [ "$build_all" = true ] || [ "$build_linux" = true ]; then
        build_for_target "x86_64-unknown-linux-gnu" "linux-amd64"
        build_for_target "aarch64-unknown-linux-gnu" "linux-arm64"
        build_for_target "armv7-unknown-linux-gnueabihf" "linux-armv7"
    fi

    if [ "$build_all" = true ] || [ "$build_macos" = true ]; then
        if is_macos; then
            build_for_target "x86_64-apple-darwin" "macos-amd64"
        else
            echo -e "${YELLOW}Skipping macOS build: Can only build for macOS on macOS.${NC}"
        fi
    fi

    if [ "$build_all" = true ] || [ "$build_windows" = true ]; then
        build_for_target "x86_64-pc-windows-msvc" "windows-amd64.exe"
    fi

    echo -e "\n${GREEN}✓ Build process completed!${NC}"
    echo -e "${YELLOW}Binaries are available in: $BUILD_DIR${NC}"
    echo -e "${BLUE}Note: For cross-platform builds, consider using GitHub Actions.${NC}"
    echo -e "${BLUE}Your GitHub workflow is already set up to build for all platforms when you create a release.${NC}"
}

# Run main function with all arguments
main "$@" 
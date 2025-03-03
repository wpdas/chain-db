#!/bin/bash

# Output colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Starting ChainDB tests...${NC}\n"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust is not installed. Please install Rust first.${NC}"
    echo "Visit: https://rustup.rs/"
    exit 1
fi

# Run tests with different configurations
echo -e "${YELLOW}Running unit tests...${NC}"
cargo test --quiet || { echo -e "${RED}Unit tests failed${NC}"; exit 1; }

echo -e "${YELLOW}Running tests with all features...${NC}"
cargo test --all-features --quiet || { echo -e "${RED}Feature tests failed${NC}"; exit 1; }

# echo -e "${YELLOW}Running optimized tests...${NC}"
# cargo test --release --quiet || { echo -e "${RED}Release tests failed${NC}"; exit 1; }

# If we got here, all tests passed
echo -e "\n${GREEN}✓ All tests passed successfully!${NC}" 
#!/bin/bash
# VstKit Build Script
#
# ⚠️  DEPRECATION NOTICE ⚠️
# This script is deprecated and will be removed in a future release.
# Please use the new Rust-based xtask build system instead:
#
#   cd engine && cargo xtask --help
#
# Migration guide:
#   ./scripts/build.sh --clean       →  cargo xtask clean
#   ./scripts/build.sh --release     →  cargo xtask bundle
#   ./scripts/build.sh --debug       →  cargo xtask bundle --debug
#   ./scripts/build.sh --install     →  cargo xtask install
#   ./scripts/build.sh --test        →  cargo xtask test
#   ./scripts/build.sh --au          →  cargo xtask au
#   ./scripts/build.sh --all         →  cargo xtask all
#
# 
# This script builds and optionally installs the VstKit audio plugin
# for all supported formats: VST3, CLAP, and AU (macOS only).
#
# Usage:
#   ./scripts/build.sh [options]
#
# Options:
#   --clean       Clean all build artifacts before building
#   --release     Build in release mode (default)
#   --debug       Build in debug mode
#   --install     Install plugins to system directories
#   --test        Run unit tests before building
#   --au          Build AU wrapper (macOS only, requires CMake)
#   --all         Build everything (equivalent to --test --au --install)
#   -h, --help    Show this help message
#
# Examples:
#   ./scripts/build.sh --clean --install     # Clean build and install
#   ./scripts/build.sh --all                 # Full build with tests and install
#   ./scripts/build.sh --debug               # Debug build only

set -e

# Print deprecation warning
echo ""
echo -e "\033[1;33m⚠️  DEPRECATION WARNING ⚠️\033[0m"
echo -e "\033[1;33mThis script is deprecated. Please use:\033[0m"
echo -e "  \033[0;36mcd engine && cargo xtask --help\033[0m"
echo ""
sleep 1

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ENGINE_DIR="${PROJECT_ROOT}/engine"
AU_WRAPPER_DIR="${PROJECT_ROOT}/packaging/macos/au-wrapper"

# Default options
CLEAN=false
BUILD_MODE="--release"
INSTALL=false
RUN_TESTS=false
BUILD_AU=false

# Plugin name (matches Cargo package name)
PLUGIN_NAME="vstkit"
PLUGIN_DISPLAY_NAME="VstKit"

# Installation directories (macOS)
VST3_INSTALL_DIR="${HOME}/Library/Audio/Plug-Ins/VST3"
CLAP_INSTALL_DIR="${HOME}/Library/Audio/Plug-Ins/CLAP"
AU_INSTALL_DIR="${HOME}/Library/Audio/Plug-Ins/Components"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN=true
            shift
            ;;
        --release)
            BUILD_MODE="--release"
            shift
            ;;
        --debug)
            BUILD_MODE=""
            shift
            ;;
        --install)
            INSTALL=true
            shift
            ;;
        --test)
            RUN_TESTS=true
            shift
            ;;
        --au)
            BUILD_AU=true
            shift
            ;;
        --all)
            RUN_TESTS=true
            BUILD_AU=true
            INSTALL=true
            shift
            ;;
        -h|--help)
            head -30 "$0" | tail -28
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Print header
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  VstKit Build Script${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo -e "${YELLOW}Cleaning build artifacts...${NC}"
    cd "${ENGINE_DIR}"
    cargo clean
    rm -rf target/bundled
    
    if [ -d "${AU_WRAPPER_DIR}/build" ]; then
        rm -rf "${AU_WRAPPER_DIR}/build"
    fi
    
    # Remove installed plugins
    rm -rf "${VST3_INSTALL_DIR}/${PLUGIN_NAME}.vst3"
    rm -rf "${CLAP_INSTALL_DIR}/${PLUGIN_NAME}.clap"
    rm -rf "${AU_INSTALL_DIR}/${PLUGIN_DISPLAY_NAME}.component"
    
    echo -e "${GREEN}Clean complete.${NC}"
    echo ""
fi

# Run tests if requested
if [ "$RUN_TESTS" = true ]; then
    echo -e "${YELLOW}Running unit tests...${NC}"
    cd "${ENGINE_DIR}"
    cargo test -p dsp -p protocol
    echo -e "${GREEN}Tests passed.${NC}"
    echo ""
fi

# Build VST3 and CLAP plugins
echo -e "${YELLOW}Building VST3 and CLAP plugins...${NC}"
cd "${ENGINE_DIR}"
cargo xtask bundle "${PLUGIN_NAME}" ${BUILD_MODE}
echo -e "${GREEN}VST3 and CLAP build complete.${NC}"
echo ""

# Build AU wrapper (macOS only)
if [ "$BUILD_AU" = true ]; then
    if [[ "$(uname)" == "Darwin" ]]; then
        if command -v cmake &> /dev/null; then
            echo -e "${YELLOW}Building AU wrapper...${NC}"
            cd "${AU_WRAPPER_DIR}"
            cmake -B build
            cmake --build build
            echo -e "${GREEN}AU build complete.${NC}"
            echo ""
        else
            echo -e "${RED}CMake not found. AU wrapper requires CMake.${NC}"
            echo -e "${YELLOW}Install with: brew install cmake${NC}"
            echo ""
        fi
    else
        echo -e "${YELLOW}Skipping AU build (macOS only).${NC}"
        echo ""
    fi
fi

# Install if requested
if [ "$INSTALL" = true ]; then
    echo -e "${YELLOW}Installing plugins...${NC}"
    
    # Create installation directories if they don't exist
    mkdir -p "${VST3_INSTALL_DIR}"
    mkdir -p "${CLAP_INSTALL_DIR}"
    
    # Install VST3
    if [ -d "${ENGINE_DIR}/target/bundled/${PLUGIN_NAME}.vst3" ]; then
        rm -rf "${VST3_INSTALL_DIR}/${PLUGIN_NAME}.vst3"
        cp -R "${ENGINE_DIR}/target/bundled/${PLUGIN_NAME}.vst3" "${VST3_INSTALL_DIR}/"
        echo -e "  ${GREEN}✓${NC} Installed ${PLUGIN_NAME}.vst3 to ${VST3_INSTALL_DIR}/"
    fi
    
    # Install CLAP
    if [ -d "${ENGINE_DIR}/target/bundled/${PLUGIN_NAME}.clap" ]; then
        rm -rf "${CLAP_INSTALL_DIR}/${PLUGIN_NAME}.clap"
        cp -R "${ENGINE_DIR}/target/bundled/${PLUGIN_NAME}.clap" "${CLAP_INSTALL_DIR}/"
        echo -e "  ${GREEN}✓${NC} Installed ${PLUGIN_NAME}.clap to ${CLAP_INSTALL_DIR}/"
    fi
    
    # Install AU (macOS only)
    if [[ "$(uname)" == "Darwin" ]]; then
        mkdir -p "${AU_INSTALL_DIR}"
        
        AU_BUNDLE="${AU_WRAPPER_DIR}/build/${PLUGIN_DISPLAY_NAME}.component"
        if [ -d "${AU_BUNDLE}" ]; then
            rm -rf "${AU_INSTALL_DIR}/${PLUGIN_DISPLAY_NAME}.component"
            cp -R "${AU_BUNDLE}" "${AU_INSTALL_DIR}/"
            echo -e "  ${GREEN}✓${NC} Installed ${PLUGIN_DISPLAY_NAME}.component to ${AU_INSTALL_DIR}/"
            
            # Refresh macOS Audio Unit cache
            echo -e "${YELLOW}Refreshing macOS AU cache...${NC}"
            killall -9 AudioComponentRegistrar 2>/dev/null || true
        fi
    fi
    
    echo -e "${GREEN}Installation complete.${NC}"
    echo ""
fi

# Print summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Build Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "Build outputs:"
if [ -d "${ENGINE_DIR}/target/bundled/${PLUGIN_NAME}.vst3" ]; then
    echo -e "  ${GREEN}✓${NC} VST3: ${ENGINE_DIR}/target/bundled/${PLUGIN_NAME}.vst3"
fi
if [ -d "${ENGINE_DIR}/target/bundled/${PLUGIN_NAME}.clap" ]; then
    echo -e "  ${GREEN}✓${NC} CLAP: ${ENGINE_DIR}/target/bundled/${PLUGIN_NAME}.clap"
fi
if [ -d "${AU_WRAPPER_DIR}/build/${PLUGIN_DISPLAY_NAME}.component" ]; then
    echo -e "  ${GREEN}✓${NC} AU:   ${AU_WRAPPER_DIR}/build/${PLUGIN_DISPLAY_NAME}.component"
fi
echo ""

if [ "$INSTALL" = true ]; then
    echo -e "Installed to:"
    [ -d "${VST3_INSTALL_DIR}/${PLUGIN_NAME}.vst3" ] && echo -e "  ${GREEN}✓${NC} ${VST3_INSTALL_DIR}/${PLUGIN_NAME}.vst3"
    [ -d "${CLAP_INSTALL_DIR}/${PLUGIN_NAME}.clap" ] && echo -e "  ${GREEN}✓${NC} ${CLAP_INSTALL_DIR}/${PLUGIN_NAME}.clap"
    [ -d "${AU_INSTALL_DIR}/${PLUGIN_DISPLAY_NAME}.component" ] && echo -e "  ${GREEN}✓${NC} ${AU_INSTALL_DIR}/${PLUGIN_DISPLAY_NAME}.component"
    echo ""
fi

echo -e "${GREEN}Done!${NC}"

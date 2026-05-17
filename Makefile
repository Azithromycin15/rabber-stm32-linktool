# Makefile for rabber-stm32-linktool
# This Makefile provides build targets for the Rust application, Go plugin loader,
# and cross-compilation for Windows releases.

# Extract version from Cargo.toml
VERSION := $(shell grep '^version' Cargo.toml | cut -d '"' -f2)

# Auto-detect host OS for release packaging
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    HOST_OS := macos
else
    HOST_OS := linux
endif

# Windows cross-compilation target (requires mingw-w64 toolchain)
WIN_TARGET ?= x86_64-pc-windows-gnu

# Go plugin loader directory and binary
GO_DIR := plugin-loader
PLUGIN_LOADER_BIN := $(GO_DIR)/plugin-loader
PLUGIN_LOADER_WIN_BIN := $(GO_DIR)/plugin-loader.exe

# Python interpreter
PYTHON := python3

# Phony targets (not files)
.PHONY: all build rust plugin-loader plugin-loader-win check run-plugin release release-win release-all clean

# Default target
all: build

# Build all components
build: rust plugin-loader

# Build Rust application
rust:
	cargo build --release

# Build Go plugin loader
plugin-loader:
	cd $(GO_DIR) && go build -o plugin-loader
	chmod +x $(PLUGIN_LOADER_BIN)

plugin-loader-win:
	cd $(GO_DIR) && GOOS=windows GOARCH=amd64 go build -o plugin-loader.exe

# Check Rust code without building
check:
	cargo check

# Run plugin loader to list components
run-plugin: plugin-loader
	$(PLUGIN_LOADER_BIN) --manifest plugins/manifest.yaml --list

# Create release build for current platform
release: build
	mkdir -p release
	cp target/release/rabber-stm32-linktool release/rabber-$(VERSION)-$(HOST_OS)
	cp $(PLUGIN_LOADER_BIN) release/plugin-loader

# Build Windows cross-compilation artifacts (requires `rustup target add x86_64-pc-windows-gnu` + mingw-w64)
release-win:
	@if ! rustup target list --installed 2>/dev/null | grep -q $(WIN_TARGET); then \
		echo "[!] Rust target $(WIN_TARGET) not installed."; \
		echo "    Install with: rustup target add $(WIN_TARGET)"; \
		echo "    Also requires mingw-w64 linker: brew install mingw-w64"; \
		exit 1; \
	fi
	cd $(GO_DIR) && GOOS=windows GOARCH=amd64 go build -o plugin-loader.exe
	cargo build --release --target $(WIN_TARGET)
	cp target/$(WIN_TARGET)/release/rabber-stm32-linktool.exe release/rabber-$(VERSION)-win64.exe
	cp $(PLUGIN_LOADER_WIN_BIN) release/plugin-loader.exe

# Full release: current platform + Windows
release-all: release release-win

# Clean build artifacts
clean:
	cargo clean
	rm -f $(PLUGIN_LOADER_BIN)

# Makefile for rabber-stm32-linktool
# This Makefile provides build targets for the Rust application, Go plugin loader,
# and cross-compilation for Windows releases.

# Extract version from Cargo.toml
VERSION := $(shell grep '^version' Cargo.toml | cut -d '"' -f2)

# Default Rust targets
RUST_TARGET ?= x86_64-unknown-linux-gnu
WIN_TARGET ?= x86_64-pc-windows-gnu

# Go plugin loader directory and binary
GO_DIR := plugin-loader
PLUGIN_LOADER_BIN := $(GO_DIR)/plugin-loader

# Python interpreter
PYTHON := python3

# Phony targets (not files)
.PHONY: all build rust plugin-loader check run-plugin release clean

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

# Check Rust code without building
check:
	cargo check

# Run plugin loader to list components
run-plugin: plugin-loader
	$(PLUGIN_LOADER_BIN) --manifest plugins/manifest.yaml --list

# Create release builds for Linux and Windows
release: build
	mkdir -p release
	cp target/release/rabber-stm32-linktool release/rabber-stm32-linktool-$(VERSION)-linux
	cargo build --release --target $(WIN_TARGET)
	cp target/$(WIN_TARGET)/release/rabber-stm32-linktool.exe release/rabber-stm32-linktool-$(VERSION)-win64.exe

# Clean build artifacts
clean:
	cargo clean
	rm -f $(PLUGIN_LOADER_BIN)

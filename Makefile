# Makefile for rabber-stm32-linktool

VERSION := $(shell grep '^version' Cargo.toml | cut -d '"' -f2)
RUST_TARGET ?= x86_64-unknown-linux-gnu
WIN_TARGET ?= x86_64-pc-windows-gnu
GO_DIR := plugin-loader
PLUGIN_LOADER_BIN := $(GO_DIR)/plugin-loader
PYTHON := python3

.PHONY: all build rust plugin-loader check run-plugin release clean

all: build

build: rust plugin-loader

rust:
	cargo build --release

plugin-loader:
	cd $(GO_DIR) && go build -o plugin-loader

check:
	cargo check

run-plugin: plugin-loader
	$(PLUGIN_LOADER_BIN) --manifest plugins/manifest.yaml --list

release: build
	mkdir -p release
	cp target/release/rabber-stm32-linktool release/rabber-stm32-linktool-$(VERSION)-linux
	cargo build --release --target $(WIN_TARGET)
	cp target/$(WIN_TARGET)/release/rabber-stm32-linktool.exe release/rabber-stm32-linktool-$(VERSION)-win64.exe

clean:
	cargo clean
	rm -f $(PLUGIN_LOADER_BIN)

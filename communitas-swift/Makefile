# Makefile for Communitas Swift Bindings

ROOT_DIR := $(shell git rev-parse --show-toplevel)
BINDINGS_DIR := $(ROOT_DIR)/communitas-bindings
SWIFT_PKG_DIR := $(ROOT_DIR)/communitas-swift/CommunitasKit
TARGET_DIR := $(ROOT_DIR)/target

# Architectures
ARCH_MACOS := aarch64-apple-darwin
ARCH_IOS := aarch64-apple-ios
ARCH_IOS_SIM := aarch64-apple-ios-sim

.PHONY: all build-rust generate-bindings clean init-targets

all: init-targets build-rust generate-bindings

init-targets:
	rustup target add $(ARCH_MACOS) $(ARCH_IOS) $(ARCH_IOS_SIM)

build-rust:
	@echo "Building Rust bindings for macOS..."
	cd $(ROOT_DIR) && cargo build --offline -p communitas-bindings --target $(ARCH_MACOS)
	# Uncomment for iOS builds (requires SDKs)
	# cd $(ROOT_DIR) && cargo build --offline -p communitas-bindings --target $(ARCH_IOS)
	# cd $(ROOT_DIR) && cargo build --offline -p communitas-bindings --target $(ARCH_IOS_SIM)

generate-bindings: build-rust
	@echo "Generating Swift bindings..."
	# We generate bindings using the macOS host library for introspection
	cd $(ROOT_DIR) && cargo run -p communitas-bindings --features=uniffi/cli --bin uniffi-bindgen generate \
		--library $(TARGET_DIR)/$(ARCH_MACOS)/debug/libcommunitas_bindings.dylib \
		--language swift \
		--out-dir $(SWIFT_PKG_DIR)/Sources/CommunitasKit

clean:
	rm -rf $(SWIFT_PKG_DIR)/Sources/CommunitasKit/communitas_bindings.swift
	cd $(ROOT_DIR) && cargo clean -p communitas-bindings

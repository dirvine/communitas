SHELL := /bin/bash

.PHONY: dev:testnet lint build:tauri test

dev:testnet:
	@echo "Launching 5 local IPv4-only nodes with churn+loss (simulated)"
	./deploy-testnet.sh --nodes 5 --ipv4-only --simulate-loss 0.15 --simulate-churn

lint:
	cargo fmt --all
	cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used -W clippy::pedantic

build:tauri:
	@echo "Reproducible Tauri build (macOS)"
	cargo build --release -p communitas-tauri

test:
	cargo test --workspace -- --nocapture


SHELL := /bin/bash

.PHONY: dev-testnet lint test

dev-testnet:
	@echo "Launching 5 local IPv4-only nodes with churn+loss (simulated)"
	./deploy-testnet.sh --nodes 5 --ipv4-only --simulate-loss 0.15 --simulate-churn

lint:
	cargo fmt --all
	cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used

test:
	cargo test --workspace -- --nocapture

.PHONY: dioxus-check
dioxus-check:
	./scripts/install_dx.sh
	cd communitas-dioxus && dx check --platform desktop

.PHONY: dioxus-bundle
dioxus-bundle:
	./scripts/ci_dx_bundle.sh

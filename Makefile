# Building packwing in one word: `make debug` runs a debug build, `make release` a
# release one; `make` alone lists the rest. Needs Rust (rustup) and, on Linux, the
# development libraries of the window systems it builds for.
#
# On Linux the window builds for Wayland by default:
#   make release FEATURES=wayland,x11

# rustup's tools, for shells that have not sourced ~/.cargo/env. Make 3.81 (macOS)
# looks commands up on the PATH it started with, so cargo is named by path too.
CARGO_BIN := $(HOME)/.cargo/bin
export PATH := $(CARGO_BIN):$(PATH)
CARGO ?= $(if $(wildcard $(CARGO_BIN)/cargo),$(CARGO_BIN)/cargo,cargo)

# The window systems to build for, and any other cargo features, like x11.
FEATURES ?= wayland
APP := -p packwing --no-default-features --features $(FEATURES)
ALL := --workspace --no-default-features --features $(FEATURES)

.DEFAULT_GOAL := help
.PHONY: help debug release build build-debug test test-window lint fmt clean

help: ## List the targets
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  make %-14s %s\n", $$1, $$2}'

debug: ## Build and run the debug version: quicker to build, slower to run
	$(CARGO) run $(APP)

release: ## Build and run the release version
	$(CARGO) run --release $(APP)

build-debug: ## Build the debug version: target/debug/packwing
	$(CARGO) build $(APP)

build: ## Build the release version: target/release/packwing
	$(CARGO) build --release $(APP)

test: ## Run every test, without the ones that draw the window
	$(CARGO) test $(ALL)

test-window: ## Run every test, drawing every screen to target/shots with no display (Linux)
	$(CARGO) test $(ALL),shots

lint: ## Check the format and run clippy
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy $(ALL) --all-targets -- -D warnings

fmt: ## Format the code
	$(CARGO) fmt --all

clean: ## Remove what was built
	$(CARGO) clean

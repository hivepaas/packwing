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

# Throwaway servers to try storages on, from the host only. Their ports are the ones
# in crates/remote/tests/servers.rs.
S3_PORT ?= 18333
WEBDAV_PORT ?= 18335
SFTP_PORT ?= 18322
S3_SERVER := packwing-test-s3
WEBDAV_SERVER := packwing-test-webdav
SFTP_SERVER := packwing-test-sftp
# The test server's key and known_hosts, for the tests.
SFTP_FILES := $(CURDIR)/target/test-sftp
# Takes the test server's lines out of known_hosts, and nothing else.
SFTP_UNTRUST := touch $(HOME)/.ssh/known_hosts; \
	lines=$$(grep -vF '[127.0.0.1]:$(SFTP_PORT) ' $(HOME)/.ssh/known_hosts); \
	printf '%s\n' "$$lines" >$(HOME)/.ssh/known_hosts

.DEFAULT_GOAL := help
.PHONY: help debug release build build-debug cli cli-slim test test-window test-ui lint fmt clean \
	s3-start s3-stop webdav-start webdav-stop sftp-start sftp-stop

help: ## List the targets
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  make %-14s %s\n", $$1, $$2}'

debug: ## Build and run the debug version: quicker to build, slower to run
	$(CARGO) run $(APP)

release: ## Build and run the release version
	$(CARGO) run --release $(APP)

build-debug: ## Build the debug version: target/debug/packwing
	$(CARGO) build $(APP)

cli: ## Build the command line, with no window: target/release/pkw
	$(CARGO) build --release -p packwing-cli

cli-slim: ## Build pkw without S3, SFTP and WebDAV, a third of the size
	$(CARGO) build --release -p packwing-cli --no-default-features

build: ## Build the release version: target/release/packwing
	$(CARGO) build --release $(APP)

test: ## Run every test, without the ones that draw the window
	$(CARGO) test $(ALL)
	@# $(ALL) builds pkw without its default `remote`: once more with it.
	$(CARGO) test -p packwing-cli

test-window: ## Run every test, drawing every screen to target/shots with no display (Linux)
	$(CARGO) test $(ALL),shots

test-ui: ## Run the UI scenarios, with no display: the real window, driven as a user does
	$(CARGO) test $(APP),ui-tests ui_tests::$(FILTER)

lint: ## Check the format and run clippy
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy $(ALL) --all-targets -- -D warnings
	$(CARGO) clippy $(APP),ui-tests --all-targets -- -D warnings
	$(CARGO) clippy -p packwing-cli --all-targets -- -D warnings

fmt: ## Format the code
	$(CARGO) fmt --all

clean: ## Remove what was built
	$(CARGO) clean

s3-start: ## Start a test S3 server in Docker, with a bucket: test
	@docker inspect $(S3_SERVER) >/dev/null 2>&1 || docker run -d --rm --name $(S3_SERVER) \
		-p 127.0.0.1:$(S3_PORT):8333 \
		-e AWS_ACCESS_KEY_ID=packwing -e AWS_SECRET_ACCESS_KEY=packwing \
		chrislusf/seaweedfs:4.47 \
		server -dir=/data -s3 -s3.port=8333 >/dev/null
	@for i in $$(seq 60); do \
		case $$(curl -s -o /dev/null -w '%{http_code}' -X PUT --aws-sigv4 aws:amz:us-east-1:s3 \
			-u packwing:packwing http://127.0.0.1:$(S3_PORT)/test) in \
			200|409) echo "S3: endpoint http://127.0.0.1:$(S3_PORT), bucket test, access key packwing, secret key packwing"; exit 0;; \
		esac; \
		sleep 1; \
	done; \
	echo "S3 did not answer; see docker logs $(S3_SERVER)"; exit 1

s3-stop: ## Stop the test S3 server and throw away what it holds
	@docker rm -fv $(S3_SERVER) >/dev/null 2>&1 || true

webdav-start: ## Start a test WebDAV server in Docker
	@docker inspect $(WEBDAV_SERVER) >/dev/null 2>&1 || docker run -d --rm --name $(WEBDAV_SERVER) \
		-p 127.0.0.1:$(WEBDAV_PORT):8080 rclone/rclone:1.75.2 \
		serve webdav /data --addr :8080 --user packwing --pass packwing >/dev/null
	@for i in $$(seq 60); do \
		if curl -s -o /dev/null http://127.0.0.1:$(WEBDAV_PORT)/; then \
			echo "WebDAV: address http://127.0.0.1:$(WEBDAV_PORT), user packwing, password packwing"; exit 0; \
		fi; \
		sleep 1; \
	done; \
	echo "WebDAV did not answer; see docker logs $(WEBDAV_SERVER)"; exit 1

webdav-stop: ## Stop the test WebDAV server and throw away what it holds
	@docker rm -fv $(WEBDAV_SERVER) >/dev/null 2>&1 || true

sftp-start: ## Start a test SFTP server in Docker, and trust it in ~/.ssh/known_hosts
	@docker inspect $(SFTP_SERVER) >/dev/null 2>&1 || { \
		rm -rf $(SFTP_FILES) && mkdir -p $(SFTP_FILES) && \
		ssh-keygen -q -t ed25519 -N '' -C packwing-test -f $(SFTP_FILES)/key && \
		docker run -d --rm --name $(SFTP_SERVER) -p 127.0.0.1:$(SFTP_PORT):2222 \
			-e USER_NAME=tester -e USER_PASSWORD=packwing -e PASSWORD_ACCESS=true \
			-e PUBLIC_KEY="$$(cat $(SFTP_FILES)/key.pub)" \
			linuxserver/openssh-server:version-10.3_p1-r1 >/dev/null; \
	}
	@for i in $$(seq 60); do \
		ssh-keyscan -q -p $(SFTP_PORT) 127.0.0.1 2>/dev/null | grep -v '^#' >$(SFTP_FILES)/known_hosts; \
		if [ -s $(SFTP_FILES)/known_hosts ]; then \
			mkdir -p $(HOME)/.ssh; \
			$(SFTP_UNTRUST); \
			cat $(SFTP_FILES)/known_hosts >>$(HOME)/.ssh/known_hosts; \
			echo "SFTP: host tester@127.0.0.1:$(SFTP_PORT), password packwing"; exit 0; \
		fi; \
		sleep 1; \
	done; \
	echo "SFTP did not answer; see docker logs $(SFTP_SERVER)"; exit 1

sftp-stop: ## Stop the test SFTP server, throw away what it holds, and stop trusting it
	@docker rm -fv $(SFTP_SERVER) >/dev/null 2>&1 || true
	@$(SFTP_UNTRUST)
	@rm -rf $(SFTP_FILES)

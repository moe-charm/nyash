# Nyash selfhosting-dev quick targets

.PHONY: build build-release run-minimal smoke-core smoke-selfhost bootstrap roundtrip clean quick fmt lint dep-tree \
	smoke-quick smoke-quick-filter smoke-integration \
	artifacts-nyash artifacts-apps artifacts-all artifacts-clean \
	artifacts-move artifacts-unlink artifacts-restore

build:
	cargo build --features cranelift-jit

build-release:
	cargo build --release --features cranelift-jit

run-minimal:
	NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend vm apps/selfhost-minimal/main.nyash

smoke-core:
	bash tools/jit_smoke.sh

smoke-selfhost:
	bash tools/selfhost_vm_smoke.sh

bootstrap:
	bash tools/bootstrap_selfhost_smoke.sh

roundtrip:
	bash tools/ny_roundtrip_smoke.sh

clean:
	cargo clean

quick: build-release smoke-selfhost

# --- v2 smokes shortcuts ---
smoke-quick:
	bash tools/smokes/v2/run.sh --profile quick

# Usage: make smoke-quick-filter FILTER="json_*"
smoke-quick-filter:
	bash tools/smokes/v2/run.sh --profile quick --filter "$(FILTER)"

smoke-integration:
	bash tools/smokes/v2/run.sh --profile integration

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings || true

# --- Self-hosting dev helpers (Ny-only inner loop) ---
dev:
	./tools/dev_selfhost_loop.sh --std -v -- --using-path apps/selfhost:apps apps/selfhost-minimal/main.nyash

dev-watch:
	./tools/dev_selfhost_loop.sh --watch --std -v -- --using-path apps/selfhost:apps apps/selfhost-minimal/main.nyash


# --- Self-host dependency tree (Ny-only) ---
dep-tree:
	cargo build --release
	./target/release/nyash --run-task dep_tree

# --- Artifacts (opt-in, non-destructive by default) ---
ARTIFACTS_DIR ?= artifacts

artifacts-nyash:
	@mkdir -p $(ARTIFACTS_DIR)/bin
	@if [ -f ./target/release/nyash ]; then \
	  cp -p ./target/release/nyash $(ARTIFACTS_DIR)/bin/nyash; \
	  echo "[artifacts] Copied nyash -> $(ARTIFACTS_DIR)/bin/nyash"; \
	else \
	  echo "[artifacts] nyash not built (run 'make build-release')"; \
	fi

# Collect root-level app/app_* binaries to artifacts/apps (copy; does not delete originals)
artifacts-apps:
	@mkdir -p $(ARTIFACTS_DIR)/apps
	@sh -c 'set -e; for f in app app_*; do \
	  if [ -f "$$f" ]; then cp -p "$$f" $(ARTIFACTS_DIR)/apps/; fi; \
	done; echo "[artifacts] Collected app* -> $(ARTIFACTS_DIR)/apps (if any)"'

artifacts-all: artifacts-nyash artifacts-apps

artifacts-clean:
	rm -rf $(ARTIFACTS_DIR)

# --- Physical move of root app/app_* to artifacts/apps (with symlinks) ---
artifacts-move:
	@bash tools/move_root_apps.sh move

artifacts-unlink:
	@bash tools/move_root_apps.sh unlink

artifacts-restore:
	@bash tools/move_root_apps.sh restore

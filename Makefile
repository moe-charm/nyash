# Nyash selfhosting-dev quick targets

.PHONY: build build-release run-minimal smoke-core smoke-selfhost bootstrap roundtrip clean quick fmt lint dep-tree \
	smoke-quick smoke-quick-filter smoke-integration

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

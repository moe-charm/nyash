# Nyash selfhosting-dev quick targets

.PHONY: build build-release run-minimal smoke-core smoke-selfhost bootstrap roundtrip clean quick fmt lint

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

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings || true

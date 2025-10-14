# Nyash selfhosting-dev quick targets

.PHONY: build build-release run-minimal smoke-core smoke-selfhost bootstrap roundtrip clean quick fmt lint dep-tree \
	smoke-quick smoke-quick-filter smoke-integration \
	artifacts-nyash artifacts-apps artifacts-all artifacts-clean \
	artifacts-move artifacts-unlink artifacts-restore release freeze-linux freeze-win-gnu freeze-win-msvc

build:
	cargo build --features cranelift-jit

build-release:
	cargo build --release --features cranelift-jit

run-minimal:
	NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend vm apps/selfhost-minimal/main.hako

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
	./tools/lints/lint_indexof_two_args.sh || true

.PHONY: lint-ny
lint-ny:
	./tools/lints/lint_indexof_two_args.sh

# --- Self-hosting dev helpers (Ny-only inner loop) ---
dev:
	./tools/dev_selfhost_loop.sh --std -v -- --using-path apps/selfhost:apps apps/selfhost-minimal/main.hako

dev-watch:
	./tools/dev_selfhost_loop.sh --watch --std -v -- --using-path apps/selfhost:apps apps/selfhost-minimal/main.hako


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

# ========================================
# 🛡️ ルート保護コマンド（2025-09-30）
# ========================================

.PHONY: check-root clean-root

# ルートの不要ファイルをチェック
check-root:
	@echo "🔍 ルートディレクトリチェック中..."
	@bad_files=$$(ls -1 *.hako *.o *.err *.log *.tmp *.bak 2>/dev/null | wc -l); \
	if [ $$bad_files -gt 0 ]; then \
		echo "❌ ルートに不要なファイルが見つかりました:"; \
		ls -1 *.hako *.o *.err *.log *.tmp *.bak 2>/dev/null || true; \
		echo ""; \
		echo "実行: make clean-root"; \
		exit 1; \
	else \
		echo "✅ ルートはクリーンです"; \
	fi

# ルートの不要ファイルを自動削除
clean-root:
	@echo "🧹 ルートクリーンアップ中..."
	@rm -f *.hako *.o *.err *.log *.tmp *.bak 2>/dev/null || true
	@rm -f *_temp.* *_tmp.* commit_message*.txt 2>/dev/null || true
	@echo "✅ ルートクリーンアップ完了"

# ビルド前にルートチェック
build: check-root
	cargo build --release

# テスト前にルートチェック  
test: check-root
	cargo test

# --- Release packaging (frozen toolchain) ---
release:
	@echo "[release] packaging dist artifacts..."
	bash tools/release/package_dist.sh
	@echo "[release] manifest: dist/release.json"

.PHONY: release-sign
release-sign:
	@if [ -z "$$GPG_SIGN" ] || [ -z "$$GPG_KEY_ID" ]; then \
	  echo "Usage: GPG_SIGN=1 GPG_KEY_ID=<KEYID> make release-sign"; \
	  exit 2; \
	fi
	bash tools/release/sign_artifacts.sh

freeze-linux:
	@echo "[freeze-linux] ensure Ubuntu frozen binary is present"
	@./target/release/hakorune --backend mir --emit-mir-json build/frozen_ubuntu/mir/main.mir.json examples/simple_return.hako
	@bash tools/aot/emit_object_via_extern_c.sh build/frozen_ubuntu/mir/main.mir.json build/frozen_ubuntu/obj/main.o
	@bash tools/aot/link_with_clang.sh -o bin/hako-frozen-v1 build/frozen_ubuntu/obj/main.o --nyrt target/release/libhako_kernel.a || bash tools/aot/link_with_clang.sh -o bin/hako-frozen-v1 build/frozen_ubuntu/obj/main.o
	@mkdir -p dist && cp -f bin/hako-frozen-v1 dist/hako-frozen-v1-linux-x64

freeze-win-gnu:
	@echo "[freeze-win-gnu] ensure MinGW frozen binary is present"
	@bash tools/aot/windows/build_mingw_static.sh target/x86_64-pc-windows-gnu/release/libhako_kernel.a build/test_min.exe
	@mkdir -p dist && cp -f build/test_min.exe dist/hako-frozen-v1-win-x64-gnu.exe

freeze-win-msvc:
	@echo "[freeze-win-msvc] copy test_msvc.exe when present (Windows build)"
	@mkdir -p dist && [ -f build/test_msvc.exe ] && cp -f build/test_msvc.exe dist/hako-frozen-v1-win-x64-msvc.exe || true

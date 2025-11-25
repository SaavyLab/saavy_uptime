#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/apps/backend"

# Detect if we're in local dev (Rust already available) vs Deploy Button (need to bootstrap)
is_local_dev() {
	# If cargo and worker-build are both available, we're in local dev
	command -v cargo >/dev/null 2>&1 && command -v worker-build >/dev/null 2>&1
}

ensure_rust() {
	if [ -f "$HOME/.cargo/env" ]; then
		# shellcheck disable=SC1091
		source "$HOME/.cargo/env"
	fi

	if command -v cargo >/dev/null 2>&1; then
		return
	fi

	echo "Installing Rust toolchain via rustup..."
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
	# shellcheck disable=SC1091
	source "$HOME/.cargo/env"
}

ensure_worker_build() {
	if command -v worker-build >/dev/null 2>&1; then
		return
	fi

	echo "Installing worker-build..."
	cargo install worker-build
}

main() {
	if is_local_dev; then
		# Fast path: just build, skip bootstrap checks
		cd "$BACKEND_DIR"
		worker-build --release
	else
		# Deploy Button path: bootstrap Rust and tools
		ensure_rust
		ensure_worker_build
		cd "$BACKEND_DIR"
		worker-build --release
	fi
}

main "$@"

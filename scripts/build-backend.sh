#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/apps/backend"

if [ -f "$HOME/.cargo/env" ]; then
	# shellcheck disable=SC1091
	source "$HOME/.cargo/env"
fi

ensure_rust() {
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
	ensure_rust
	ensure_worker_build

	cd "$BACKEND_DIR"
	worker-build --release
}

main "$@"

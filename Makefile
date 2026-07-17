lint:
	cargo fmt-check
	cargo lint
	cargo xtask codegen --check

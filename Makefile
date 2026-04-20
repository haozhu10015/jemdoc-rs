BLUE  := \033[36m
BOLD  := \033[1m
RESET := \033[0m

.DEFAULT_GOAL := ci

.PHONY: ci fmt clippy test snapshots update-snapshots publish-check clean help

ci: fmt clippy test snapshots publish-check ## run all CI checks

fmt: ## check formatting
	@printf "$(BLUE)fmt$(RESET)\n"
	@cargo fmt --all -- --check

clippy: ## run clippy
	@printf "$(BLUE)clippy$(RESET)\n"
	@cargo clippy --all-targets --all-features -- -D warnings

test: ## run tests and build docs
	@printf "$(BLUE)test$(RESET)\n"
	@cargo test --all-features
	@cargo test --no-default-features
	@cargo doc --no-deps --all-features

snapshots: ## check example HTML snapshots
	@printf "$(BLUE)snapshots$(RESET)\n"
	@cargo build --release 2>/dev/null
	@cd example && ../target/release/jemdoc-rs -c mysite.conf *.jemdoc
	@git diff --exit-code -- example/*.html || { \
		echo ""; \
		echo "Example HTML is out of date. Run: make update-snapshots"; \
		exit 1; \
	}

update-snapshots: ## regenerate and commit example HTML snapshots
	@printf "$(BLUE)update-snapshots$(RESET)\n"
	@cargo build --release 2>/dev/null
	@cd example && ../target/release/jemdoc-rs -c mysite.conf *.jemdoc
	@printf "$(BLUE)Snapshots updated. Stage with: git add example/*.html$(RESET)\n"

publish-check: ## verify the package can be published
	@printf "$(BLUE)publish-check$(RESET)\n"
	@cargo publish --dry-run

clean: ## remove build artifacts
	@cargo clean

help: ## display this help message
	@printf "$(BOLD)Usage:$(RESET)\n"
	@printf "  make $(BLUE)<target>$(RESET)\n\n"
	@printf "$(BOLD)Targets:$(RESET)\n"
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(BLUE)%-20s$(RESET) %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

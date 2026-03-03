# --- Configuration ---------------------------------------------------------
BLUE  := \033[36m
BOLD  := \033[1m
RESET := \033[0m

CONF  := mysite.conf
JEMDOC_SRC := *.jemdoc
JEMDOC_RS_REPO := https://github.com/haozhu10015/jemdoc-rs.git

.DEFAULT_GOAL := help

# --- Helpers (auto-install if missing) ------------------------------------
ensure = @command -v $(1) >/dev/null 2>&1 || \
	{ printf "$(BLUE)$(1) not found, installing...$(RESET)\n" && $(2); }

# --- Targets --------------------------------------------------------------
.PHONY: install jemdoc update preview clean help

install: ## install all dependencies (requires Rust/Cargo)
	@command -v cargo >/dev/null 2>&1 || { printf "cargo not found, install Rust first: https://rustup.rs\n"; exit 1; }
	@printf "$(BLUE)Installing jemdoc-rs...$(RESET)\n"
	@cargo install --git $(JEMDOC_RS_REPO)
	@printf "$(BLUE)Installing simple-http-server...$(RESET)\n"
	@cargo install simple-http-server

jemdoc: ## generate all jemdoc pages
	$(call ensure,jemdoc-rs,cargo install --git $(JEMDOC_RS_REPO))
	@printf "$(BLUE)Generating jemdoc webpage...$(RESET)\n"
	@jemdoc-rs -c $(CONF) $(JEMDOC_SRC)

preview: ## preview the webpage locally
	$(call ensure,simple-http-server,cargo install simple-http-server)
	@printf "$(BLUE)Starting local web server on http://127.0.0.1:8000...$(RESET)\n"
	@simple-http-server -p 8000 --ip 127.0.0.1 -i .

clean: ## clean generated files and directories
	@printf "$(BLUE)Cleaning project...$(RESET)\n"
	@git clean -d -X -f

help: ## display this help message
	@printf "$(BOLD)Usage:$(RESET)\n"
	@printf "  make $(BLUE)<target>$(RESET)\n\n"
	@printf "$(BOLD)Targets:$(RESET)\n"
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(BLUE)%-10s$(RESET) %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
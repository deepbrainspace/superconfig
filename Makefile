init:
	curl -fsSL https://moonrepo.dev/install/proto.sh | bash
	proto install rust
	proto install moon
	proto plugin add direnv "https://raw.githubusercontent.com/appthrust/proto-toml-plugins/main/direnv/plugin.toml"
	proto install direnv
	cargo install cargo-examples
	proto use moon
	@echo "🎉 Development tools installed successfully!"
	@echo ""
	@echo "📋 Installed versions:"
	@echo "  proto:         $$(proto --version 2>/dev/null || echo 'not found')"
	@echo "  rust:          $$(rustc --version 2>/dev/null || echo 'not found')"
	@echo "  cargo:         $$(cargo --version 2>/dev/null || echo 'not found')"
	@echo "  moon:          $$(moon --version 2>/dev/null || echo 'not found')"
	@echo "  direnv:        $$(direnv --version 2>/dev/null || echo 'not found')"
	@echo "  cargo-examples: $$(cargo examples --version 2>/dev/null || echo 'not found')"
	@echo ""
	@echo "✅ Ready to develop! Try 'moon --help' to get started."

.PHONY: init
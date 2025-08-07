init:
	curl -fsSL https://moonrepo.dev/install/proto.sh | bash
	proto use
	cargo install cargo-examples

.PHONY: init
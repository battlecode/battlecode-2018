include colors.mk

build:
	@$(MAKE) -wC frankenstein
	$(call build_command,cargo build)

test: build
	$(call test_command,cargo test)
	@$(MAKE) -wC frankenstein test

.PHONY: build test
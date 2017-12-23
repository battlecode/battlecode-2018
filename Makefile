include colors.mk

build:
	$(call build_command,cargo build)
	@$(MAKE) -wC frankenstein

test:
	$(call test_command,cargo test)
	@$(MAKE) -wC frankenstein test

.PHONY: build test
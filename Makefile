include colors.mk

build:
	@$(MAKE) -wC bindings
	$(call build_command,cargo build)

test:
	@$(MAKE) -wC bindings test

.PHONY: build test